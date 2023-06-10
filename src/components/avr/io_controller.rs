mod gpio;

use std::collections::HashMap;
use std::marker::PhantomData;
use mockall::*;

use crate::pins::{PinId, PinState};

use self::gpio::GpioPort;

use super::mcu_model::McuModel;

/// An AVR IO controller (can be mocked)
#[automock]
pub trait IoControllerTrait: Send {
    /// Reads internal IO port
    fn read_internal_u8(&self, id: u8) -> u8;
    /// Reads external IO port
    fn read_external_u8(&self, addr: u16) -> u8;
    /// Writes to internal IO port
    fn write_internal_u8(&mut self, id: u8, val: u8);
    /// Writes to external IO port
    fn write_external_u8(&mut self, addr: u16, val: u8);

    /// Returns `true` if is on rising edge of the clock
    fn is_clock_rising(&self) -> bool;
    // Get current clock pin value
    fn clock_pin(&self) -> PinState;

    /// Get number of pins
    fn pin_count() -> usize;
    /// Get pin names
    fn pin_name(pin: PinId) -> String;
    /// Set input pin value
    fn set_pin(&mut self, pin: PinId, state: PinState);
    /// Get output pin changes (by filling a [HashMap])
    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId, PinState>);
    /// Advance the simulation through one step.
    /// 
    /// After this step all the pin value changes must be accounted for.
    fn advance(&mut self);
}

/// Main implementation for [IoControllerTrait]
pub struct IoController<M: McuModel> {
    model: PhantomData<M>,
    clock_pin: PinState,
    is_clock_rising: bool,

    gpio: [GpioPort; 11]
}

impl<M: McuModel + 'static> IoController<M>{
    pub fn new() -> IoController<M> {
        IoController { 
            model: PhantomData,
            clock_pin: PinState::Z,
            is_clock_rising: false,
            gpio: std::array::from_fn(|_| GpioPort::new()),
        }
    }
}

impl<M: McuModel> IoController<M> {
    fn pin_number(pin: &'static str) -> PinId {
        let bytes: Vec<u8> = pin.bytes().collect();
        match bytes[..] {
            [b'C', b'L', b'K'] => 0,

            [b'P', b'A', b'0'] => 1,
            [b'P', b'A', b'1'] => 2,
            [b'P', b'A', b'2'] => 3,
            [b'P', b'A', b'3'] => 4,
            [b'P', b'A', b'4'] => 5,
            [b'P', b'A', b'5'] => 6,
            [b'P', b'A', b'6'] => 7,
            [b'P', b'A', b'7'] => 8,

            [b'P', b'B', b'0'] => 1*8 + 1,
            [b'P', b'B', b'1'] => 1*8 + 2,
            [b'P', b'B', b'2'] => 1*8 + 3,
            [b'P', b'B', b'3'] => 1*8 + 4,
            [b'P', b'B', b'4'] => 1*8 + 5,
            [b'P', b'B', b'5'] => 1*8 + 6,
            [b'P', b'B', b'6'] => 1*8 + 7,
            [b'P', b'B', b'7'] => 1*8 + 8,

            [b'P', b'C', b'0'] => 2*8 + 1,
            [b'P', b'C', b'1'] => 2*8 + 2,
            [b'P', b'C', b'2'] => 2*8 + 3,
            [b'P', b'C', b'3'] => 2*8 + 4,
            [b'P', b'C', b'4'] => 2*8 + 5,
            [b'P', b'C', b'5'] => 2*8 + 6,
            [b'P', b'C', b'6'] => 2*8 + 7,
            [b'P', b'C', b'7'] => 2*8 + 8,

            [b'P', b'D', b'0'] => 3*8 + 1,
            [b'P', b'D', b'1'] => 3*8 + 2,
            [b'P', b'D', b'2'] => 3*8 + 3,
            [b'P', b'D', b'3'] => 3*8 + 4,
            [b'P', b'D', b'4'] => 3*8 + 5,
            [b'P', b'D', b'5'] => 3*8 + 6,
            [b'P', b'D', b'6'] => 3*8 + 7,
            [b'P', b'D', b'7'] => 3*8 + 8,

            [b'P', b'E', b'0'] => 4*8 + 1,
            [b'P', b'E', b'1'] => 4*8 + 2,
            [b'P', b'E', b'2'] => 4*8 + 3,
            [b'P', b'E', b'3'] => 4*8 + 4,
            [b'P', b'E', b'4'] => 4*8 + 5,
            [b'P', b'E', b'5'] => 4*8 + 6,
            [b'P', b'E', b'6'] => 4*8 + 7,
            [b'P', b'E', b'7'] => 4*8 + 8,

            [b'P', b'F', b'0'] => 5*8 + 1,
            [b'P', b'F', b'1'] => 5*8 + 2,
            [b'P', b'F', b'2'] => 5*8 + 3,
            [b'P', b'F', b'3'] => 5*8 + 4,
            [b'P', b'F', b'4'] => 5*8 + 5,
            [b'P', b'F', b'5'] => 5*8 + 6,
            [b'P', b'F', b'6'] => 5*8 + 7,
            [b'P', b'F', b'7'] => 5*8 + 8,

            [b'P', b'G', b'0'] => 6*8 + 1,
            [b'P', b'G', b'1'] => 6*8 + 2,
            [b'P', b'G', b'2'] => 6*8 + 3,
            [b'P', b'G', b'3'] => 6*8 + 4,
            [b'P', b'G', b'4'] => 6*8 + 5,
            [b'P', b'G', b'5'] => 6*8 + 6,

            [b'P', b'H', b'0'] => 6*8 + 6 + 1,
            [b'P', b'H', b'1'] => 6*8 + 6 + 2,
            [b'P', b'H', b'2'] => 6*8 + 6 + 3,
            [b'P', b'H', b'3'] => 6*8 + 6 + 4,
            [b'P', b'H', b'4'] => 6*8 + 6 + 5,
            [b'P', b'H', b'5'] => 6*8 + 6 + 6,
            [b'P', b'H', b'6'] => 6*8 + 6 + 7,
            [b'P', b'H', b'7'] => 6*8 + 6 + 8,

            [b'P', b'J', b'0'] => 7*8 + 6 + 1,
            [b'P', b'J', b'1'] => 7*8 + 6 + 2,
            [b'P', b'J', b'2'] => 7*8 + 6 + 3,
            [b'P', b'J', b'3'] => 7*8 + 6 + 4,
            [b'P', b'J', b'4'] => 7*8 + 6 + 5,
            [b'P', b'J', b'5'] => 7*8 + 6 + 6,
            [b'P', b'J', b'6'] => 7*8 + 6 + 7,
            [b'P', b'J', b'7'] => 7*8 + 6 + 8,

            [b'P', b'K', b'0'] => 8*8 + 6 + 1,
            [b'P', b'K', b'1'] => 8*8 + 6 + 2,
            [b'P', b'K', b'2'] => 8*8 + 6 + 3,
            [b'P', b'K', b'3'] => 8*8 + 6 + 4,
            [b'P', b'K', b'4'] => 8*8 + 6 + 5,
            [b'P', b'K', b'5'] => 8*8 + 6 + 6,
            [b'P', b'K', b'6'] => 8*8 + 6 + 7,
            [b'P', b'K', b'7'] => 8*8 + 6 + 8,

            [b'P', b'L', b'0'] => 9*8 + 6 + 1,
            [b'P', b'L', b'1'] => 9*8 + 6 + 2,
            [b'P', b'L', b'2'] => 9*8 + 6 + 3,
            [b'P', b'L', b'3'] => 9*8 + 6 + 4,
            [b'P', b'L', b'4'] => 9*8 + 6 + 5,
            [b'P', b'L', b'5'] => 9*8 + 6 + 6,
            [b'P', b'L', b'6'] => 9*8 + 6 + 7,
            [b'P', b'L', b'7'] => 9*8 + 6 + 8,

            _ => panic!("Invalid pin number")
        }
    }
}

impl<M: McuModel + 'static> IoControllerTrait for IoController<M> {
    fn read_internal_u8(&self, id: u8) -> u8 {
        match id {
            0x00 => self.gpio[0].read_pin(), // PINA
            0x01 => self.gpio[0].read_ddr(), // DDRA
            0x02 => self.gpio[0].read_port(), // PORTA

            0x03 => self.gpio[1].read_pin(), // PINB
            0x04 => self.gpio[1].read_ddr(), // DDRB
            0x05 => self.gpio[1].read_port(), // PORTB

            0x06 => self.gpio[2].read_pin(), // PINC
            0x07 => self.gpio[2].read_ddr(), // DDRC
            0x08 => self.gpio[2].read_port(), // PORTC

            0x09 => self.gpio[3].read_pin(), // PIND
            0x0A => self.gpio[3].read_ddr(), // DDRD
            0x0B => self.gpio[3].read_port(), // PORTD

            0x0C => self.gpio[4].read_pin(), // PINE
            0x0D => self.gpio[4].read_ddr(), // DDRE
            0x0E => self.gpio[4].read_port(), // PORTE

            0x0F => self.gpio[5].read_pin(), // PINF
            0x10 => self.gpio[5].read_ddr(), // DDRF
            0x11 => self.gpio[5].read_port(), // PORTF

            0x12 => self.gpio[6].read_pin(), // PING
            0x13 => self.gpio[6].read_ddr(), // DDRG
            0x14 => self.gpio[6].read_port(), // PORTG
            _ => 0
        }
    }

    fn read_external_u8(&self, addr: u16) -> u8 {
        match addr {
            0x100 => self.gpio[7].read_pin(), // PINH
            0x101 => self.gpio[7].read_ddr(), // DDRH
            0x102 => self.gpio[7].read_port(), // PORTH

            0x103 => self.gpio[8].read_pin(), // PINJ
            0x104 => self.gpio[8].read_ddr(), // DDRJ
            0x105 => self.gpio[8].read_port(), // PORTJ

            0x106 => self.gpio[9].read_pin(), // PINK
            0x107 => self.gpio[9].read_ddr(), // DDRK
            0x108 => self.gpio[9].read_port(), // PORTK

            0x109 => self.gpio[10].read_pin(), // PINL
            0x10A => self.gpio[10].read_ddr(), // DDRL
            0x10B => self.gpio[10].read_port(), // PORTL
            _ => 0
        }
    }

    fn write_internal_u8(&mut self, id: u8, val: u8) {
        match id {
            0x00 => self.gpio[0].write_pin(val), // PINA
            0x01 => self.gpio[0].write_ddr(val), // DDRA
            0x02 => self.gpio[0].write_port(val), // PORTA

            0x03 => self.gpio[1].write_pin(val), // PINB
            0x04 => self.gpio[1].write_ddr(val), // DDRB
            0x05 => self.gpio[1].write_port(val), // PORTB

            0x06 => self.gpio[2].write_pin(val), // PINC
            0x07 => self.gpio[2].write_ddr(val), // DDRC
            0x08 => self.gpio[2].write_port(val), // PORTC

            0x09 => self.gpio[3].write_pin(val), // PIND
            0x0A => self.gpio[3].write_ddr(val), // DDRD
            0x0B => self.gpio[3].write_port(val), // PORTD

            0x0C => self.gpio[4].write_pin(val), // PINE
            0x0D => self.gpio[4].write_ddr(val), // DDRE
            0x0E => self.gpio[4].write_port(val), // PORTE

            0x0F => self.gpio[5].write_pin(val), // PINF
            0x10 => self.gpio[5].write_ddr(val), // DDRF
            0x11 => self.gpio[5].write_port(val), // PORTF

            0x12 => self.gpio[6].write_pin(val), // PING
            0x13 => self.gpio[6].write_ddr(val), // DDRG
            0x14 => self.gpio[6].write_port(val), // PORTG
            _ => {}
        }
    }

    fn write_external_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x100 => self.gpio[7].write_pin(val), // PINH
            0x101 => self.gpio[7].write_ddr(val), // DDRH
            0x102 => self.gpio[7].write_port(val), // PORTH

            0x103 => self.gpio[8].write_pin(val), // PINJ
            0x104 => self.gpio[8].write_ddr(val), // DDRJ
            0x105 => self.gpio[8].write_port(val), // PORTJ

            0x106 => self.gpio[9].write_pin(val), // PINK
            0x107 => self.gpio[9].write_ddr(val), // DDRK
            0x108 => self.gpio[9].write_port(val), // PORTK

            0x109 => self.gpio[10].write_pin(val), // PINL
            0x10A => self.gpio[10].write_ddr(val), // DDRL
            0x10B => self.gpio[10].write_port(val), // PORTL
            _ => {}
        }
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        if pin == Self::pin_number("CLK") {
            self.is_clock_rising = self.clock_pin == PinState::Low && state == PinState::High;
            self.clock_pin = state;
        }
        else if pin < 1 + 86 {
            let (gpio_bank, gpio_index) = match pin {
                1 ..=8  => (0, pin - 1),
                9 ..=16 => (1, pin - 9),
                17..=24 => (2, pin - 17),
                25..=32 => (3, pin - 25),
                33..=40 => (4, pin - 33),
                41..=48 => (5, pin - 41),
                49..=54 => (6, pin - 49),
                55..=62 => (7, pin - 55),
                63..=70 => (8, pin - 63),
                71..=78 => (9, pin - 71),
                79..=86 => (10, pin - 79),
                _ => panic!("Invalid pin number")
            };
            self.gpio[gpio_bank as usize].set_input_pin(gpio_index, state);
        }
    }

    fn pin_count() -> usize {
        1 + 86
    }

    fn pin_name(pin: PinId) -> String {
        match pin {
            0 => "CLK".to_string(),
            1..=86 => {
                let mut s = String::with_capacity(3);
                s.push('P');
                let (port, i) = match pin {
                    1 ..=8  => ('A', pin - 1),
                    9 ..=16 => ('B', pin - 9),
                    17..=24 => ('C', pin - 17),
                    25..=32 => ('D', pin - 25),
                    33..=40 => ('E', pin - 33),
                    41..=48 => ('F', pin - 41),
                    49..=54 => ('G', pin - 49),
                    55..=62 => ('H', pin - 55),
                    63..=70 => ('J', pin - 63),
                    71..=78 => ('K', pin - 71),
                    79..=86 => ('L', pin - 79),
                    _ => panic!("Invalid pin number")
                };
                s.push(port);
                s.push(char::from_digit(i as u32, 10).unwrap());
                s
            }
            _ => panic!("Invalid pin number")
        }
    }

    fn is_clock_rising(&self) -> bool {
        self.is_clock_rising
    }
    fn clock_pin(&self) -> PinState {
        self.clock_pin
    }

    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId,PinState>) {
        const GPIO_STARTS: [u16; 11] = [1, 9, 17, 25, 33, 41, 49, 55, 63, 71, 79];
        for (gpio_bank,     bank) in self.gpio.iter_mut().enumerate() {
            for &(pin_index, state) in bank.get_output_changes() {
                let pin = GPIO_STARTS[gpio_bank] + pin_index;
                changes.insert(pin, state);
            }
            bank.reset_pins();
        }
    }

    fn advance(&mut self) {
        if self.is_clock_rising {
            for gpio_bank in self.gpio.iter_mut() {
                gpio_bank.clock_rising_edge();
            }
        }
    }
}