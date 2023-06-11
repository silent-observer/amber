use crate::pins::PinVec;

use super::{config::VcdConfig, VcdTree, VcdTreeModule, VcdTreeSignal, builder::VcdModuleBuilder};

/// Something that has VCD signals inside.
/// 
/// Can be either one signal, or a module containing many named symbols.
/// Which option is used is determined by associated constant `IS_SIGNAL`.
pub trait VcdFiller {
    const IS_SIGNAL: bool;

    /// Initialize [VcdTree] using a [VcdModuleBuilder]
    /// 
    /// Override this if implementing a module.
    fn init_vcd_module(&self, _builder: &mut VcdModuleBuilder) {
        panic!("Cannot init this as a module")
    }
    /// Initialize [VcdTree] with a single signal.
    /// 
    /// Override this if implementing a signal.
    fn init_vcd_signal(&self) -> VcdTreeSignal {
        panic!("Cannot init this as a signal")
    }

    /// Initialize [VcdTree] from a config
    /// using either [init_vcd_module](VcdFiller::init_vcd_module) or
    /// [init_vcd_signal](VcdFiller::init_vcd_signal).
    /// 
    /// Override this if using a custom initializer.
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

    /// Update [VcdTreeModule] with new signals.
    /// 
    /// Override this if implementing a module.
    fn fill_module(&self, _module: &mut VcdTreeModule) -> bool {
        panic!("Cannot fill this as a module")
    }
    /// Read current signal state.
    /// 
    /// Override this if implementing a signal.
    fn get_signal_state(&self) -> PinVec {
        panic!("Cannot fill this as a signal")
    }

    /// Update existing [VcdTree] with new signals
    /// using either [fill_module](VcdFiller::fill_module) or
    /// [get_signal_state](VcdFiller::get_signal_state).
    /// 
    /// Override this if using a custom initializer.
    fn fill_vcd(&self, tree: &mut VcdTree) -> bool {
        if Self::IS_SIGNAL {
            match tree {
                VcdTree::Disabled => {false},
                VcdTree::Module(_) => panic!("This can only fill a signal node!"),
                VcdTree::Signal(signal) => {
                    signal.update(self.get_signal_state())
                }
            }
        } else {
            match tree {
                VcdTree::Disabled => {false},
                VcdTree::Module(module) => self.fill_module(module),
                VcdTree::Signal(_) => panic!("This can only fill a module node!"),
            }
        }
    }
}