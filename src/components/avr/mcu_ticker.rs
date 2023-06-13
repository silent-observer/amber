use crate::vcd::{VcdFiller, VcdConfig, VcdTree};
use crate::pins::{PinId, PinState};
use crate::component::Component;

use super::{mcu::Mcu, mcu_model::McuModel, io_controller::{IoControllerTrait, IoController}};

/// Top level AVR MCU component.
/// 
/// Accounts for correct timings.
pub struct McuTicker<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    mcu: Mcu<M, Io>,
    ticks: u8,
}

/// AVR MCU with a default IoController.
pub type McuDefault<M> = McuTicker<M, IoController<M>>;

impl<M> McuTicker<M, IoController<M>>
where
    M: McuModel + 'static,
{
    /// Creates a new AVR MCU.
    pub fn new() -> McuTicker<M, IoController<M>> {
        let io = IoController::new();
        McuTicker {
            mcu: Mcu::new(io),
            ticks: 1
        }
    }
}

impl<M, Io> McuTicker<M, Io> 
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    // Advances MCU a single clock forward.
    pub fn tick(&mut self) {
        if self.ticks == 0 {
            self.ticks = self.mcu.step();
        }
        
        assert!(self.ticks > 0);
        self.ticks -= 1;
    }

    /// Loads MCU flash memory from a slice.
    pub fn load_flash(&mut self, data: &[u16]) {
        self.mcu.load_flash(data);
    }
    /// Loads MCU flash memory from a hex file.
    pub fn load_flash_hex(&mut self, filename: &str) {
        self.mcu.load_flash_hex(filename);
    }
}

/// Custom [Component] implementation, forwarding everything to `IoController`.
impl<M, Io> Component for McuTicker<M, Io> 
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    fn pin_count() -> usize {
        IoController::<M>::pin_count()
    }

    fn advance(&mut self, _time_ns: f64) -> Option<f64> {None}

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.mcu.io.set_pin(pin, state)
    }

    fn get_output_changes(&mut self) -> &[(PinId, PinState)] {
        self.mcu.io.get_output_changes()
    }

    fn pin_name(pin: PinId) -> String {
        Io::pin_name(pin)
    }

    fn clock_rising_edge(&mut self) {
        self.mcu.io.clock_rising_edge();
        self.tick();
    }

    fn clock_falling_edge(&mut self) {
        self.mcu.io.clock_falling_edge();
    }
}

/// Custom [VcdFiller] implementation, forwarding everything to `Mcu`.
impl<M, Io> VcdFiller for McuTicker<M, Io>
where
    M: McuModel + 'static,
    Io: IoControllerTrait,
{
    const IS_SIGNAL: bool = false;
    fn init_vcd(&self, config: &VcdConfig) -> VcdTree {
        self.mcu.init_vcd(config)
    }
    fn fill_vcd(&self, tree: &mut VcdTree) -> bool {
        self.mcu.fill_vcd(tree)
    }
}