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
    #[inline]
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
/// 
/// This is unique only for every component.
pub type PinId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinVec {
    SinglePin(PinState),
    SmallLogical {
        size: u8,
        bits: u32,
    },
    //Big(Vec<PinState>),
    Repeated {
        size: u8,
        state: PinState,
    },
}

pub struct PinVecIter<'a> {
    vec: &'a PinVec,
    pos: i8
}

impl PinVec {
    pub fn new(size: u8, val: PinState) -> PinVec {
        if size == 1 {
            PinVec::SinglePin(val)
        } else {
            PinVec::Repeated {
                size,
                state: val
            }
        }
    }

    #[inline]
    pub fn init_logical(size: u8, val: u32) -> PinVec {
        if size == 1 {
            PinVec::SinglePin(if val.bit(0) {PinState::High} else {PinState::Low})
        } else {
            PinVec::SmallLogical { size, bits: val }
        }
    }

    #[inline]
    pub fn len(&self) -> u8 {
        match self {
            PinVec::SinglePin(_) => 1,
            PinVec::SmallLogical { size, .. } => *size,
            //PinVec::Big(v) => v.len() as u8,
            PinVec::Repeated { size, .. } => *size,
        }
    }

    #[inline]
    pub fn iter(&self) -> PinVecIter {
        PinVecIter { vec: self, pos: self.len() as i8 - 1 }
    }
}

impl Iterator for PinVecIter<'_> {
    type Item = PinState;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vec {
            PinVec::SinglePin(pin) => {
                if self.pos == 0 {
                    self.pos -= 1;
                    Some(*pin)
                } else {None}
            },
            PinVec::SmallLogical { bits, .. } => {
                if self.pos >= 0 {
                    if bits.bit(self.pos as usize) {
                        self.pos -= 1;
                        Some(PinState::High)
                    } else {
                        self.pos -= 1;
                        Some(PinState::Low)
                    }
                } else {None}
            },
            // PinVec::Big(v) => {
            //     if self.pos >= 0 {
            //         let r = v[self.pos as usize];
            //         self.pos -= 1;
            //         Some(r)
            //     } else {None}
            // },
            PinVec::Repeated {state, .. } => {
                if self.pos >= 0 {
                    self.pos -= 1;
                    Some(*state)
                } else {None}
            },
        }
    }
}

/// Can be converted to a vector of [PinState].
/// 
/// This is used for VCD signals.
pub trait PinStateConvertible {
    fn to_pin_vec(self) -> PinVec;
}

impl PinStateConvertible for bool {
    fn to_pin_vec(self) -> PinVec {
        PinVec::SinglePin(PinState::from_bool(self))
    }
}

impl PinStateConvertible for u8 {
    fn to_pin_vec(self) -> PinVec {
        PinVec::SmallLogical { size: 8, bits: self as u32 }
    }
}

impl PinStateConvertible for u16 {
    fn to_pin_vec(self) -> PinVec {
        PinVec::SmallLogical { size: 16, bits: self as u32 }
    }
}

impl PinStateConvertible for u32 {
    fn to_pin_vec(self) -> PinVec {
        PinVec::SmallLogical { size: 32, bits: self as u32 }
    }
}

impl PinStateConvertible for PinState {
    fn to_pin_vec(self) -> PinVec {
        PinVec::SinglePin(self)
    }
}

impl PinStateConvertible for PinVec {
    fn to_pin_vec(self) -> PinVec {
        self
    }
}