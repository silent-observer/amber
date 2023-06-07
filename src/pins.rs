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