use amber::{board::Board, components::{avr::Atmega2560, led::Led}, vcd::config::{VcdConfig}, vcd_config};

#[macro_use]
extern crate timeit;

fn main() {
    let mut board = Board::new("out.vcd", 16e6);
    let mut mcu = Atmega2560::new();
    mcu.load_flash_hex("hex/uart_test.hex");

    let mcu = board.add_component_threadless(
        mcu, "mcu", 
        
        &vcd_config!{
            // clk
            // regs
            // pc
        });
    board.add_clock_wire(&mcu);

    let xck_led = Led::new();
    let xck_led = board.add_component(
        xck_led, "xck_led", 
        &VcdConfig::Enable);

    let tx_led = Led::new();
    let tx_led = board.add_component(
        tx_led, "tx_led", 
        &VcdConfig::Enable);

    board.add_wire(&[mcu.pin("PE2"), xck_led.pin("LED")]);
    board.add_wire(&[mcu.pin("PE1"), tx_led.pin("LED")]);

    timeit_loops!(1, {
        board.simulate(16000000);
    });
}
