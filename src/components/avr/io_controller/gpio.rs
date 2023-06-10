use bitfield::Bit;

use crate::pins::{PinState, PinId};

/// GPIO port, together with IO registers.
#[derive(Debug, Clone)]
pub struct GpioPort {
    port_register: u8,
    ddr_register: u8,

    output_states: [PinState; 8],
    output_changes: Vec<(PinId, PinState)>,
    readable_states: [PinState; 8],
    input_states: [PinState; 8],
    pub changed: bool,
}

impl GpioPort {
    pub fn new() -> GpioPort {
        GpioPort {
            port_register: 0,
            ddr_register: 0,
            output_states: [PinState::Z; 8],
            output_changes: Vec::with_capacity(8),
            readable_states: [PinState::Z; 8],
            input_states: [PinState::Z; 8],
            changed: false,
        }
    }

    #[inline]
    pub fn set_input_pin(&mut self, pin: PinId, state: PinState) {
        assert!(pin < 8);
        self.input_states[pin as usize] = state.read();
    }
    #[inline]
    pub fn get_output_changes(&self) -> &[(PinId, PinState)] {
        &self.output_changes
    }
    #[inline]
    pub fn reset_pins(&mut self) {
        self.output_changes.clear()
    }

    #[inline]
    pub fn read_port(&self) -> u8 {
        self.port_register
    }

    #[inline]
    pub fn read_ddr(&self) -> u8 {
        self.ddr_register
    }

    pub fn read_pin(&self) -> u8 {
        let mut x = 0;
        for i in 0..8 {
            if self.readable_states[i] == PinState::High {
                x.set_bit(i, true);
            }
        }
        x
    }

    #[inline]
    pub fn clock_rising_edge(&mut self) {
        self.readable_states = self.input_states;
    }

    fn set_output_state(&mut self, i: usize, state: PinState) {
        if self.output_states[i] != state {
            self.output_states[i] = state;
            self.output_changes.push((i as u16, state));
            self.changed = true;
        }
    }

    fn update_outputs(&mut self) {
        for i in 0..8 {
            let port = self.port_register.bit(i);
            let dd = self.ddr_register.bit(i);
            match (dd, port) {
                (false, false) => self.set_output_state(i, PinState::Z),
                (false, true) => self.set_output_state(i, PinState::WeakHigh),
                (true, false) => self.set_output_state(i, PinState::Low),
                (true, true) => self.set_output_state(i, PinState::High),
            }
        }
    }

    #[inline]
    pub fn write_port(&mut self, val: u8) {
        self.port_register = val;
        self.update_outputs();
    }

    #[inline]
    pub fn write_ddr(&mut self, val: u8) {
        self.ddr_register = val;
        self.update_outputs();
    }

    pub fn write_pin(&mut self, val: u8) {
        for i in 0..8 {
            if val.bit(i) {
                self.port_register ^= 1 << i;
            }
        }
        self.update_outputs();
    }
}