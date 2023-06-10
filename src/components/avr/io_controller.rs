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
    const PIN_CLK: PinId = 0;

    const _PIN_PA0: PinId = 1;
    const _PIN_PA1: PinId = 2;
    const _PIN_PA2: PinId = 3;
    const _PIN_PA3: PinId = 4;
    const _PIN_PA4: PinId = 5;
    const _PIN_PA5: PinId = 6;
    const _PIN_PA6: PinId = 7;
    const _PIN_PA7: PinId = 8;

    const _PIN_PB0: PinId = 1*8 + 1;
    const _PIN_PB1: PinId = 1*8 + 2;
    const _PIN_PB2: PinId = 1*8 + 3;
    const _PIN_PB3: PinId = 1*8 + 4;
    const _PIN_PB4: PinId = 1*8 + 5;
    const _PIN_PB5: PinId = 1*8 + 6;
    const _PIN_PB6: PinId = 1*8 + 7;
    const _PIN_PB7: PinId = 1*8 + 8;

    const _PIN_PC0: PinId = 2*8 + 1;
    const _PIN_PC1: PinId = 2*8 + 2;
    const _PIN_PC2: PinId = 2*8 + 3;
    const _PIN_PC3: PinId = 2*8 + 4;
    const _PIN_PC4: PinId = 2*8 + 5;
    const _PIN_PC5: PinId = 2*8 + 6;
    const _PIN_PC6: PinId = 2*8 + 7;
    const _PIN_PC7: PinId = 2*8 + 8;

    const _PIN_PD0: PinId = 3*8 + 1;
    const _PIN_PD1: PinId = 3*8 + 2;
    const _PIN_PD2: PinId = 3*8 + 3;
    const _PIN_PD3: PinId = 3*8 + 4;
    const _PIN_PD4: PinId = 3*8 + 5;
    const _PIN_PD5: PinId = 3*8 + 6;
    const _PIN_PD6: PinId = 3*8 + 7;
    const _PIN_PD7: PinId = 3*8 + 8;

    const _PIN_PE0: PinId = 4*8 + 1;
    const _PIN_PE1: PinId = 4*8 + 2;
    const _PIN_PE2: PinId = 4*8 + 3;
    const _PIN_PE3: PinId = 4*8 + 4;
    const _PIN_PE4: PinId = 4*8 + 5;
    const _PIN_PE5: PinId = 4*8 + 6;
    const _PIN_PE6: PinId = 4*8 + 7;
    const _PIN_PE7: PinId = 4*8 + 8;

    const _PIN_PF0: PinId = 5*8 + 1;
    const _PIN_PF1: PinId = 5*8 + 2;
    const _PIN_PF2: PinId = 5*8 + 3;
    const _PIN_PF3: PinId = 5*8 + 4;
    const _PIN_PF4: PinId = 5*8 + 5;
    const _PIN_PF5: PinId = 5*8 + 6;
    const _PIN_PF6: PinId = 5*8 + 7;
    const _PIN_PF7: PinId = 5*8 + 8;

    const _PIN_PG0: PinId = 6*8 + 1;
    const _PIN_PG1: PinId = 6*8 + 2;
    const _PIN_PG2: PinId = 6*8 + 3;
    const _PIN_PG3: PinId = 6*8 + 4;
    const _PIN_PG4: PinId = 6*8 + 5;
    const _PIN_PG5: PinId = 6*8 + 6;

    const _PIN_PH0: PinId = 6*8 + 6 + 1;
    const _PIN_PH1: PinId = 6*8 + 6 + 2;
    const _PIN_PH2: PinId = 6*8 + 6 + 3;
    const _PIN_PH3: PinId = 6*8 + 6 + 4;
    const _PIN_PH4: PinId = 6*8 + 6 + 5;
    const _PIN_PH5: PinId = 6*8 + 6 + 6;
    const _PIN_PH6: PinId = 6*8 + 6 + 7;
    const _PIN_PH7: PinId = 6*8 + 6 + 8;

    const _PIN_PJ0: PinId = 7*8 + 6 + 1;
    const _PIN_PJ1: PinId = 7*8 + 6 + 2;
    const _PIN_PJ2: PinId = 7*8 + 6 + 3;
    const _PIN_PJ3: PinId = 7*8 + 6 + 4;
    const _PIN_PJ4: PinId = 7*8 + 6 + 5;
    const _PIN_PJ5: PinId = 7*8 + 6 + 6;
    const _PIN_PJ6: PinId = 7*8 + 6 + 7;
    const _PIN_PJ7: PinId = 7*8 + 6 + 8;

    const _PIN_PK0: PinId = 8*8 + 6 + 1;
    const _PIN_PK1: PinId = 8*8 + 6 + 2;
    const _PIN_PK2: PinId = 8*8 + 6 + 3;
    const _PIN_PK3: PinId = 8*8 + 6 + 4;
    const _PIN_PK4: PinId = 8*8 + 6 + 5;
    const _PIN_PK5: PinId = 8*8 + 6 + 6;
    const _PIN_PK6: PinId = 8*8 + 6 + 7;
    const _PIN_PK7: PinId = 8*8 + 6 + 8;

    const _PIN_PL0: PinId = 9*8 + 6 + 1;
    const _PIN_PL1: PinId = 9*8 + 6 + 2;
    const _PIN_PL2: PinId = 9*8 + 6 + 3;
    const _PIN_PL3: PinId = 9*8 + 6 + 4;
    const _PIN_PL4: PinId = 9*8 + 6 + 5;
    const _PIN_PL5: PinId = 9*8 + 6 + 6;
    const _PIN_PL6: PinId = 9*8 + 6 + 7;
    const _PIN_PL7: PinId = 9*8 + 6 + 8;
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
        if pin == Self::PIN_CLK {
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