use super::{mcu::Mcu, mcu_model::McuModel};

pub struct McuTicker<M: McuModel> {
    mcu: Mcu<M>,
    ticks: u8,
}

impl<M: McuModel> McuTicker<M> {
    pub fn new() -> McuTicker<M> {
        McuTicker {
            mcu: Mcu::new(),
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