#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinState {
    Low,
    High,
    WeakLow,
    WeakHigh,
    Z,
    Error,
}

pub type PinId = u16;