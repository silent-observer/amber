use std::collections::HashMap;

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
    fn fill_vcd(&self, tree: &mut VcdTree) {
        self.mcu.fill_vcd(tree)
    }
}