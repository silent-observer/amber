use crate::{pins::PinState, component::Component};

struct Led {
    state: PinState,
}

impl Component for Led {
    fn pin_count() -> usize {
        1
    }

    fn advance(&mut self) {
        todo!()
    }

    fn set_pin(&mut self, pin: crate::pins::PinId, state: PinState) {
        assert!(pin == 0);
        self.state = state;
    }

    fn get_pin_output_changes(&self) -> &[(crate::pins::PinId, PinState)] {
        &[]
    }

    fn reset_pins(&mut self) {}
}