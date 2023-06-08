use std::collections::HashMap;

use crate::pins::PinState;

use super::{VcrTree, fillers::VcrFiller, VcrTreeSignal};

pub enum VcrConfig {
    Module(HashMap<String, VcrConfig>),
    Enable,
    Disable,
}

impl VcrConfig {
    pub fn make_signal(&self, size: u16, val: PinState) -> VcrTree {
        match self {
            Self::Enable => VcrTree::Signal(VcrTreeSignal::new(size, val)),
            Self::Disable => VcrTree::Disabled,
            Self::Module(_) => panic!("Expected leaf config!"),
        }
    }

    pub fn make_node<T: VcrFiller>(&self, filler: &T) -> VcrTree {
        match self {
            Self::Enable | Self::Module(_) => filler.init_vcr(self),
            Self::Disable => VcrTree::Disabled,
        }
    }

    pub fn get(&self, key: &str) -> &VcrConfig {
        match self {
            Self::Enable | Self::Disable => self,
            Self::Module(hash_map) => {
                hash_map.get(key).unwrap_or(&Self::Disable)
            }
        }
    }
}

#[macro_export]
macro_rules! vcr_config_module_list {
    ($hash_map:expr; ) => {};
    ($hash_map:expr; $x:ident : {$($internal:tt),*} $(,$tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), {
            let mut submodule = std::collections::HashMap::new();
            $crate::vcr_config_module_list!(submodule; $($internal),*);
            VcrConfig::Module(submodule)
        });
        $crate::vcr_config_module_list!($hash_map; $($tail),*)
    };
    ($hash_map:expr; $x:ident $(,$tail:tt)*) => {
        $hash_map.insert(stringify!($x).to_string(), VcrConfig::Enable);
        $crate::vcr_config_module_list!($hash_map; $($tail),*)
    };
}

#[macro_export]
macro_rules! vcr_config {
    ($($tail:tt)*) => {
        {
            let mut hash_map = std::collections::HashMap::new();
            $crate::vcr_config_module_list!(hash_map; $($tail)*);
            VcrConfig::Module(hash_map)
        }
    };
}