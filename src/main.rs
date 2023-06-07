use amber::{board::Board, components::{avr::{mcu_ticker::McuTicker, mcu_model::Atmega2560}, led::Led}};

fn main() {
    let mut board = Board::new();
    let mut mcu = McuTicker::<Atmega2560, _>::new();
    mcu.load_flash(&[
        0x9A27, //0x0000: sbi DDRB, 7
        0x9A2F, //0x0001: sbi PORTB, 7
        0x0000, //0x0002: nop
        0x982F, //0x0003: cbi PORTB, 7
        0xCFFC,//0x0004: rjmp 0x0001
    ]);

    let mcu = board.add_component(mcu);
    board.add_clock_wire(&[(mcu, 0)]);

    let led = Led::new();
    let led = board.add_component(led);
    board.add_wire(&[(mcu, 16), (led, 0)]);

    for _ in 0..50 {
        board.toggle_clock();
    }
}
