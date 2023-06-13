use bitfield::Bit;

use crate::pins::{PinId, PinState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UartMode {
    Async = 0,
    Sync = 1,
    MasterSpi = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParityMode {
    Disabled = 0,
    Even = 2,
    Odd = 3,
}

pub struct Uart {
    ubbr: u16,
    counter: u16,
    xck_pin: PinId,
    tx_pin: PinId,
    prescaler: u8,

    u2x: bool,
    mpcm: bool,
    reciever_enabled: bool,
    transmitter_enabled: bool,
    char_size: u8,
    parity_pos: u8,
    stop_bit_pos: u8,
    mode: UartMode,
    parity: ParityMode,
    stop_two_bit: bool,
    polarity_inverted: bool,

    data_register_empty: bool,

    xck_ddr: bool,

    transmitter_udr: u16,
    transmitter_shift: u16,
    transmitter_pos: u8,
    transmitter_parity: bool,
}

impl Uart {
    pub fn new(xck_pin: PinId, tx_pin: PinId) -> Uart {
        Uart { 
            ubbr: 0,
            counter: 0,
            xck_pin,
            tx_pin,
            prescaler: 0,

            u2x: false,
            mpcm: false,
            reciever_enabled: false,
            transmitter_enabled: false,
            char_size: 8,
            parity_pos: 0,
            stop_bit_pos: 9,

            mode: UartMode::Async,
            parity: ParityMode::Disabled,
            stop_two_bit: false,
            polarity_inverted: false,
            data_register_empty: true,

            xck_ddr: true,

            transmitter_udr: 0,
            transmitter_shift: 0,
            transmitter_parity: false,
            transmitter_pos: 0,
        }
    }

    pub fn tick(&mut self, output_changes: &mut Vec<(PinId, PinState)>, interrupt: &mut bool) {
        if !self.transmitter_enabled && !self.reciever_enabled {
            return;
        }
        if self.counter == 0 {
            self.counter = self.ubbr;
            self.prescaler = (self.prescaler + 1) % 16;
            if self.mode == UartMode::Sync && self.xck_ddr {
                let xck = self.prescaler.bit(0);
                output_changes.push((self.xck_pin, PinState::from_bool(xck)));
                if self.transmitter_enabled && xck != self.polarity_inverted {
                    self.tick_transmitter(output_changes, interrupt);
                }
            }
        } else {
            self.counter = self.counter.wrapping_sub(1);
        }
    }

    fn tick_transmitter(&mut self, output_changes: &mut Vec<(PinId, PinState)>, interrupt: &mut bool) {
        if self.transmitter_pos == 0 {
            if !self.data_register_empty {
                self.transmitter_shift = self.transmitter_udr;
                self.data_register_empty = true;
                self.transmitter_pos = 1;
                self.transmitter_parity = self.parity == ParityMode::Odd;
                output_changes.push((self.tx_pin, PinState::Low)); // Start bit
            } else {
                return;
            }
        } else {
            if self.transmitter_pos <= self.char_size {
                let bit = self.transmitter_shift.bit(0);
                output_changes.push((self.tx_pin, PinState::from_bool(bit)));
                self.transmitter_shift >>= 1;
                self.transmitter_pos += 1;
                self.transmitter_parity ^= bit;
            } else if self.parity != ParityMode::Disabled && self.transmitter_pos == self.parity_pos {
                output_changes.push((self.tx_pin, PinState::from_bool(self.transmitter_parity)));
                self.transmitter_pos += 1;
            } else {
                output_changes.push((self.tx_pin, PinState::High));
                if self.stop_two_bit && self.transmitter_pos == self.stop_bit_pos {
                    self.transmitter_pos += 1;
                } else {
                    self.transmitter_pos = 0;
                }
            }
        }
    }

    #[inline]
    pub fn read_udr(&self) -> u8 {
        todo!()
    }

    #[inline]
    pub fn write_udr(&mut self, val: u8) {
        // TODO: check ready
        self.transmitter_udr = self.transmitter_udr & 0x0100 | val as u16;
        self.data_register_empty = false;
    }

    #[inline]
    pub fn read_ucsra(&self) -> u8 {
        (self.data_register_empty as u8) << 5 |
        (self.u2x as u8) << 1 |
        (self.mpcm as u8)
    }

    #[inline]
    pub fn write_ucsra(&mut self, val: u8) {
        self.u2x = val.bit(1);
        self.mpcm = val.bit(0);
    }

    #[inline]
    pub fn read_ucsrb(&self) -> u8 {
        // TODO: interrupts
        (self.reciever_enabled as u8) << 4 |
        (self.transmitter_enabled as u8) << 3 |
        ((self.char_size == 9) as u8) << 2 |
        (self.transmitter_udr.bit(8) as u8)
    }

    #[inline]
    pub fn write_ucsrb(&mut self, val: u8, output_changes: &mut Vec<(PinId, PinState)>) {
        self.transmitter_udr = self.transmitter_udr & 0xFF | (val as u16 & 0x1) << 8;
        if val.bit(2) {
            self.char_size = 9
        } else if self.char_size == 9 {
            self.char_size = 8;
        }
        self.reciever_enabled = val.bit(4);
        self.transmitter_enabled = val.bit(3);
        if self.transmitter_enabled {
            output_changes.push((self.tx_pin, PinState::High));
        }

        self.parity_pos = if self.parity == ParityMode::Disabled {0} else {self.char_size + 1};
        self.stop_bit_pos = if self.parity == ParityMode::Disabled {
            self.char_size + 1
        } else {
            self.char_size + 2
        };
    }

    #[inline]
    pub fn read_ucsrc(&self) -> u8 {
        let cs: u8 = match self.char_size {
            5 => 0,
            6 => 1,
            7 => 2,
            8 | 9 => 3,
            _ => 0
        };
        // TODO: interrupts
        (self.mode as u8) << 6 |
        (self.parity as u8) << 4 |
        (self.stop_two_bit as u8) << 3 |
        cs << 1 | (self.polarity_inverted as u8)
    }

    #[inline]
    pub fn write_ucsrc(&mut self, val: u8) {
        match val >> 6 {
            0b00 => self.mode = UartMode::Async,
            0b01 => self.mode = UartMode::Sync,
            0b11 => self.mode = UartMode::MasterSpi,
            _ => {}
        }
        match (val >> 4) & 0x3 {
            0b00 => self.parity = ParityMode::Disabled,
            0b10 => self.parity = ParityMode::Even,
            0b11 => self.parity = ParityMode::Odd,
            _ => {}
        }
        self.stop_two_bit = val.bit(3);
        match (val >> 1) & 0x3 {
            0b00 => self.char_size = 5,
            0b01 => self.char_size = 6,
            0b10 => self.char_size = 7,
            0b11 => if self.char_size != 9 {self.char_size = 8},
            _ => {}
        }
        self.polarity_inverted = val.bit(0);
        self.parity_pos = if self.parity == ParityMode::Disabled {0} else {self.char_size + 1};
        self.stop_bit_pos = if self.parity == ParityMode::Disabled {
            self.char_size + 1
        } else {
            self.char_size + 2
        };
    }

    #[inline]
    pub fn read_ubrrl(&self) -> u8 {
        self.ubbr as u8
    }

    #[inline]
    pub fn write_ubrrl(&mut self, val: u8) {
        self.ubbr = self.ubbr & 0x0F00 | val as u16;
    }

    #[inline]
    pub fn read_ubrrh(&self) -> u8 {
        (self.ubbr >> 8) as u8
    }

    #[inline]
    pub fn write_ubrrh(&mut self, val: u8) {
        self.ubbr = self.ubbr & 0x00FF | ((val as u16) & 0xF) << 8;
    }
}