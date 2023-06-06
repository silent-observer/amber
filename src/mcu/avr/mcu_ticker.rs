use crate::component::Component;

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

    fn set_pin(&mut self, pin: crate::pins::PinId, state: crate::pins::PinState) {
        self.mcu.io.set_pin(pin, state)
    }

    fn get_pin_output_changes(&self) -> &[(crate::pins::PinId, crate::pins::PinState)] {
        self.mcu.io.get_pin_output_changes()
    }

    fn reset_pins(&mut self) {
        self.mcu.io.reset_pins()
    }
}