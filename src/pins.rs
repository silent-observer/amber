use bitfield::Bit;

/// State of an external pin.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinState {
    /// High-impendance state, also known as tri-state
    Z,
    /// Pin is connected to a strong sink (low voltage)
    Low,
    /// Pin is connected to a strong source (high voltage)
    High,
    /// Pin is connected to a pull-down resistor
    WeakLow,
    /// Pin is connected to a pull-up resistor
    WeakHigh,
    /// Pin is connected to a mix of sinks and sources, and so is in undefined state
    Error,
}

impl PinState {
    /// Converts a bool to [PinState].
    /// 
    /// ```
    /// # use amber::pins::PinState;
    /// assert_eq!(PinState::from_bool(true), PinState::High);
    /// assert_eq!(PinState::from_bool(false), PinState::Low);
    /// ```
    pub fn from_bool(b: bool) -> PinState {
        if b {
            PinState::High
        } else {
            PinState::Low
        }
    }

    /// Converts a full wire state to a state that would be read by an input pin.
    /// 
    /// Only converts weak states to `High`/`Low`.
    /// 
    /// ```
    /// # use amber::pins::PinState;
    /// assert_eq!(PinState::Low.read(), PinState::Low);
    /// assert_eq!(PinState::WeakLow.read(), PinState::Low);
    /// ```
    pub fn read(self) -> PinState {
        match self {
            PinState::Z => PinState::Z,
            PinState::Low | PinState::WeakLow => PinState::Low,
            PinState::High | PinState::WeakHigh => PinState::High,
            PinState::Error => PinState::Error,
        }
    }
}

/// Type for pin numbers of components.
pub type PinId = u16;

/// Can be converted to a vector of [PinState].
/// 
/// This is used for VCD signals.
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