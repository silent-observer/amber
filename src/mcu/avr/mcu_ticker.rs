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