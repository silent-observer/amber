use std::ops::RangeInclusive;

pub fn bit_field(x: u16, r: &RangeInclusive<usize>) -> u16 {
    (x as u16) << (32 - r.start() - 1) >> (32 - r.start() - 1 + r.end())
}

pub fn bit_field_combined(x: u16, data: &[RangeInclusive<usize>]) -> u16 {
    let mut result = 0;
    let mut offset = 0;
    for r in data.iter().rev() {
        result |= bit_field(x, r) << offset;
        offset += r.start() - r.end() + 1;
    }
    result
}

pub fn get_r_field(opcode: u16, size: usize) -> u16 {
    match size {
        3 => 0x18 | bit_field_combined(opcode, &[2..=0]),
        4 => 0x10 | bit_field_combined(opcode, &[3..=0]),
        5 => bit_field_combined(opcode, &[9..=9, 3..=0]),
        _ => panic!("Invalid R field size"),
    }
}

pub fn get_d_field(opcode: u16, size: usize) -> u16 {
    match size {
        2 => 0x18 | bit_field_combined(opcode, &[5..=4]) << 1,
        3 => 0x18 | bit_field_combined(opcode, &[6..=4]),
        4 => 0x10 | bit_field_combined(opcode, &[7..=4]),
        5 => bit_field_combined(opcode, &[8..=4]),
        _ => panic!("Invalid R field size"),
    }
}

pub fn get_rd_fields(opcode: u16, size: usize) -> (u16, u16) {
    (get_r_field(opcode, size), get_d_field(opcode, size))
}

pub fn get_k6(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[7..=6, 3..=0]) as u8
}

pub fn get_k8(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[11..=8, 3..=0]) as u8
}

pub fn get_io6(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[10..=9, 3..=0]) as u8
}

pub fn get_io5(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[7..=3]) as u8
}

pub fn is_two_word(opcode: u16) -> bool {
    (opcode & 0xFC0F) == 0x9000 ||
    (opcode & 0xFE0C) == 0x940C
}