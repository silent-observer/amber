//! Value Change Dump exporter module.
//! 
//! This module is used to capture signals from the components and write them to a VCD file.

pub mod fillers;
pub mod config;
pub mod writer;
pub mod builder;

use std::{sync::{Mutex, Arc}, cell::RefCell, rc::Rc};

use crate::pins::{PinState, PinStateConvertible, PinVec};

pub use fillers::VcdFiller;
pub use config::VcdConfig;
pub use builder::VcdModuleBuilder;
pub use writer::VcdWriter;

/// VCD snapshot of the whole module tree.
#[derive(Debug, Clone)]
pub enum VcdTree {
    Module(VcdTreeModule),
    Signal(VcdTreeSignal),
    Disabled
}

#[derive(Debug, Clone)]
pub struct VcdTreeModuleEntry {
    name: String,
    val: Option<VcdTree>
}

/// A recursive node of a VCD tree.
///
/// Can contain other modules and signals, accessed by their names.
#[derive(Debug, Clone)]
pub struct VcdTreeModule(Vec<VcdTreeModuleEntry>);

/// A leaf node of a VCD tree, containing a signal.
#[derive(Debug, Clone)]
pub struct VcdTreeSignal {
    /// Number of bits in the signal
    size: u8,
    /// Short ASCII id assigned to the signal by VCD
    id: Option<String>,
    /// Previous state of the signal, used to track changes
    old_state: PinVec,
    /// Current state of the signal
    new_state: PinVec,
}

pub struct VcdTreeHandle {
    pub tree: VcdTree
}

/// VCD tree behind a mutex, for passing to other threads.
enum MutexVcdTree {
    Threaded(Arc<Mutex<VcdTreeHandle>>),
    Threadless(Rc<RefCell<VcdTreeHandle>>),
}

/// Top level structure, containing multiple VCD trees for different components.
/// 
/// Every VCD tree in the [VcdForest] is stored behind a mutex,
/// since every component works in a different thread.
pub struct VcdForest(Vec<(String, MutexVcdTree)>);

impl VcdTreeModule {
    /// Creates a new empty module.
    pub fn new() -> VcdTreeModule {
        VcdTreeModule(Vec::new())
    }

    /// Adds a child VCD tree `val` under name `key`.
    #[inline]
    pub fn add(&mut self, key: &str, val: Option<VcdTree>) {
        self.0.push(VcdTreeModuleEntry { name: key.to_string(), val });
    }

    /// Updates a child signal in this module.
    #[inline]
    pub fn update_subsignal<T: PinStateConvertible>(&mut self, index: usize, state: T) -> bool {
        if let Some(child) = &mut self.0[index].val {
            match child {
                VcdTree::Module(_) => panic!("Cannot update module as signal!"),
                VcdTree::Signal(signal) => signal.update(state),
                VcdTree::Disabled => {false},
            }
        } else {
            false
        }
    }

    /// Updates a child tree in this module using a [VcdFiller].
    #[inline]
    pub fn update_child<T: VcdFiller>(&mut self, index: usize, filler: &T) -> bool {
        if let Some(child) = &mut self.0[index].val {
            filler.fill_vcd(child)
        } else {
            false
        }
    }
}

impl VcdTreeSignal {
    /// Creates a new signal of specified size, filled initially with `val`.
    pub fn new(size: u8, val: PinState) -> VcdTreeSignal {
        VcdTreeSignal {
            size,
            id: None,
            old_state: PinVec::new(size, val),
            new_state: PinVec::new(size, val)
        }
    }

    /// Updates signal's state.
    #[inline]
    pub fn update<T: PinStateConvertible>(&mut self, state: T) -> bool {
        self.old_state = std::mem::replace(&mut self.new_state, state.to_pin_vec());
        self.old_state != self.new_state
    }
}

impl VcdForest {
    /// Creates an empty VCD forest.
    pub fn new() -> VcdForest {
        VcdForest(Vec::new())
    }

    /// Adds a new threaded component into the forest with the name `key`.
    pub fn add_threaded(&mut self, key: &str, val: VcdTree) -> Arc<Mutex<VcdTreeHandle>> {
        let handle = VcdTreeHandle{tree: val};
        let mutex = Arc::new(Mutex::new(handle));
        self.0.push((key.to_string(), MutexVcdTree::Threaded(mutex.clone())));
        mutex
    }

    /// Adds a new threadless component into the forest with the name `key`.
    pub fn add_threadless(&mut self, key: &str, val: VcdTree) -> Rc<RefCell<VcdTreeHandle>> {
        let handle = VcdTreeHandle{tree: val};
        let rc = Rc::new(RefCell::new(handle));
        self.0.push((key.to_string(), MutexVcdTree::Threadless(rc.clone())));
        rc
    }
}