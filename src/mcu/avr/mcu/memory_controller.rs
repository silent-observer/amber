use crate::mcu::avr::{mcu_model::McuModel, io_controller::IoControllerTrait};

use super::{SRAM_SIZE, Mcu};

impl<M, Io> Mcu<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
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

#[cfg(test)]
mod tests {
    use crate::mcu::avr::mcu_model::Atmega2560;

    use super::*;

    #[test]
    fn memory_reads() {
        let mut mcu: Mcu<Atmega2560, _> = Mcu::default();
        mcu.flash[0..4].clone_from_slice(&[1, 2, 3, 4]);
        mcu.sram[0..4].clone_from_slice(&[5, 6, 7, 8]);
        mcu.reg_file.regs[0..4].clone_from_slice(&[9, 10, 11, 12]);

        assert_eq!(mcu.read_flash(0x0), 1);
        assert_eq!(mcu.read_flash(0x1), 2);
        assert_eq!(mcu.read_flash(0x2), 3);
        assert_eq!(mcu.read_flash(0x3), 4);

        assert_eq!(mcu.read(0x0), 9);
        assert_eq!(mcu.read(0x1), 10);
        assert_eq!(mcu.read(0x2), 11);
        assert_eq!(mcu.read(0x3), 12);

        assert_eq!(mcu.read(0x200), 5);
        assert_eq!(mcu.read(0x201), 6);
        assert_eq!(mcu.read(0x202), 7);
        assert_eq!(mcu.read(0x203), 8);
    }

    #[test]
    fn memory_writes() {
        let mut mcu: Mcu<Atmega2560, _> = Mcu::default();

        mcu.write_flash(0x0, 1);
        mcu.write_flash(0x1, 2);
        mcu.write_flash(0x2, 3);
        mcu.write_flash(0x3, 4);

        mcu.write(0x200, 5);
        mcu.write(0x201, 6);
        mcu.write(0x202, 7);
        mcu.write(0x203, 8);

        mcu.write(0x0, 9);
        mcu.write(0x1, 10);
        mcu.write(0x2, 11);
        mcu.write(0x3, 12);

        assert_eq!(mcu.flash[0..4], [1, 2, 3, 4]);
        assert_eq!(mcu.sram[0..4], [5, 6, 7, 8]);
        assert_eq!(mcu.reg_file.regs[0..4], [9, 10, 11, 12]);
    }

    #[test]
    fn memory_extended() {
        let mut mcu: Mcu<Atmega2560, _> = Mcu::default();

        mcu.rampz = 0x12;
        mcu.eind = 0x34;
        let z = 0x5678_u16;
        assert_eq!(mcu.rampz_address(z), 0x00125678_u32);
        assert_eq!(mcu.eind_address(z), 0x00345678_u32);
    }
}