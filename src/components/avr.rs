//! Module for all AVR MCUs.

use self::mcu_ticker::McuDefault;

pub mod mcu_ticker;
mod regfile;
mod mcu;
mod io_controller;
pub mod mcu_model;
mod sreg;
mod bit_helpers;

pub type Atmega2560 = McuDefault<mcu_model::Atmega2560>;