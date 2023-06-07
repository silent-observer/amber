pub trait McuModel: Send {
    fn flash_size() -> usize;
    fn rampz_mask() -> u8;
    fn eind_mask() -> u8;
}

pub struct Atmega2560;

impl McuModel for Atmega2560 {
    fn flash_size() -> usize {
        128 * 1024
    }

    fn rampz_mask() -> u8 {
        0x03
    }

    fn eind_mask() -> u8 {
        0x01
    }
}