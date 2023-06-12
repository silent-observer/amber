mod add_sub;
mod logical;
mod mul;
mod transfer;
mod branches;
mod bitops;
mod memory_controller;
pub mod hex;

use std::marker::PhantomData;

use bitfield::Bit;

use crate::pins::PinState;
use crate::vcd::{VcdFiller, VcdTreeModule, VcdModuleBuilder};

use super::{regfile::RegisterFile, mcu_model::McuModel, io_controller::{IoController, IoControllerTrait}, sreg::StatusRegister, bit_helpers::bit_field_combined};

/// Internal AVR MCU structure.
pub struct Mcu<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    reg_file: RegisterFile,
    pub io: Io,
    sram: Vec<u8>,
    flash: Vec<u16>,

    pc: u32,
    sp: u16,
    sreg: StatusRegister,

    rampz: u8,
    eind: u8,

    model: PhantomData<M>,
}

const SRAM_SIZE: usize = 8192;
const SRAM_END: u16 = 0x200 + SRAM_SIZE as u16 - 1;

impl<M> Default for Mcu<M, IoController<M>>
where
    M: McuModel + 'static,
{
    /// Initializes MCU with a default [IoController].
    fn default() -> Mcu<M, IoController<M>> {
        let io = IoController::new();
        Mcu::new(io)
    }
}

impl<M, Io> Mcu<M, Io> 
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    /// Creates a new MCU with a specified [IoControllerTrait].
    pub fn new(io: Io) -> Mcu<M, Io> {
        Mcu { 
            reg_file: RegisterFile::new(),
            io,
            sram: vec![0; SRAM_SIZE],
            flash: vec![0; M::flash_size()],

            pc: 0,
            sp: 0,
            rampz: 0,
            eind: 0,
            sreg: StatusRegister(0),

            model: PhantomData
        }
    }

    /// Executes one instruction at PC address and returns number of cycles.
    pub fn step(&mut self) -> u8 {
        if self.io.has_interrupt() && self.sreg.i() {
            if let Some(addr) = self.io.get_interrupt_address() {
                return self.execute_interrupt(addr)
            }
        }

        let opcode: u16 = self.read_at_pc_offset(0);
        self.execute(opcode)
    }

    /// Executes an opcode and returns number of cycles.
    fn execute(&mut self, opcode: u16) -> u8 {
        let head = (opcode >> 8) as u8;
        match head {
            0x00 => {
                if opcode == 0x0000 {
                    self.pc += 1; // NOP
                    1
                } else {
                    panic!("Reserved")
                }
            }
            0x01        => self.instr_movw(opcode),
            0x02        => self.instr_muls(opcode),
            0x03 => {
                let bits = bit_field_combined(opcode, &[7..=7, 3..=3]);
                match bits {
                    0b00 => self.instr_mulsu(opcode),
                    0b01 => self.instr_fmul(opcode),
                    0b10 => self.instr_fmuls(opcode),
                    0b11 => self.instr_fmulsu(opcode),
                    _ => panic!("2-bit field impossible value"),
                }
            },
            0x04..=0x07 => self.instr_cpc(opcode),
            0x08..=0x0B => self.instr_sbc(opcode),
            0x0C..=0x0F => self.instr_add(opcode),
            0x10..=0x13 => self.instr_cpse(opcode),
            0x14..=0x17 => self.instr_cp(opcode),
            0x18..=0x1B => self.instr_sub(opcode),
            0x1C..=0x1F => self.instr_adc(opcode),

            0x20..=0x23 => self.instr_and(opcode),
            0x24..=0x27 => self.instr_eor(opcode),
            0x28..=0x2B => self.instr_or(opcode),
            0x2C..=0x2F => self.instr_mov(opcode),

            0x30..=0x3F => self.instr_cpi(opcode),
            0x40..=0x4F => self.instr_sbci(opcode),
            0x50..=0x5F => self.instr_subi(opcode),
            0x60..=0x6F => self.instr_ori(opcode),
            0x70..=0x7F => self.instr_andi(opcode),

            0x80..=0x8F |
            0xA0..=0xAF => if head.bit(1) {
                self.instr_std(opcode)
            } else {
                self.instr_ldd(opcode)
            },

            0x90 | 0x91 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_lds(opcode),
                    0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => self.instr_ld(opcode),
                    0x4 | 0x5 => self.instr_lpm(opcode),
                    0x6 | 0x7 => self.instr_elpm(opcode),
                    0xF => self.instr_pop(opcode),
                    0x3 | 0x8 | 0xB => panic!("Reserved"),
                    _ => panic!("Impossible for 4-bit value"),
                }
            },

            0x92 | 0x93 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_sts(opcode),
                    0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => self.instr_st(opcode),
                    0xF => self.instr_push(opcode),
                    0x3..=0x8 | 0xB => panic!("Reserved"),
                    _ => panic!("Impossible for 4-bit value"),
                }
            },

            0x94 | 0x95 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_com(opcode),
                    0x1 => self.instr_neg(opcode),
                    0x2 => self.instr_swap(opcode),
                    0x3 => self.instr_inc(opcode),
                    0x4 => panic!("Reserved"),
                    0x5 => self.instr_asr(opcode),
                    0x6 => self.instr_lsr(opcode),
                    0x7 => self.instr_ror(opcode),

                    0x8 => if head == 0x94 {
                        if opcode.bit(7) {
                            self.instr_bclr(opcode)
                        } else {
                            self.instr_bset(opcode)
                        }
                    } else {
                        match opcode {
                            0x9508 => self.instr_ret(opcode),
                            0x9518 => self.instr_reti(opcode),
                            0x9588 => todo!(),
                            0x9598 => todo!(),
                            0x95A8 => todo!(),
                            0x95C8 => self.instr_lpm(opcode),
                            0x95D8 => self.instr_elpm(opcode),
                            0x95E8 => self.instr_spm(opcode),
                            _ => panic!("Reserved"),
                        }
                    }
                    0x9 => match opcode {
                        0x9409 => self.instr_ijmp(opcode),
                        0x9419 => self.instr_eijmp(opcode),
                        0x9509 => self.instr_icall(opcode),
                        0x9519 => self.instr_eicall(opcode),
                        _ => panic!("Reserved"),
                    }
                    0xA => self.instr_dec(opcode),
                    0xB => panic!("Reserved"),
                    0xC | 0xD => self.instr_jmp(opcode),
                    0xE | 0xF => self.instr_call(opcode),
                    _ => panic!("Impossible for 4-bit value"),
                }
            },

            0x96        => self.instr_adiw(opcode),
            0x97        => self.instr_sbiw(opcode),
            0x98        => self.instr_cbi(opcode),
            0x99        => self.instr_sbic(opcode),
            0x9A        => self.instr_sbi(opcode),
            0x9B        => self.instr_sbis(opcode),
            0x9C..=0x9F => self.instr_mul(opcode),

            0xB0..=0xB7 => self.instr_in(opcode),
            0xB8..=0xBF => self.instr_out(opcode),

            0xC0..=0xCF => self.instr_rjmp(opcode),
            0xD0..=0xDF => self.instr_rcall(opcode),
            0xE0..=0xEF => self.instr_ldi(opcode),

            0xF0..=0xF3 => self.instr_brbs(opcode),
            0xF4..=0xF7 => self.instr_brbc(opcode),
            0xF8..=0xF9 => self.instr_bld(opcode),
            0xFA..=0xFB => self.instr_bst(opcode),
            0xFC..=0xFD => self.instr_sbrc(opcode),
            0xFE..=0xFF => self.instr_sbrs(opcode),
        }
    }

    /// Loads flash memory from a slice
    pub fn load_flash(&mut self, data: &[u16]) {
        for (addr, &val) in data.iter().enumerate() {
            self.write_flash(addr as u32, val);
        }
    }
}

/// An implementation for [VcdFiller].
/// 
/// [Mcu] is a module, containing the following signals and submodules:
/// 
/// ### Signals
/// - `clk` - CPU clock.
/// - `pc[31:0]` - Program counter (PC) register.
/// ### Submodules
/// - `regs` - Register file.
/// - `sreg` - Status register.
impl<M, Io> VcdFiller for Mcu<M, Io> 
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    const IS_SIGNAL: bool = false;

    fn init_vcd_module(&self, builder: &mut VcdModuleBuilder) {
        builder.add_signal("clk", 1, PinState::Low);
        builder.add_signal("pc", 32, PinState::Low);
        builder.add_node("regs", &self.reg_file);
        builder.add_node("sreg", &self.sreg);
        builder.add_node("timer1", self.io.timer1());
        builder.add_node("timer3", self.io.timer3());
        builder.add_node("timer4", self.io.timer4());
        builder.add_node("timer5", self.io.timer5());
    }

    fn fill_module(&self, module: &mut VcdTreeModule) -> bool {
        let mut r = module.update_subsignal(0, self.io.clock_pin());
        r |= module.update_subsignal(1, self.pc);
        r |= module.update_child(2, &self.reg_file);
        r |= module.update_child(3, &self.sreg);
        r |= module.update_child(4, self.io.timer1());
        r |= module.update_child(5, self.io.timer3());
        r |= module.update_child(6, self.io.timer4());
        r |= module.update_child(7, self.io.timer5());
        r
    }
}

#[cfg(test)]
mod test_helper {
    use crate::components::avr::{mcu_model::McuModel, sreg::test_helper::assert_sreg, io_controller::IoControllerTrait};

    use super::Mcu;

    impl<M, Io> Mcu<M, Io> 
    where
        M: McuModel + 'static,
        Io: IoControllerTrait,
    {
        /// Helper test function, for executing an instruction and checking the correct [StatusRegister] change.
        pub fn execute_and_assert_sreg(&mut self, opcode: u16, sreg_mask: &'static str) {
            let sreg_initial = self.sreg;
            self.execute(opcode);
            assert_sreg(&self.sreg, &sreg_initial, sreg_mask);
        }
    }
}