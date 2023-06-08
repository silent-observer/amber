use std::collections::HashMap;

use crate::{pins::{PinState, PinStateConvertible}, vcr::{fillers::VcrFiller, builder::VcrModuleBuilder, VcrTreeModule}};

pub struct RegisterFile {
    pub regs: [u8; 32],
}

impl RegisterFile {
    pub fn new() -> RegisterFile {
        RegisterFile {
            regs: [0; 32],
        }
    }

    pub fn read_u16(&self, i: usize) -> u16 {
        (self.regs[i+1] as u16) << 8 | (self.regs[i] as u16)
    }

    pub fn write_u16(&mut self, i: usize, val: u16) {
        self.regs[i] = val as u8;
        self.regs[i+1] = (val >> 8) as u8;
    }
}

impl VcrFiller for RegisterFile {
    const IS_SIGNAL: bool = false;

    fn init_vcr_module(&self, builder: &mut VcrModuleBuilder) {
        for i in 0..32 {
            let s = format!("r{}", i);
            builder.add_signal(&s, 8, PinState::Low);
        }
    }

    fn fill_module(&self, module: &mut VcrTreeModule) {
        for i in 0..32 {
            let s = format!("r{}", i);
            module.update_subsignal(&s, self.regs[i].to_pin_vec());
        }
    }
}


// impl VcrFillerNode for RegisterFile {
//     fn fill_vcr(&self, hash_map: &mut std::collections::HashMap<String, VcrTree>) {
//         for i in 0..32 {
//             let s = format!("r{}", i);
//             hash_map
//                 .get_mut(&s)
//                 .expect("No register key")
//                 .update_leaf(self.regs[i]);
//         }
//     }

//     fn init_vcr(&self, hash_map: &mut std::collections::HashMap<String, VcrTree>) {
//         for i in 0..32 {
//             let s = format!("r{}", i);
//             hash_map.insert(s, VcrTree::new_leaf(8, PinState::Low));
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_read_u16() {
        let mut reg_file = RegisterFile::new();
        reg_file.regs[26] = 0x34;
        reg_file.regs[27] = 0x12;
        assert_eq!(reg_file.read_u16(26), 0x1234);
    }

    #[test]
    fn reg_write_u16() {
        let mut reg_file = RegisterFile::new();
        reg_file.write_u16(26, 0x1234);
        assert_eq!(reg_file.regs[26], 0x34);
        assert_eq!(reg_file.regs[27], 0x12);
    }
}