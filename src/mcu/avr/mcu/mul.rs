use bitfield::Bit;

use crate::mcu::avr::{mcu_model::McuModel, bit_helpers::get_rd_fields, io_controller::IoControllerTrait};

use super::{Mcu};

impl<M, Io> Mcu<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    pub fn instr_mul(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (rd as u16) * (rr as u16);

        self.write_register_pair(0, result);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_muls(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 4);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = ((rd as i16) * (rr as i16)) as u16;

        self.write_register_pair(0, result);
        self.sreg.set_c((result).bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_mulsu(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (((rd as u16) as i16) * (rr as i16)) as u16;

        self.write_register_pair(0, result);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmul(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (rd as u16) * (rr as u16);

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmuls(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = ((rd as i16) * (rr as i16)) as u16;

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmulsu(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (((rd as u16) as i16) * (rr as i16)) as u16;

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }
}