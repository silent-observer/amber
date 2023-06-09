use std::collections::HashMap;

use crate::pins::PinState;

use super::{VcdTree, fillers::VcdFiller, VcdTreeSignal};

/// A configuration for VCD dump.
/// 
/// Determines if a module/signal should be included in the VCD.
pub enum VcdConfig {
    Module(HashMap<String, VcdConfig>),
    Enable,
    Disable,
}

impl VcdConfig {
    /// Creates a new signal according to the config.
    /// 
    /// If the signal is disabled, it is not created.
    pub fn make_signal(&self, size: u16, val: PinState) -> VcdTree {
        match self {
            Self::Enable => VcdTree::Signal(VcdTreeSignal::new(size, val)),
            Self::Disable => VcdTree::Disabled,
            Self::Module(_) => panic!("Expected leaf config!"),
        }
    }

    /// Creates a new node (using a [VcdFiller]) according to the config.
    /// 
    /// If the node is disabled, it is not created.
    pub fn make_node<T: VcdFiller>(&self, filler: &T) -> VcdTree {
        match self {
            Self::Enable | Self::Module(_) => filler.init_vcd(self),
            Self::Disable => VcdTree::Disabled,
        }
    }

    /// Gets a config sub-node.
    /// 
    /// If the node is missing, it is considered disabled.
    /// Sub-nodes of enabled and disabled leaf nodes are also
    /// considered enabled and disabled respectively.
    pub fn get(&self, key: &str) -> &VcdConfig {
        match self {
            Self::Enable | Self::Disable => self,
            Self::Module(hash_map) => {
                hash_map.get(key).unwrap_or(&Self::Disable)
            }
        }
    }
}


/// A helper macro for `vcd_config`
#[macro_export]
macro_rules! vcd_config_module_list {
    ($hash_map:expr; ) => {};
    ($hash_map:expr; $x:ident : {$($internal:tt)*} $($tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), {
            let mut submodule = std::collections::HashMap::new();
            $crate::vcd_config_module_list!(submodule; $($internal)*);
            VcdConfig::Module(submodule)
        });
        $crate::vcd_config_module_list!($hash_map; $($tail)*)
    };
    ($hash_map:expr; $x:ident $($tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), VcdConfig::Enable);
        $crate::vcd_config_module_list!($hash_map; $($tail)*)
    };
}

/// A declarative macro to create a [VcdConfig].
/// 
/// ```
/// # use amber::{vcd_config, vcd::VcdConfig};
/// let m = vcd_config!{
///     module1: {
///         signal1_1
///         signal1_2
///         module1_3: {
///             signal1_3_1
///         }
///     }
///     module2: {
///         signal2
///     }
///     signal3
///     signal4
/// };
/// ```
#[macro_export]
macro_rules! vcd_config {
    ($($tail:tt)*) => {
        {
            let mut hash_map = std::collections::HashMap::new();
            $crate::vcd_config_module_list!(hash_map; $($tail)*);
            VcdConfig::Module(hash_map)
        }
    };
}