use bitfield::Bit;

use crate::{pins::{PinState, PinId, PinVec}, component::Component, vcd::{fillers::VcdFiller, VcdModuleBuilder, VcdTreeModule}};

pub struct Uart<const CHAR_SIZE: u8> {
    rx_pin_next: PinState,
    rx_pin: PinState,
    rx_falling_edge: bool,
    tx_pin: PinState,

    clk_period: f64,

    rx_start_time: f64,
    rx_pos: i8,

    rx_data: u16,
    rx_data_ready: Option<u16>,
    rx_running_parity: bool,
    rx_parity_error: bool,
    rx_frame_error: bool,

    parity: Option<bool>,
}

impl<const CHAR_SIZE: u8> Uart<CHAR_SIZE> {
    pub fn new(baud_rate: f64, parity: Option<bool>) -> Uart<CHAR_SIZE> {
        Uart {
            rx_pin_next: PinState::WeakHigh,
            rx_pin: PinState::WeakHigh,
            rx_falling_edge: false,
            tx_pin: PinState::High,

            clk_period: 1e9 / baud_rate,

            rx_start_time: 0.0,
            rx_pos: -1,

            parity,

            rx_data: 0,
            rx_data_ready: None,
            rx_running_parity: false,
            rx_parity_error: false,
            rx_frame_error: false,
        }
    }

    fn handle_bit(&mut self, bit: bool) {
        if self.rx_pos == 0 {
            if bit {
                self.rx_pos = -1;
            } else {
                self.rx_pos = 1;
                if let Some(parity) = self.parity {
                    self.rx_running_parity = parity;
                }
            }
        } else if self.rx_pos <= CHAR_SIZE as i8 {
            self.rx_data.set_bit(self.rx_pos as usize - 1, bit);
            self.rx_running_parity ^= bit;
            self.rx_pos += 1;
        } else if self.parity.is_some() && self.rx_pos == CHAR_SIZE as i8 + 1 {
            if self.rx_running_parity != bit {
                self.rx_parity_error = true;
            }
            self.rx_pos += 1;
        } else {
            if !bit {
                self.rx_frame_error = true;
            }
            if self.rx_frame_error {
                println!("Frame error!");
            } else if self.rx_parity_error {
                println!("Parity error!");
            } else {
                println!("{:02X}", self.rx_data);
                self.rx_data_ready = Some(self.rx_data);
            }
            self.rx_pos = -1;
        }
    }
}

impl<const CHAR_SIZE: u8> Component for Uart<CHAR_SIZE> {
    fn pin_count() -> usize {
        2
    }

    fn advance(&mut self, time_ns: f64) -> Option<f64> {
        if self.rx_pos == -1 {
            if self.rx_falling_edge {
                self.rx_start_time = time_ns;
                self.rx_pos = 0;
                self.rx_parity_error = false;
                self.rx_frame_error = false;
                self.rx_pin = self.rx_pin_next;
                Some(time_ns + self.clk_period * (CHAR_SIZE as f64 + 2.5))
            } else {
                self.rx_pin = self.rx_pin_next;
                None
            }
        } else {
            let bit = self.rx_pin == PinState::High;
            let mut next_bit_time = self.rx_start_time +
                (0.5 + self.rx_pos as f64) * self.clk_period;
            while time_ns >= next_bit_time {
                self.handle_bit(bit);
                next_bit_time += self.clk_period;
            }
            self.rx_pin = self.rx_pin_next;
            None
        }
    }

    fn set_pin(&mut self, pin: PinId, state: PinState) {
        match pin {
            1 => {
                if (self.rx_pin == PinState::High || self.rx_pin == PinState::WeakHigh) &&
                    state == PinState::Low {
                        self.rx_falling_edge = true;
                } else {
                    self.rx_falling_edge = false;
                }
                self.rx_pin_next = state
            },
            _ => {}
        }
    }

    fn get_output_changes(&mut self) -> &[(PinId, PinState)] {
        &[]
    }

    fn pin_name(pin_id: PinId) -> String {
        match pin_id {
            0 => "TX".to_string(),
            1 => "RX".to_string(),
            _ => panic!("Invalid pin id")
        }
    }

    fn clock_rising_edge(&mut self) {
        unimplemented!()
    }

    fn clock_falling_edge(&mut self) {
        unimplemented!()
    }
}

impl<const CHAR_SIZE: u8> VcdFiller for Uart<CHAR_SIZE> {
    const IS_SIGNAL: bool = false;
    
    fn init_vcd_module(&self, builder: &mut VcdModuleBuilder) {
        builder.add_signal("tx", 1, PinState::High);
        builder.add_signal("rx", 1, PinState::High);
        builder.add_signal("rx_data", CHAR_SIZE, PinState::Z);
    }

    fn fill_module(&self, module: &mut VcdTreeModule) -> bool {
        let mut r = module.update_subsignal(0, self.tx_pin);
        r |= module.update_subsignal(1, self.rx_pin);
        if let Some(data) = self.rx_data_ready {
            r |= module.update_subsignal(2, PinVec::init_logical(CHAR_SIZE, data as u32));
        };
        r
    }
}