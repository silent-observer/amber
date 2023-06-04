use bitfield::Bit;

use crate::mcu::avr::{mcu_model::McuModel, bit_helpers::{get_rd_fields, bit_field_combined, get_k8, get_d_field, get_io6}};

use super::{Mcu};

const X_REG: u16 = 26;
const Y_REG: u16 = 28;
const Z_REG: u16 = 30;

impl<M:McuModel> Mcu<M> {
    pub fn instr_mov(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);

        self.write_register(d, rr);
        self.pc += 1;

        1
    }

    pub fn instr_movw(&mut self, opcode: u16) -> u8 {
        let r = 
            bit_field_combined(opcode, &[3..=0]) << 1;
        let d =
            bit_field_combined(opcode, &[7..=4]) << 1;
        let rr = self.read_register_pair(r);

        self.write_register_pair(d, rr);
        self.pc += 1;

        1
    }

    pub fn instr_ldi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);

        self.write_register(d, k);
        self.pc += 1;

        1
    }

    pub fn instr_ld(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        
        match bit_field_combined(opcode, &[0..=3]) {
            0b0001 => {
                let addr = self.read_register_pair(Z_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(Z_REG, addr + 1);
            }
            0b0010 => {
                let addr = self.read_register_pair(Z_REG);
                let val = self.read(addr - 1);
                self.write_register(d, val);
                self.write_register_pair(Z_REG, addr - 1);
            }

            0b1001 => {
                let addr = self.read_register_pair(Y_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(Y_REG, addr + 1);
            }
            0b1010 => {
                let addr = self.read_register_pair(Y_REG);
                let val = self.read(addr - 1);
                self.write_register(d, val);
                self.write_register_pair(Y_REG, addr - 1);
            }

            0b1100 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr);
                self.write_register(d, val);
            }
            0b1101 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(X_REG, addr + 1);
            }
            0b1110 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr-1);
                self.write_register(d, val);
                self.write_register_pair(X_REG, addr - 1);
            }
            _ => panic!("Invalid LD instruction")
        }

        self.pc += 1;

        2
    }

    pub fn instr_ldd(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let q =
            bit_field_combined(opcode, &[13..=13, 11..=10, 2..=0]);

        let addr_base = if opcode.bit(3) {
            self.read_register_pair(Y_REG)
        } else {
            self.read_register_pair(Z_REG)
        };

        let addr = addr_base.wrapping_add(q);
        let val = self.read(addr);
        self.write_register(d, val);
        self.pc += 1;

        2
    }

    pub fn instr_lds(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);

        let addr = self.read_at_pc_offset(1);

        let val = self.read(addr);
        self.write_register(d, val);
        self.pc += 2;

        2
    }

    pub fn instr_st(&mut self, opcode: u16) -> u8 {
        

        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);
        
        match bit_field_combined(opcode, &[0..=3]) {
            0b0001 => {
                let addr = self.read_register_pair(Z_REG);
                self.write(addr, val);
                self.write_register_pair(Z_REG, addr+1);
            }
            0b0010 => {
                let addr = self.read_register_pair(Z_REG);
                self.write(addr - 1, val);
                self.write_register_pair(Z_REG, addr-1);
            }

            0b1001 => {
                let addr = self.read_register_pair(Y_REG);
                self.write(addr, val);
                self.write_register_pair(Y_REG, addr+1);
            }
            0b1010 => {
                let addr = self.read_register_pair(Y_REG);
                self.write(addr - 1, val);
                self.write_register_pair(Y_REG, addr-1);
            }

            0b1100 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr, val);
            }
            0b1101 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr, val);
                self.write_register_pair(X_REG, addr+1);
            }
            0b1110 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr - 1, val);
                self.write_register_pair(X_REG, addr-1);
            }
            _ => panic!("Invalid LD instruction")
        }

        self.pc += 1;

        2
    }

    pub fn instr_std(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let q =
            bit_field_combined(opcode, &[13..=13, 11..=10, 2..=0]);

        let addr_base = if opcode.bit(3) {
            self.read_register_pair(Y_REG)
        } else {
            self.read_register_pair(Z_REG)
        };

        let addr = addr_base.wrapping_add(q);
        let val = self.read_register(d);
        self.write(addr, val);
        self.pc += 1;

        2
    }

    pub fn instr_sts(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);

        let addr = self.read_at_pc_offset(1);

        let val = self.read_register(d);
        self.write(addr, val);
        self.pc += 2;

        2
    }

    pub fn instr_lpm(&mut self, opcode: u16) -> u8 {
        let d = if opcode == 0x95C8 {0} else {get_d_field(opcode, 5)};

        let addr = self.read_register_pair(Z_REG);

        let val = self.read_flash(addr as u32 >> 1);

        let val = if addr.bit(0) {(val >> 8) as u8} else {val as u8};
        self.write_register(d, val);
        if opcode & 0x000F == 0x5 {
            self.write_register_pair(Z_REG, addr + 1);
        }
        self.pc += 1;

        3
    }

    pub fn instr_elpm(&mut self, opcode: u16) -> u8 {
        let d = if opcode == 0x95D8 {0} else {get_d_field(opcode, 5)};

        let z = self.read_register_pair(Z_REG);
        let addr =self.rampz_address(z);

        let val = self.read_flash(addr as u32 >> 1);

        let val = if addr.bit(0) {(val >> 8) as u8} else {val as u8};
        self.write_register(d, val);
        if opcode & 0x000F == 0x7 {
            self.write_register_pair(Z_REG, (addr + 1) as u16);
            self.rampz = ((addr + 1) >> 16) as u8;
        }
        self.pc += 1;

        3
    }

    pub fn instr_spm(&mut self, _opcode: u16) -> u8 {
        todo!()
    }

    pub fn instr_in(&mut self, opcode: u16) -> u8 {
        let io = get_io6(opcode);
        let d = get_d_field(opcode, 5);
        let val = self.read_io(io);
        self.write_register(d, val);
        self.pc += 1;
        1
    }

    pub fn instr_out(&mut self, opcode: u16) -> u8 {
        let io = get_io6(opcode);
        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);
        self.write_io(io, val);
        self.pc += 1;
        1
    }

    pub fn instr_push(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);
        self.write_at_sp_offset(0, val);
        self.sp -= 1;
        self.pc += 1;
        2
    }

    pub fn instr_pop(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        self.sp += 1;
        let val = self.read_at_sp_offset(0);
        self.write_register(d, val);
        self.pc += 1;
        2
    }
}