use bitfield::Bit;

use crate::mcu::avr::{mcu_model::McuModel, bit_helpers::{bit_field_combined, get_rd_fields, is_two_word, get_d_field, get_io5}, io_controller::IoControllerTrait};

use super::{Mcu};

const Z_REG: u16 = 30;

impl<M, Io> Mcu<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    pub fn instr_rjmp(&mut self, opcode: u16) -> u8 {
        let k = opcode & 0x0FFF;

        if k.bit(11) {
            let k = k ^ 0x0FFF + 1;
            self.pc = self.pc - (k as u32) + 1;
        } else {
            self.pc = self.pc + (k as u32) + 1;
        }
        2
    }

    pub fn instr_ijmp(&mut self, _opcode: u16) -> u8 {
        self.pc = self.read_register_pair(Z_REG) as u32;
        2
    }

    pub fn instr_eijmp(&mut self, _opcode: u16) -> u8 {
        let z = self.read_register_pair(Z_REG);
        self.pc = self.eind_address(z);
        2
    }

    pub fn instr_jmp(&mut self, opcode: u16) -> u8 {
        let addr = 
            (bit_field_combined(opcode, &[8..=4, 0..=0]) as u32) << 16 |
            self.read_at_pc_offset(1) as u32;
        self.pc = addr;
        3
    }

    fn push_pc(&mut self) {
        self.pc += 1;
        self.write_at_sp_offset(0, (self.pc) as u8);
        self.write_at_sp_offset(-1, (self.pc >> 8) as u8);
        self.write_at_sp_offset(-2, (self.pc >> 16) as u8);
        self.sp -= 3;
    }

    pub fn instr_rcall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_rjmp(opcode) + 2
    }

    pub fn instr_icall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_ijmp(opcode) + 2
    }

    pub fn instr_eicall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_eijmp(opcode) + 2
    }

    pub fn instr_call(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_jmp(opcode) + 2
    }

    pub fn instr_ret(&mut self, _opcode: u16) -> u8 {
        
        let v1 = self.read_at_sp_offset(1) as u32;
        let v2 = self.read_at_sp_offset(2) as u32;
        let v3 = self.read_at_sp_offset(3) as u32;
        self.sp += 3;
        
        self.pc = v1 << 16 | v2 << 8 | v3;

        5
    }

    pub fn instr_reti(&mut self, opcode: u16) -> u8 {
        self.sreg.set_i(true);
        self.instr_ret(opcode)
    }

    fn skip_if(&mut self, cond: bool) -> u8 {
        if cond {
            if is_two_word(self.read_at_pc_offset(1)) {
                self.pc += 3;
                3
            } else {
                self.pc += 2;
                2
            }
        } else {
            self.pc += 1;
            1
        }
    }

    fn jump_if(&mut self, cond: bool, k: u16) -> u8 {
        self.pc += 1;
        if cond {
            if k.bit(7) {
                let k = k ^ 0x007F + 1; // Negation
                self.pc = self.pc - (k as u32);
            } else {
                self.pc = self.pc + (k as u32);
            }
            2
        } else {
            1
        }
    }


    pub fn instr_cpse(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        self.skip_if(rr == rd)
    }

    pub fn instr_sbrc(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let b = opcode & 0x0007;
        let rd = self.read_register(d);

        self.skip_if(!rd.bit(b as usize))
    }

    pub fn instr_sbrs(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let b = opcode & 0x0007;
        let rd = self.read_register(d);

        self.skip_if(rd.bit(b as usize))
    }

    pub fn instr_sbic(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let val = self.read_io(io);

        self.skip_if(!val.bit(b as usize))
    }

    pub fn instr_sbis(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let val = self.read_io(io);

        self.skip_if(val.bit(b as usize))
    }

    pub fn instr_brbc(&mut self, opcode: u16) -> u8 {
        let k = bit_field_combined(opcode, &[9..=3]);
        let s = bit_field_combined(opcode, &[2..=0]) as usize;

        self.jump_if(!self.sreg.bit(s), k)
    }

    pub fn instr_brbs(&mut self, opcode: u16) -> u8 {
        let k = bit_field_combined(opcode, &[9..=3]);
        let s = bit_field_combined(opcode, &[2..=0]) as usize;

        self.jump_if(self.sreg.bit(s), k)
    }
}