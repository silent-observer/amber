use std::marker::PhantomData;
use mockall::*;

use crate::pins::{PinId, PinState};

use super::mcu_model::McuModel;

#[automock]
pub trait IoControllerTrait {
    fn read_internal_u8(&self, id: u8) -> u8;
    fn read_external_u8(&self, addr: u16) -> u8;
    fn write_internal_u8(&mut self, id: u8, val: u8);
    fn write_external_u8(&mut self, addr: u16, val: u8);

    fn is_clock_rising(&self) -> bool;

    fn pin_count() -> usize;
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn get_pin_output_changes(&self) -> &[(PinId, PinState)];
    fn reset_pins(&mut self);
}

pub struct IoController<M: McuModel> {
    model: PhantomData<M>,
    pins: Vec<PinState>,
    pin_output_changes: Vec<(PinId, PinState)>,
    is_clock_rising: bool,
}

impl<M: McuModel + 'static> IoController<M>{
    pub fn new() -> IoController<M> {
        IoController { 
            model: PhantomData,
            pins: vec![PinState::Z; Self::pin_count()],
            pin_output_changes: vec![],
            is_clock_rising: false,
        }
    }
}

const CLOCK_PIN: PinId = 0;

impl<M: McuModel + 'static> IoControllerTrait for IoController<M> {

    fn read_internal_u8(&self, id: u8) -> u8 {
        todo!()
    }

    fn read_external_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_internal_u8(&mut self, id: u8, val: u8) {
        todo!()
    }

    fn write_external_u8(&mut self, addr: u16, val: u8) {
        todo!()
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        if pin == CLOCK_PIN {
            if self.pins[CLOCK_PIN as usize] == PinState::Low && state == PinState::High {
                self.is_clock_rising = true;
            }
        }
        self.pins[pin as usize] = state;
        
    }

    fn get_pin_output_changes(&self) -> &[(PinId,PinState)] {
        &self.pin_output_changes
    }

    fn reset_pins(&mut self) {
        self.pin_output_changes.clear();
        self.is_clock_rising = false;
    }

    fn pin_count() -> usize {
        1
    }

    fn is_clock_rising(&self) -> bool {
        self.is_clock_rising
    }
}