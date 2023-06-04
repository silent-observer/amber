use bitfield::Bit;

use crate::mcu::avr::{mcu_model::McuModel, bit_helpers::{get_d_field, get_io5, bit_field_combined}};

use super::{Mcu};

impl<M:McuModel> Mcu<M> {
    pub fn instr_sbi(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let mut val = self.read_io(io);
        val.set_bit(b as usize, true);

        self.pc += 1;
        2
    }

    pub fn instr_cbi(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let mut val = self.read_io(io);
        val.set_bit(b as usize, false);

        self.pc += 1;
        2
    }

    fn status_shr(&mut self, rd: u8, result: u8) {
        self.sreg.set_c(rd.bit(0));
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(self.sreg.n() ^ self.sreg.c());
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
    }

    pub fn instr_lsr(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1;

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_ror(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1 | if self.sreg.c() {0x80} else {0x00};

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_asr(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1 | if rd.bit(7) {0x80} else {0x00};

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_swap(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = (rd >> 4) | (rd << 4);

        self.write_register(d, result);
        self.pc += 1;

        1
    }

    pub fn instr_bset(&mut self, opcode: u16) -> u8 {
        let b = bit_field_combined(opcode, &[6..=4]);
        self.sreg.set_bit(b as usize, true);
        self.pc += 1;
        1
    }

    pub fn instr_bclr(&mut self, opcode: u16) -> u8 {
        let b = bit_field_combined(opcode, &[6..=4]);
        self.sreg.set_bit(b as usize, false);
        self.pc += 1;
        1
    }

    pub fn instr_bst(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 3);
        let rd = self.read_register(d);
        let b = opcode & 0x0007;
        self.sreg.set_t(rd.bit(b as usize));
        self.pc += 1;
        1
    }

    pub fn instr_bld(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 3);
        let mut rd = self.read_register(d);
        let b = opcode & 0x0007;
        rd.set_bit(b as usize, self.sreg.t());
        self.write_register(d, rd);
        self.pc += 1;
        1
    }
}