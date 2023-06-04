use bitfield::Bit;

use crate::mcu::avr::{mcu_model::McuModel, bit_helpers::{get_rd_fields, get_d_field, get_k6, get_k8}};

use super::Mcu;

impl<M:McuModel> Mcu<M> {

    fn status_add(&mut self, rd: u8, rr: u8, r: u8) {
        let rd7 = rd.bit(7);
        let rr7 = rr.bit(7);
        let r7 = r.bit(7);
        let rd3 = rd.bit(3);
        let rr3 = rr.bit(3);
        let r3 = r.bit(3);

        self.sreg.set_c(rd7 && rr7 || rd7 && !r7 || !r7 && rr7);
        self.sreg.set_z(r == 0x00);
        self.sreg.set_n(r7);
        self.sreg.set_v(rd7 && rr7 && !r7 || !rd7 && !rr7 && r7);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(rd3 && rr3 || rd3 && !r3 || !r3 && rr3);
    }

    pub fn instr_add(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rr.wrapping_add(rd);

        self.write_register(d, result);
        self.status_add(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_adc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rr.wrapping_add(rd).wrapping_add(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_add(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_adiw(&mut self, opcode: u16) -> u8 {
        let k = get_k6(opcode) as u16;
        let d = get_d_field(opcode, 2);
        let rd = self.read_register_pair(d);
        let result = rd.wrapping_add(k);

        self.write_register_pair(d, result);
        
        let rd15 = rd.bit(15);
        let r15 = result.bit(15);

        self.sreg.set_c(!r15 && rd15);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(r15);
        self.sreg.set_v(!rd15 && r15);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());

        self.pc += 1;
        2
    }

    fn status_sub(&mut self, rd: u8, rr: u8, r: u8) {
        let rd7 = rd.bit(7);
        let rr7 = rr.bit(7);
        let r7 = r.bit(7);
        let rd3 = rd.bit(3);
        let rr3 = rr.bit(3);
        let r3 = r.bit(3);

        self.sreg.set_c(!rd7 && rr7 || rr7 && r7 || r7 && !rd7);
        self.sreg.set_z(r == 0x00);
        self.sreg.set_n(r7);
        self.sreg.set_v(rd7 && !rr7 && !r7 || !rd7 && rr7 && r7);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(!rd3 && rr3 || rr3 && r3 || r3 && !rd3);
    }

    pub fn instr_sub(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr);

        self.write_register(d, result);
        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_subi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k);

        self.write_register(d, result);
        self.status_sub(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr).wrapping_sub(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbci(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k).wrapping_sub(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_sub(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbiw(&mut self, opcode: u16) -> u8 {
        let k = get_k6(opcode) as u16;
        let d = get_d_field(opcode, 2);
        let rd = self.read_register_pair(d);
        let result = rd.wrapping_add(k);

        self.write_register_pair(d, result);
        
        let rd15 = rd.bit(15);
        let r15 = result.bit(15);

        self.sreg.set_c(r15 && !rd15);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(r15);
        self.sreg.set_v(r15 && !rd15);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());

        self.pc += 1;
        2
    }

    pub fn instr_inc(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd.wrapping_add(1);

        self.write_register(d, result);
        
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x80);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.pc += 1;

        1
    }

    pub fn instr_dec(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(1);

        self.write_register(d, result);
        
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x7F);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.pc += 1;

        1
    }

    pub fn instr_cp(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr);

        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_cpi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k);

        self.status_sub(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_cpc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr).wrapping_sub(self.sreg.c() as u8);

        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }
}