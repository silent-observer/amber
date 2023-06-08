use crate::pins::PinState;

use super::{config::VcrConfig, VcrTree, VcrTreeModule, VcrTreeSignal, builder::VcrModuleBuilder};

pub trait VcrFiller {
    const IS_SIGNAL: bool;

    fn init_vcr_module(&self, builder: &mut VcrModuleBuilder) {
        panic!("Cannot init this as a module")
    }
    fn init_vcr_signal(&self) -> VcrTreeSignal {
        panic!("Cannot init this as a signal")
    }

    fn init_vcr(&self, config: &VcrConfig) -> VcrTree {
        if Self::IS_SIGNAL {
            match config {
                VcrConfig::Enable => VcrTree::Signal(self.init_vcr_signal()),
                VcrConfig::Disable => VcrTree::Disabled,
                VcrConfig::Module(_) => panic!("This can only init a signal node!"),
            }
        } else {
            match config {
                VcrConfig::Module(_) | VcrConfig::Enable => {
                    let mut builder = VcrModuleBuilder::new(config);
                    self.init_vcr_module(&mut builder);
                    VcrTree::Module(builder.take())
                },
                VcrConfig::Disable => VcrTree::Disabled,
            }
        }
    }

    fn fill_module(&self, _module: &mut VcrTreeModule) {
        panic!("Cannot fill this as a module")
    }
    fn get_signal_state(&self) -> Vec<PinState> {
        panic!("Cannot fill this as a signal")
    }

    fn fill_vcr(&self, tree: &mut VcrTree) {
        if Self::IS_SIGNAL {
            match tree {
                VcrTree::Module(_) => panic!("This can only fill a signal node!"),
                VcrTree::Disabled => {},
                VcrTree::Signal(signal) => {
                    signal.update(self.get_signal_state())
                }
            }
        } else {
            match tree {
                VcrTree::Module(module) => self.fill_module(module),
                VcrTree::Disabled => {},
                VcrTree::Signal(_) => panic!("This can only fill a module node!"),
            }
        }
    }
}