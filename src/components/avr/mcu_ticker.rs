use std::collections::HashMap;

use crate::{component::Component, pins::{PinId, PinState}};

use super::{mcu::Mcu, mcu_model::McuModel, io_controller::{IoControllerTrait, IoController}};

pub struct McuTicker<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    mcu: Mcu<M, Io>,
    ticks: u8,
}

impl<M> McuTicker<M, IoController<M>>
where
    M: McuModel + 'static,
{
    pub fn new() -> McuTicker<M, IoController<M>> {
        let io = IoController::new();
        McuTicker {
            mcu: Mcu::new(io),
            ticks: 1
        }
    }

    pub fn tick(&mut self) {
        if self.ticks == 0 {
            self.ticks = self.mcu.step();
        }
        
        assert!(self.ticks > 0);
        self.ticks -= 1;
    }

    pub fn load_flash(&mut self, data: &[u16]) {
        self.mcu.load_flash(data);
    }
}

impl<M> Component for McuTicker<M, IoController<M>> 
where
    M: McuModel + 'static
{
    fn pin_count() -> usize {
        IoController::<M>::pin_count()
    }

    fn advance(&mut self) {
        if self.mcu.io.is_clock_rising() {
            self.tick()
        }
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.mcu.io.set_pin(pin, state)
    }

    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId, PinState>) {
        self.mcu.io.fill_output_changes(changes)
    }
}