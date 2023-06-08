pub mod fillers;
pub mod config;
pub mod writer;
pub mod builder;

use std::{collections::HashMap, sync::{Mutex, Arc}};

use crate::pins::{PinState, PinStateConvertible};

use self::fillers::VcrFiller;


#[derive(Debug, Clone)]

pub struct VcrTreeModule(HashMap<String, VcrTree>);

#[derive(Debug, Clone)]
pub struct VcrTreeSignal {
    size: u16,
    id: Option<String>,
    old_state: Vec<PinState>,
    new_state: Vec<PinState>,
}

#[derive(Debug, Clone)]
pub enum VcrTree {
    Module(VcrTreeModule),
    Signal(VcrTreeSignal),
    Disabled
}

pub type MutexVcrTree = Arc<Mutex<VcrTree>>;

pub struct VcrForest(HashMap<String, MutexVcrTree>);

impl VcrTreeModule {
    pub fn new() -> VcrTreeModule {
        VcrTreeModule(HashMap::new())
    }

    pub fn add(&mut self, key: &str, val: VcrTree) {
        self.0.insert(key.to_string(), val);
    }

    pub fn update_subsignal(&mut self, key: &str, state: Vec<PinState>) {
        if let Some(child) = self.0.get_mut(key) {
            match child {
                VcrTree::Module(_) => panic!("Cannot update module as signal!"),
                VcrTree::Signal(signal) => signal.update(state),
                VcrTree::Disabled => {},
            }
        }
    }

    pub fn update_child<T: VcrFiller>(&mut self, key: &str, filler: &T) {
        if let Some(child) = self.0.get_mut(key) {
            filler.fill_vcr(child);
        }
    }
}

impl VcrTreeSignal {
    pub fn new(size: u16, val: PinState) -> VcrTreeSignal {
        VcrTreeSignal {
            size,
            id: None,
            old_state: vec![val; size as usize],
            new_state: vec![val; size as usize]
        }
    }

    pub fn update<T: PinStateConvertible>(&mut self, state: T) {
        self.old_state = std::mem::replace(&mut self.new_state, state.to_pin_vec().to_vec());
    }
}

impl VcrForest {
    pub fn new() -> VcrForest {
        VcrForest(HashMap::new())
    }

    pub fn add(&mut self, key: &str, val: VcrTree) -> MutexVcrTree {
        let mutex = Arc::new(Mutex::new(val));
        self.0.insert(key.to_string(), mutex.clone());
        mutex
    }
}