use crate::pins::PinState;

use super::{config::VcdConfig, VcdTree, VcdTreeModule, VcdTreeSignal, builder::VcdModuleBuilder};

pub trait VcdFiller {
    const IS_SIGNAL: bool;

    fn init_vcd_module(&self, _builder: &mut VcdModuleBuilder) {
        panic!("Cannot init this as a module")
    }
    fn init_vcd_signal(&self) -> VcdTreeSignal {
        panic!("Cannot init this as a signal")
    }

    fn init_vcd(&self, config: &VcdConfig) -> VcdTree {
        if Self::IS_SIGNAL {
            match config {
                VcdConfig::Enable => VcdTree::Signal(self.init_vcd_signal()),
                VcdConfig::Disable => VcdTree::Disabled,
                VcdConfig::Module(_) => panic!("This can only init a signal node!"),
            }
        } else {
            match config {
                VcdConfig::Module(_) | VcdConfig::Enable => {
                    let mut builder = VcdModuleBuilder::new(config);
                    self.init_vcd_module(&mut builder);
                    VcdTree::Module(builder.take())
                },
                VcdConfig::Disable => VcdTree::Disabled,
            }
        }
    }

    fn fill_module(&self, _module: &mut VcdTreeModule) {
        panic!("Cannot fill this as a module")
    }
    fn get_signal_state(&self) -> Vec<PinState> {
        panic!("Cannot fill this as a signal")
    }

    fn fill_vcd(&self, tree: &mut VcdTree) {
        if Self::IS_SIGNAL {
            match tree {
                VcdTree::Module(_) => panic!("This can only fill a signal node!"),
                VcdTree::Disabled => {},
                VcdTree::Signal(signal) => {
                    signal.update(self.get_signal_state())
                }
            }
        } else {
            match tree {
                VcdTree::Module(module) => self.fill_module(module),
                VcdTree::Disabled => {},
                VcdTree::Signal(_) => panic!("This can only fill a module node!"),
            }
        }
    }
}