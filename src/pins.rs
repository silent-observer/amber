use bitfield::Bit;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinState {
    Z,
    Low,
    High,
    WeakLow,
    WeakHigh,
    Error,
}

impl PinState {
    pub fn from_bool(b: bool) -> PinState {
        if b {
            PinState::High
        } else {
            PinState::Low
        }
    }

    pub fn read(self) -> PinState {
        match self {
            PinState::Z => PinState::Z,
            PinState::Low | PinState::WeakLow => PinState::Low,
            PinState::High | PinState::WeakHigh => PinState::High,
            PinState::Error => PinState::Error,
        }
    }
}

pub type PinId = u16;

pub trait PinStateConvertible {
    fn to_pin_vec(self) -> Vec<PinState>;
}

impl PinStateConvertible for bool {
    fn to_pin_vec(self) -> Vec<PinState> {
        vec![PinState::from_bool(self)]
    }
}

impl PinStateConvertible for u8 {
    fn to_pin_vec(self) -> Vec<PinState> {
        let mut result = vec![PinState::Low; 8];
        for i in 0..8 {
            result[i] = PinState::from_bool(self.bit(7-i));
        }
        result
    }
}

impl PinStateConvertible for u16 {
    fn to_pin_vec(self) -> Vec<PinState> {
        let mut result = vec![PinState::Low; 16];
        for i in 0..16 {
            result[i] = PinState::from_bool(self.bit(15-i));
        }
        result
    }
}

impl PinStateConvertible for u32 {
    fn to_pin_vec(self) -> Vec<PinState> {
        let mut result = vec![PinState::Low; 32];
        for i in 0..32 {
            result[i] = PinState::from_bool(self.bit(31-i));
        }
        result
    }
}

impl PinStateConvertible for PinState {
    fn to_pin_vec(self) -> Vec<PinState> {
        vec![self]
    }
}

impl PinStateConvertible for Vec<PinState> {
    fn to_pin_vec(self) -> Vec<PinState> {
        self
    }
}