mod gpio;
mod timer16;

use std::marker::PhantomData;
use mockall::*;

use crate::pins::{PinId, PinState};

use self::{gpio::GpioPort, timer16::Timer16};

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
    fn clock_rising_edge(&mut self);

    fn clock_falling_edge(&mut self);
    // Get current clock pin value
    fn clock_pin(&self) -> PinState;

    /// Get number of pins
    fn pin_count() -> usize;
    /// Get pin names
    fn pin_name(pin: PinId) -> String;
    /// Set input pin value
    fn set_pin(&mut self, pin: PinId, state: PinState);
    /// Get output pin changes (by filling a [HashMap])
    fn get_output_changes(&mut self) -> &[(PinId, PinState)];
}
/// Main implementation for [IoControllerTrait]
pub struct IoController<M: McuModel> {
    model: PhantomData<M>,
    clock_pin: PinState,

    output_changes: Vec<(PinId, PinState)>,

    gpio: [GpioPort; 11],

    timer_prescaler: u16,

    timer1: Timer16,
    timer3: Timer16,
    timer4: Timer16,
    timer5: Timer16,

    gpio_pins: [(bool, PinState); 86],
}

impl<M: McuModel + 'static> IoController<M>{
    pub fn new() -> IoController<M> {
        IoController { 
            model: PhantomData,
            clock_pin: PinState::Low,
            gpio: std::array::from_fn(|_| GpioPort::new()),
            output_changes: Vec::with_capacity(8),
            gpio_pins: [(true, PinState::Z); 86],
            timer_prescaler: 0,
            timer1: Timer16::new([Self::PIN_PB5, Self::PIN_PB6, Self::PIN_PB7]),
            timer3: Timer16::new([Self::PIN_PE3, Self::PIN_PE4, Self::PIN_PE5]),
            timer4: Timer16::new([Self::PIN_PH3, Self::PIN_PH4, Self::PIN_PH5]),
            timer5: Timer16::new([Self::PIN_PL3, Self::PIN_PL4, Self::PIN_PL5]),
        }
    }
}

impl<M: McuModel> IoController<M> {
    const _PIN_PA0: PinId = 0;
    const _PIN_PA1: PinId = 1;
    const _PIN_PA2: PinId = 2;
    const _PIN_PA3: PinId = 3;
    const _PIN_PA4: PinId = 4;
    const _PIN_PA5: PinId = 5;
    const _PIN_PA6: PinId = 6;
    const _PIN_PA7: PinId = 7;

    const _PIN_PB0: PinId = 1*8 + 0;
    const _PIN_PB1: PinId = 1*8 + 1;
    const _PIN_PB2: PinId = 1*8 + 2;
    const _PIN_PB3: PinId = 1*8 + 3;
    const _PIN_PB4: PinId = 1*8 + 4;
    const PIN_PB5: PinId = 1*8 + 5;
    const PIN_PB6: PinId = 1*8 + 6;
    const PIN_PB7: PinId = 1*8 + 7;

    const _PIN_PC0: PinId = 2*8 + 0;
    const _PIN_PC1: PinId = 2*8 + 1;
    const _PIN_PC2: PinId = 2*8 + 2;
    const _PIN_PC3: PinId = 2*8 + 3;
    const _PIN_PC4: PinId = 2*8 + 4;
    const _PIN_PC5: PinId = 2*8 + 5;
    const _PIN_PC6: PinId = 2*8 + 6;
    const _PIN_PC7: PinId = 2*8 + 7;

    const _PIN_PD0: PinId = 3*8 + 0;
    const _PIN_PD1: PinId = 3*8 + 1;
    const _PIN_PD2: PinId = 3*8 + 2;
    const _PIN_PD3: PinId = 3*8 + 3;
    const _PIN_PD4: PinId = 3*8 + 4;
    const _PIN_PD5: PinId = 3*8 + 5;
    const _PIN_PD6: PinId = 3*8 + 6;
    const _PIN_PD7: PinId = 3*8 + 7;

    const _PIN_PE0: PinId = 4*8 + 0;
    const _PIN_PE1: PinId = 4*8 + 1;
    const _PIN_PE2: PinId = 4*8 + 2;
    const PIN_PE3: PinId = 4*8 + 3;
    const PIN_PE4: PinId = 4*8 + 4;
    const PIN_PE5: PinId = 4*8 + 5;
    const _PIN_PE6: PinId = 4*8 + 6;
    const _PIN_PE7: PinId = 4*8 + 7;

    const _PIN_PF0: PinId = 5*8 + 0;
    const _PIN_PF1: PinId = 5*8 + 1;
    const _PIN_PF2: PinId = 5*8 + 2;
    const _PIN_PF3: PinId = 5*8 + 3;
    const _PIN_PF4: PinId = 5*8 + 4;
    const _PIN_PF5: PinId = 5*8 + 5;
    const _PIN_PF6: PinId = 5*8 + 6;
    const _PIN_PF7: PinId = 5*8 + 7;

    const _PIN_PG0: PinId = 6*8 + 0;
    const _PIN_PG1: PinId = 6*8 + 1;
    const _PIN_PG2: PinId = 6*8 + 2;
    const _PIN_PG3: PinId = 6*8 + 3;
    const _PIN_PG4: PinId = 6*8 + 4;
    const _PIN_PG5: PinId = 6*8 + 5;

    const _PIN_PH0: PinId = 6*8 + 6 + 0;
    const _PIN_PH1: PinId = 6*8 + 6 + 1;
    const _PIN_PH2: PinId = 6*8 + 6 + 2;
    const PIN_PH3: PinId = 6*8 + 6 + 3;
    const PIN_PH4: PinId = 6*8 + 6 + 4;
    const PIN_PH5: PinId = 6*8 + 6 + 5;
    const _PIN_PH6: PinId = 6*8 + 6 + 6;
    const _PIN_PH7: PinId = 6*8 + 6 + 7;

    const _PIN_PJ0: PinId = 7*8 + 6 + 0;
    const _PIN_PJ1: PinId = 7*8 + 6 + 1;
    const _PIN_PJ2: PinId = 7*8 + 6 + 2;
    const _PIN_PJ3: PinId = 7*8 + 6 + 3;
    const _PIN_PJ4: PinId = 7*8 + 6 + 4;
    const _PIN_PJ5: PinId = 7*8 + 6 + 5;
    const _PIN_PJ6: PinId = 7*8 + 6 + 6;
    const _PIN_PJ7: PinId = 7*8 + 6 + 7;

    const _PIN_PK0: PinId = 8*8 + 6 + 0;
    const _PIN_PK1: PinId = 8*8 + 6 + 1;
    const _PIN_PK2: PinId = 8*8 + 6 + 2;
    const _PIN_PK3: PinId = 8*8 + 6 + 3;
    const _PIN_PK4: PinId = 8*8 + 6 + 4;
    const _PIN_PK5: PinId = 8*8 + 6 + 5;
    const _PIN_PK6: PinId = 8*8 + 6 + 6;
    const _PIN_PK7: PinId = 8*8 + 6 + 7;

    const _PIN_PL0: PinId = 9*8 + 6 + 0;
    const _PIN_PL1: PinId = 9*8 + 6 + 1;
    const _PIN_PL2: PinId = 9*8 + 6 + 2;
    const PIN_PL3: PinId = 9*8 + 6 + 3;
    const PIN_PL4: PinId = 9*8 + 6 + 4;
    const PIN_PL5: PinId = 9*8 + 6 + 5;
    const _PIN_PL6: PinId = 9*8 + 6 + 6;
    const _PIN_PL7: PinId = 9*8 + 6 + 7;
}

fn update_changes(output_changes: &mut Vec<(PinId, PinState)>, gpio_bank: usize, changes: &[(PinId, PinState)], gpio_pins: &mut [(bool, PinState); 86]) {
    const GPIO_STARTS: [u16; 11] = [0, 8, 16, 24, 32, 40, 48, 54, 62, 70, 78];
    for &(pin_index, state) in changes {
        let pin = GPIO_STARTS[gpio_bank] + pin_index;
        if !gpio_pins[pin as usize].0 {
            output_changes.push((pin, state));
        }
        gpio_pins[pin as usize].1 = state;
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
            0x080 => self.timer1.read_tccra(),
            0x081 => self.timer1.read_tccrb(),
            0x084 => self.timer1.read_tcntl(),
            0x085 => self.timer1.read_tcnth(),
            0x088 => self.timer1.read_ocral(),
            0x089 => self.timer1.read_ocrah(),
            0x08A => self.timer1.read_ocrbl(),
            0x08B => self.timer1.read_ocrbh(),
            0x08C => self.timer1.read_ocrcl(),
            0x08D => self.timer1.read_ocrch(),

            0x090 => self.timer3.read_tccra(),
            0x091 => self.timer3.read_tccrb(),
            0x094 => self.timer3.read_tcntl(),
            0x095 => self.timer3.read_tcnth(),
            0x098 => self.timer3.read_ocral(),
            0x099 => self.timer3.read_ocrah(),
            0x09A => self.timer3.read_ocrbl(),
            0x09B => self.timer3.read_ocrbh(),
            0x09C => self.timer3.read_ocrcl(),
            0x09D => self.timer3.read_ocrch(),

            0x0A0 => self.timer4.read_tccra(),
            0x0A1 => self.timer4.read_tccrb(),
            0x0A4 => self.timer4.read_tcntl(),
            0x0A5 => self.timer4.read_tcnth(),
            0x0A8 => self.timer4.read_ocral(),
            0x0A9 => self.timer4.read_ocrah(),
            0x0AA => self.timer4.read_ocrbl(),
            0x0AB => self.timer4.read_ocrbh(),
            0x0AC => self.timer4.read_ocrcl(),
            0x0AD => self.timer4.read_ocrch(),

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

            0x120 => self.timer5.read_tccra(),
            0x121 => self.timer5.read_tccrb(),
            0x124 => self.timer5.read_tcntl(),
            0x125 => self.timer5.read_tcnth(),
            0x128 => self.timer5.read_ocral(),
            0x129 => self.timer5.read_ocrah(),
            0x12A => self.timer5.read_ocrbl(),
            0x12B => self.timer5.read_ocrbh(),
            0x12C => self.timer5.read_ocrcl(),
            0x12D => self.timer5.read_ocrch(),
            _ => 0
        }
    }

    fn write_internal_u8(&mut self, id: u8, val: u8) {
        const EMPTY: &[(PinId, PinState)] = &[];
        let (gpio_bank, changes) = match id {
            0x00 => (0, self.gpio[0].write_pin(val)), // PINA
            0x01 => (0, self.gpio[0].write_ddr(val)), // DDRA
            0x02 => (0, self.gpio[0].write_port(val)), // PORTA

            0x03 => (1, self.gpio[1].write_pin(val)), // PINB
            0x04 => (1, self.gpio[1].write_ddr(val)), // DDRB
            0x05 => (1, self.gpio[1].write_port(val)), // PORTB

            0x06 => (2, self.gpio[2].write_pin(val)), // PINC
            0x07 => (2, self.gpio[2].write_ddr(val)), // DDRC
            0x08 => (2, self.gpio[2].write_port(val)), // PORTC

            0x09 => (3, self.gpio[3].write_pin(val)), // PIND
            0x0A => (3, self.gpio[3].write_ddr(val)), // DDRD
            0x0B => (3, self.gpio[3].write_port(val)), // PORTD

            0x0C => (4, self.gpio[4].write_pin(val)), // PINE
            0x0D => (4, self.gpio[4].write_ddr(val)), // DDRE
            0x0E => (4, self.gpio[4].write_port(val)), // PORTE

            0x0F => (5, self.gpio[5].write_pin(val)), // PINF
            0x10 => (5, self.gpio[5].write_ddr(val)), // DDRF
            0x11 => (5, self.gpio[5].write_port(val)), // PORTF

            0x12 => (6, self.gpio[6].write_pin(val)), // PING
            0x13 => (6, self.gpio[6].write_ddr(val)), // DDRG
            0x14 => (6, self.gpio[6].write_port(val)), // PORTG
            _ => {(0, EMPTY)}
        };
        update_changes(&mut self.output_changes, gpio_bank, changes, &mut self.gpio_pins)
    }

    fn write_external_u8(&mut self, addr: u16, val: u8) {
        const EMPTY: &[(PinId, PinState)] = &[];
        let (gpio_bank, changes) = match addr {
            0x080 => {self.timer1.write_tccra(val, &mut self.output_changes, &mut self.gpio_pins); (0, EMPTY)}
            0x081 => {self.timer1.write_tccrb(val); (0, EMPTY)}
            0x084 => {self.timer1.write_tcntl(val); (0, EMPTY)}
            0x085 => {self.timer1.write_tcnth(val); (0, EMPTY)}
            0x088 => {self.timer1.write_ocral(val); (0, EMPTY)}
            0x089 => {self.timer1.write_ocrah(val); (0, EMPTY)}
            0x08A => {self.timer1.write_ocrbl(val); (0, EMPTY)}
            0x08B => {self.timer1.write_ocrbh(val); (0, EMPTY)}
            0x08C => {self.timer1.write_ocrcl(val); (0, EMPTY)}
            0x08D => {self.timer1.write_ocrch(val); (0, EMPTY)}

            0x090 => {self.timer3.write_tccra(val, &mut self.output_changes, &mut self.gpio_pins); (0, EMPTY)}
            0x091 => {self.timer3.write_tccrb(val); (0, EMPTY)}
            0x094 => {self.timer3.write_tcntl(val); (0, EMPTY)}
            0x095 => {self.timer3.write_tcnth(val); (0, EMPTY)}
            0x098 => {self.timer3.write_ocral(val); (0, EMPTY)}
            0x099 => {self.timer3.write_ocrah(val); (0, EMPTY)}
            0x09A => {self.timer3.write_ocrbl(val); (0, EMPTY)}
            0x09B => {self.timer3.write_ocrbh(val); (0, EMPTY)}
            0x09C => {self.timer3.write_ocrcl(val); (0, EMPTY)}
            0x09D => {self.timer3.write_ocrch(val); (0, EMPTY)}

            0x0A0 => {self.timer4.write_tccra(val, &mut self.output_changes, &mut self.gpio_pins); (0, EMPTY)}
            0x0A1 => {self.timer4.write_tccrb(val); (0, EMPTY)}
            0x0A4 => {self.timer4.write_tcntl(val); (0, EMPTY)}
            0x0A5 => {self.timer4.write_tcnth(val); (0, EMPTY)}
            0x0A8 => {self.timer4.write_ocral(val); (0, EMPTY)}
            0x0A9 => {self.timer4.write_ocrah(val); (0, EMPTY)}
            0x0AA => {self.timer4.write_ocrbl(val); (0, EMPTY)}
            0x0AB => {self.timer4.write_ocrbh(val); (0, EMPTY)}
            0x0AC => {self.timer4.write_ocrcl(val); (0, EMPTY)}
            0x0AD => {self.timer4.write_ocrch(val); (0, EMPTY)}

            0x100 => (7, self.gpio[7].write_pin(val)), // PINH
            0x101 => (7, self.gpio[7].write_ddr(val)), // DDRH
            0x102 => (7, self.gpio[7].write_port(val)), // PORTH

            0x103 => (8, self.gpio[8].write_pin(val)), // PINJ
            0x104 => (8, self.gpio[8].write_ddr(val)), // DDRJ
            0x105 => (8, self.gpio[8].write_port(val)), // PORTJ

            0x106 => (9, self.gpio[9].write_pin(val)), // PINK
            0x107 => (9, self.gpio[9].write_ddr(val)), // DDRK
            0x108 => (9, self.gpio[9].write_port(val)), // PORTK

            0x109 => (10, self.gpio[10].write_pin(val)), // PINL
            0x10A => (10, self.gpio[10].write_ddr(val)), // DDRL
            0x10B => (10, self.gpio[10].write_port(val)), // PORTL

            0x120 => {self.timer5.write_tccra(val, &mut self.output_changes, &mut self.gpio_pins); (0, EMPTY)}
            0x121 => {self.timer5.write_tccrb(val); (0, EMPTY)}
            0x124 => {self.timer5.write_tcntl(val); (0, EMPTY)}
            0x125 => {self.timer5.write_tcnth(val); (0, EMPTY)}
            0x128 => {self.timer5.write_ocral(val); (0, EMPTY)}
            0x129 => {self.timer5.write_ocrah(val); (0, EMPTY)}
            0x12A => {self.timer5.write_ocrbl(val); (0, EMPTY)}
            0x12B => {self.timer5.write_ocrbh(val); (0, EMPTY)}
            0x12C => {self.timer5.write_ocrcl(val); (0, EMPTY)}
            0x12D => {self.timer5.write_ocrch(val); (0, EMPTY)}
            _ => {(0, EMPTY)}
        };
        update_changes(&mut self.output_changes, gpio_bank, changes, &mut self.gpio_pins)
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        let (gpio_bank, gpio_index) = match pin {
            0 ..=7  => (0, pin - 0),
            8 ..=15 => (1, pin - 8),
            16..=23 => (2, pin - 16),
            24..=31 => (3, pin - 24),
            32..=39 => (4, pin - 32),
            40..=47 => (5, pin - 40),
            48..=53 => (6, pin - 48),
            54..=61 => (7, pin - 54),
            62..=69 => (8, pin - 62),
            70..=77 => (9, pin - 70),
            78..=85 => (10, pin - 78),
            _ => panic!("Invalid pin number")
        };
        self.gpio[gpio_bank as usize].set_input_pin(gpio_index, state);
    }

    fn pin_count() -> usize {
        86
    }

    fn pin_name(pin: PinId) -> String {
        match pin {
            0..=85 => {
                let mut s = String::with_capacity(3);
                s.push('P');
                let (port, i) = match pin {
                    0 ..=7  => ('A', pin - 0),
                    8 ..=15 => ('B', pin - 8),
                    16..=23 => ('C', pin - 16),
                    24..=31 => ('D', pin - 24),
                    32..=39 => ('E', pin - 32),
                    40..=47 => ('F', pin - 40),
                    48..=53 => ('G', pin - 48),
                    54..=61 => ('H', pin - 54),
                    62..=69 => ('J', pin - 62),
                    70..=77 => ('K', pin - 70),
                    78..=85 => ('L', pin - 78),
                    _ => panic!("Invalid pin number")
                };
                s.push(port);
                s.push(char::from_digit(i as u32, 10).unwrap());
                s
            }
            _ => panic!("Invalid pin number")
        }
    }
    
    #[inline]
    fn clock_pin(&self) -> PinState {
        self.clock_pin
    }

    fn get_output_changes(&mut self) -> &[(PinId, PinState)] {
        &self.output_changes
    }

    fn clock_rising_edge(&mut self) {
        self.clock_pin = PinState::High;
        self.output_changes.clear();
        for gpio_bank in self.gpio.iter_mut() {
            gpio_bank.clock_rising_edge();
        }
        self.timer1.tick_prescaler(self.timer_prescaler, &mut self.output_changes);
        self.timer_prescaler = (self.timer_prescaler + 1) % 1024;
    }

    fn clock_falling_edge(&mut self) {
        self.clock_pin = PinState::Low;
    }
}