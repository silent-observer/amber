//! Simple LED component that can be used to capture a signal.

use std::collections::HashMap;

use crate::{pins::{PinState, PinId, PinStateConvertible, PinVec}, component::Component, vcd::{fillers::VcdFiller, VcdTreeSignal}};

pub struct Led {
    state: PinState,
}

impl Led {
    pub fn new() -> Led {
        Led {
            state: PinState::Z
        }
    }
}

impl Component for Led {
    fn pin_count() -> usize {
        1
    }

    fn advance(&mut self) {}

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        assert!(pin == 0);
        self.state = state;
        if self.state == PinState::High || self.state == PinState::Low {
        }
    }

    fn fill_output_changes(&mut self, _changes: &mut HashMap<PinId, PinState>) {}

    fn pin_name(pin_id: PinId) -> String {
        assert_eq!(pin_id, 0);
        "LED".to_string()
    }
}

impl VcdFiller for Led {
    const IS_SIGNAL: bool = true;

    fn init_vcd_signal(&self) -> VcdTreeSignal {
        VcdTreeSignal::new(1, PinState::Z)
    }

    fn get_signal_state(&self) -> PinVec {
        self.state.to_pin_vec()
    }
}