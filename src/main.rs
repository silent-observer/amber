use amber::{board::Board, components::{avr::Atmega2560, led::Led}, vcd::config::{VcdConfig}, vcd_config};

fn main() {
    let mut board = Board::new("out.vcd", 16e6);
    let mut mcu = Atmega2560::new();
    mcu.load_flash(&[
        0x9A27, //0x0000: sbi DDRB, 7
        0x9A2F, //0x0001: sbi PORTB, 7
        0x0000, //0x0002: nop
        0x982F, //0x0003: cbi PORTB, 7
        0xCFFC,//0x0004: rjmp 0x0001
    ]);

    let mcu = board.add_component(
        mcu, "mcu", 
        &vcd_config!{
            clk
            pc
        });
    board.add_clock_wire(&[(mcu, 0)]);

    let led = Led::new();
    let led = board.add_component(
        led, "led", 
        &VcdConfig::Enable);
    board.add_wire(&[(mcu, 16), (led, 0)]);

    board.simulate(50);
}
