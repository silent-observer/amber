mod gpio;

use std::collections::HashMap;
use std::marker::PhantomData;
use mockall::*;

use crate::pins::{PinId, PinState};

use self::gpio::GpioPort;

use super::mcu_model::McuModel;

#[automock]
pub trait IoControllerTrait {
    fn read_internal_u8(&self, id: u8) -> u8;
    fn read_external_u8(&self, addr: u16) -> u8;
    fn write_internal_u8(&mut self, id: u8, val: u8);
    fn write_external_u8(&mut self, addr: u16, val: u8);

    fn is_clock_rising(&self) -> bool;
    fn clock_pin(&self) -> PinState;

    fn pin_count() -> usize;
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId, PinState>);
}

pub struct IoController<M: McuModel> {
    model: PhantomData<M>,
    clock_pin: PinState,
    is_clock_rising: bool,

    gpio: [GpioPort; 2]
}

impl<M: McuModel + 'static> IoController<M>{
    pub fn new() -> IoController<M> {
        IoController { 
            model: PhantomData,
            clock_pin: PinState::Z,
            is_clock_rising: false,
            gpio: [
                GpioPort::new(),
                GpioPort::new(),
            ]
        }
    }
}

const CLOCK_PIN: PinId = 0;

impl<M: McuModel + 'static> IoControllerTrait for IoController<M> {
    fn read_internal_u8(&self, id: u8) -> u8 {
        match id {
            0x00 => self.gpio[0].read_pin(), // PINA
            0x01 => self.gpio[0].read_ddr(), // DDRA
            0x02 => self.gpio[0].read_port(), // PORTA
            0x03 => self.gpio[1].read_pin(), // PINB
            0x04 => self.gpio[1].read_ddr(), // DDRB
            0x05 => self.gpio[1].read_port(), // PORTB
            _ => 0
        }
    }

    fn read_external_u8(&self, _addr: u16) -> u8 {
        todo!()
    }

    fn write_internal_u8(&mut self, id: u8, val: u8) {
        match id {
            0x00 => self.gpio[0].write_pin(val), // PINA
            0x01 => self.gpio[0].write_ddr(val), // DDRA
            0x02 => self.gpio[0].write_port(val), // PORTA
            0x03 => self.gpio[1].write_pin(val), // PINB
            0x04 => self.gpio[1].write_ddr(val), // DDRB
            0x05 => self.gpio[1].write_port(val), // PORTB
            _ => {}
        }
    }

    fn write_external_u8(&mut self, _addr: u16, _val: u8) {
        todo!()
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        if pin == CLOCK_PIN {
            self.is_clock_rising = self.clock_pin == PinState::Low && state == PinState::High;
            self.clock_pin = state;
        }
        else if pin < 1 + 2 * 8 {
            let gpio_bank = (pin - 1) / 8;
            let gpio_index = (pin - 1) % 8;
            self.gpio[gpio_bank as usize].set_input_pin(gpio_index, state);
        }
    }

    fn pin_count() -> usize {
        1 + 2 * 8
    }

    fn is_clock_rising(&self) -> bool {
        self.is_clock_rising
    }
    fn clock_pin(&self) -> PinState {
        self.clock_pin
    }

    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId,PinState>) {
        for (gpio_bank,     bank) in self.gpio.iter_mut().enumerate() {
            for &(pin_index, state) in bank.get_output_changes() {
                let pin = 1 + pin_index + (gpio_bank as u16) * 8;
                changes.insert(pin, state);
            }
            bank.reset_pins();
        }
    }
}