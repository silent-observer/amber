use std::collections::HashMap;

use crate::pins::PinState;

use super::{VcdTree, fillers::VcdFiller, VcdTreeSignal};

/// A configuration for VCD dump
pub enum VcdConfig {
    Module(HashMap<String, VcdConfig>),
    Enable,
    Disable,
}

impl VcdConfig {
    pub fn make_signal(&self, size: u16, val: PinState) -> VcdTree {
        match self {
            Self::Enable => VcdTree::Signal(VcdTreeSignal::new(size, val)),
            Self::Disable => VcdTree::Disabled,
            Self::Module(_) => panic!("Expected leaf config!"),
        }
    }

    pub fn make_node<T: VcdFiller>(&self, filler: &T) -> VcdTree {
        match self {
            Self::Enable | Self::Module(_) => filler.init_vcd(self),
            Self::Disable => VcdTree::Disabled,
        }
    }

    pub fn get(&self, key: &str) -> &VcdConfig {
        match self {
            Self::Enable | Self::Disable => self,
            Self::Module(hash_map) => {
                hash_map.get(key).unwrap_or(&Self::Disable)
            }
        }
    }
}

#[macro_export]
macro_rules! vcd_config_module_list {
    ($hash_map:expr; ) => {};
    ($hash_map:expr; $x:ident : {$($internal:tt),*} $(,$tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), {
            let mut submodule = std::collections::HashMap::new();
            $crate::vcd_config_module_list!(submodule; $($internal),*);
            VcdConfig::Module(submodule)
        });
        $crate::vcd_config_module_list!($hash_map; $($tail),*)
    };
    ($hash_map:expr; $x:ident $(,$tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), VcdConfig::Enable);
        $crate::vcd_config_module_list!($hash_map; $($tail),*)
    };
}

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