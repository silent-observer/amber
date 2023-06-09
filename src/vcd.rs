//! Value Change Dump exporter module.
//! 
//! This module is used to capture signals from the components and write them to a VCD file.

pub mod fillers;
pub mod config;
pub mod writer;
pub mod builder;

use std::{collections::HashMap, sync::{Mutex, Arc}};

use crate::pins::{PinState, PinStateConvertible};

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

/// A recursive node of a VCD tree.
///
/// Can contain other modules and signals, accessed by their names.
#[derive(Debug, Clone)]

pub struct VcdTreeModule(HashMap<String, VcdTree>);

/// A leaf node of a VCD tree, containing a signal.
#[derive(Debug, Clone)]
pub struct VcdTreeSignal {
    /// Number of bits in the signal
    size: u16,
    /// Short ASCII id assigned to the signal by VCD
    id: Option<String>,
    /// Previous state of the signal, used to track changes
    old_state: Vec<PinState>,
    /// Current state of the signal
    new_state: Vec<PinState>,
}

/// VCD tree behind a mutex, for passing to other threads.
pub type MutexVcdTree = Arc<Mutex<VcdTree>>;

/// Top level structure, containing multiple VCD trees for different components.
/// 
/// Every VCD tree in the [VcdForest] is stored behind a mutex,
/// since every component works in a different thread.
pub struct VcdForest(HashMap<String, MutexVcdTree>);

impl VcdTreeModule {
    /// Creates a new empty module.
    pub fn new() -> VcdTreeModule {
        VcdTreeModule(HashMap::new())
    }

    /// Adds a child VCD tree `val` under name `key`.
    pub fn add(&mut self, key: &str, val: VcdTree) {
        self.0.insert(key.to_string(), val);
    }

    /// Updates a child signal in this module.
    pub fn update_subsignal(&mut self, key: &str, state: Vec<PinState>) {
        if let Some(child) = self.0.get_mut(key) {
            match child {
                VcdTree::Module(_) => panic!("Cannot update module as signal!"),
                VcdTree::Signal(signal) => signal.update(state),
                VcdTree::Disabled => {},
            }
        }
    }

    /// Updates a child tree in this module using a [VcdFiller].
    pub fn update_child<T: VcdFiller>(&mut self, key: &str, filler: &T) {
        if let Some(child) = self.0.get_mut(key) {
            filler.fill_vcd(child);
        }
    }
}

impl VcdTreeSignal {
    /// Creates a new signal of specified size, filled initially with `val`.
    pub fn new(size: u16, val: PinState) -> VcdTreeSignal {
        VcdTreeSignal {
            size,
            id: None,
            old_state: vec![val; size as usize],
            new_state: vec![val; size as usize]
        }
    }

    /// Updates signal's state.
    pub fn update<T: PinStateConvertible>(&mut self, state: T) {
        self.old_state = std::mem::replace(&mut self.new_state, state.to_pin_vec().to_vec());
    }
}

impl VcdForest {
    /// Creates an empty VCD forest.
    pub fn new() -> VcdForest {
        VcdForest(HashMap::new())
    }

    /// Adds a new component into the forest with the name `key`.
    pub fn add(&mut self, key: &str, val: VcdTree) -> MutexVcdTree {
        let mutex = Arc::new(Mutex::new(val));
        self.0.insert(key.to_string(), mutex.clone());
        mutex
    }
}