use crate::pins::PinState;

use super::{config::VcdConfig, VcdTreeModule, VcdTree, VcdTreeSignal, fillers::VcdFiller};

/// Builder struct for constructing [VcdTreeModule] using a [VcdConfig].
pub struct VcdModuleBuilder<'a> {
    config: &'a VcdConfig,
    module: VcdTreeModule
}

impl<'a> VcdModuleBuilder<'a> {
    /// Creates a new [VcdModuleBuilder] from a [VcdConfig].
    pub fn new<'b>(config: &'b VcdConfig) -> VcdModuleBuilder<'b> {
        VcdModuleBuilder { config, module: VcdTreeModule::new() }
    }

    /// Adds a new signal under a given `name`, with a given `size` and initial value.
    pub fn add_signal(&mut self, name: &str, size: u8, val: PinState) {
        match self.config.get(name) {
            VcdConfig::Enable => {
                let s = VcdTree::Signal(VcdTreeSignal::new(size, val));
                self.module.add(name, s);
            }
            VcdConfig::Disable => {},
            VcdConfig::Module(_) => panic!("Expected leaf config!"),
        }
    }

    /// Adds a new node filled with a specified `filler`.
    pub fn add_node<T: VcdFiller>(&mut self, name: &str, filler: &T)  {
        match self.config.get(name) {
            VcdConfig::Enable | VcdConfig::Module(_) => {
                let n = filler.init_vcd(self.config.get(name));
                self.module.add(name, n);
            },
            VcdConfig::Disable => {},
        }
    }
    
    /// Takes the resulting [VcdTreeModule], moving it out of the [VcdModuleBuilder].
    pub fn take(self) -> VcdTreeModule {
        self.module
    }
}