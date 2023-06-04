pub trait McuModel {
    fn flash_size() -> usize;
}

pub struct Atmega2560;

impl McuModel for Atmega2560 {
    fn flash_size() -> usize {
        128 * 1024
    }
}