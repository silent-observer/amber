use crate::pins::PinState;

use super::{config::VcrConfig, VcrTreeModule, VcrTree, VcrTreeSignal, fillers::VcrFiller};

pub struct VcrModuleBuilder<'a> {
    config: &'a VcrConfig,
    module: VcrTreeModule
}

impl<'a> VcrModuleBuilder<'a> {
    pub fn new<'b>(config: &'b VcrConfig) -> VcrModuleBuilder<'b> {
        VcrModuleBuilder { config, module: VcrTreeModule::new() }
    }

    pub fn add_signal(&mut self, name: &str, size: u16, val: PinState) {
        match self.config.get(name) {
            VcrConfig::Enable => {
                let s = VcrTree::Signal(VcrTreeSignal::new(size, val));
                self.module.add(name, s);
            }
            VcrConfig::Disable => {},
            VcrConfig::Module(_) => panic!("Expected leaf config!"),
        }
    }

    pub fn add_node<T: VcrFiller>(&mut self, name: &str, filler: &T)  {
        match self.config.get(name) {
            VcrConfig::Enable | VcrConfig::Module(_) => {
                let n = filler.init_vcr(self.config.get(name));
                self.module.add(name, n);
            },
            VcrConfig::Disable => {},
        }
    }
    
    pub fn take(self) -> VcrTreeModule {
        self.module
    }
}