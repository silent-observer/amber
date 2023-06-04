use crate::mcu::avr::mcu_model::McuModel;

use super::{SRAM_SIZE, Mcu};

impl<M:McuModel> Mcu<M> {
    pub fn read_register(&self, i: u16) -> u8 {
        assert!(i < 32);
        self.reg_file.regs[i as usize]
    }

    pub fn write_register(&mut self, i: u16, val: u8) {
        assert!(i < 32);
        self.reg_file.regs[i as usize] = val;
    }

    pub fn read_register_pair(&self, i: u16) -> u16 {
        assert!(i < 32);
        self.reg_file.read_u16(i as usize)
    }

    pub fn write_register_pair(&mut self, i: u16, val: u16) {
        assert!(i < 32);
        self.reg_file.write_u16(i as usize, val);
    }

    pub fn read_io(&self, i: u8) -> u8 {
        self.io.read_internal_u8(i)
    }

    pub fn write_io(&mut self, i: u8, val: u8) {
        self.io.write_internal_u8(i, val)
    }

    pub fn read_flash(&self, addr: u32) -> u16 {
        self.flash[addr as usize]
    }

    pub fn write_flash(&mut self, addr: u32, val: u16) {
        self.flash[addr as usize] = val
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x20 {
            self.read_register(addr)
        } else if addr < 0x60 {
            self.read_io((addr - 0x20) as u8)
        } else if addr < 0x200 {
            self.io.read_external_u8(addr)
        } else if addr < 0x200 + SRAM_SIZE as u16 {
            self.sram[addr as usize - 0x200]
        } else {
            0
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x20 {
            self.write_register(addr, val);
        } else if addr < 0x60 {
            self.write_io((addr - 0x20) as u8, val);
        } else if addr < 0x200 {
            self.io.write_external_u8(addr, val);
        } else if addr < 0x200 + SRAM_SIZE as u16 {
            self.sram[addr as usize - 0x200] = val;
        }
    }

    pub fn read_at_pc_offset(&self, x: u32) -> u16 {
        self.read_flash(self.pc + x)
    }

    pub fn read_at_sp_offset(&self, x: i16) -> u8 {
        self.read(self.sp.wrapping_add(x as u16))
    }
    pub fn write_at_sp_offset(&mut self, x: i16, val: u8) {
        self.write(self.sp.wrapping_add(x as u16), val)
    }

    pub fn rampz_address(&self, z: u16) -> u32 {
        (self.rampz as u32) << 16 | z as u32
    }
    pub fn eind_address(&self, z: u16) -> u32 {
        (self.eind as u32) << 16 | z as u32
    }
}