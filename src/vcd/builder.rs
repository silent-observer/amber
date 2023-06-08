use crate::pins::PinState;

use super::{config::VcdConfig, VcdTreeModule, VcdTree, VcdTreeSignal, fillers::VcdFiller};

pub struct VcdModuleBuilder<'a> {
    config: &'a VcdConfig,
    module: VcdTreeModule
}

impl<'a> VcdModuleBuilder<'a> {
    pub fn new<'b>(config: &'b VcdConfig) -> VcdModuleBuilder<'b> {
        VcdModuleBuilder { config, module: VcdTreeModule::new() }
    }

    pub fn add_signal(&mut self, name: &str, size: u16, val: PinState) {
        match self.config.get(name) {
            VcdConfig::Enable => {
                let s = VcdTree::Signal(VcdTreeSignal::new(size, val));
                self.module.add(name, s);
            }
            VcdConfig::Disable => {},
            VcdConfig::Module(_) => panic!("Expected leaf config!"),
        }
    }

    pub fn add_node<T: VcdFiller>(&mut self, name: &str, filler: &T)  {
        match self.config.get(name) {
            VcdConfig::Enable | VcdConfig::Module(_) => {
                let n = filler.init_vcd(self.config.get(name));
                self.module.add(name, n);
            },
            VcdConfig::Disable => {},
        }
    }
    
    pub fn take(self) -> VcdTreeModule {
        self.module
    }
}