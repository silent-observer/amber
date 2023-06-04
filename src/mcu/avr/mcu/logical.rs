use bitfield::Bit;

use crate::mcu::avr::{
    mcu_model::McuModel, 
    bit_helpers::{get_rd_fields, get_k8, get_d_field}
};

use super::{Mcu};

impl<M:McuModel> Mcu<M> {
    fn status_logic(&mut self, r: u8) {
        self.sreg.set_z(r == 0);
        self.sreg.set_n(r.bit(7));
        self.sreg.set_v(false);
        self.sreg.set_s(r.bit(7));
    }

    pub fn instr_and(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd & rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_andi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd & k;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_or(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd | rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_ori(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd | k;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_eor(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd ^ rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_com(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = !rd;

        self.write_register(d, result);
        
        self.sreg.set_c(true);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_neg(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = 0x00u8.wrapping_sub(rd);

        self.write_register(d, result);
        
        self.sreg.set_c(result != 0);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x80);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(!rd.bit(3) & result.bit(3));
        self.pc += 1;

        1
    }



    
}