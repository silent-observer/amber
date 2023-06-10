use amber::{board::Board, components::{avr::Atmega2560, led::Led}, vcd::config::{VcdConfig}, vcd_config};

fn main() {
    let mut board = Board::new("out.vcd", 16e6);
    let mut mcu = Atmega2560::new();
    mcu.load_flash_hex("hex/blink.hex");

    let mcu = board.add_component(
        mcu, "mcu", 
        
        &vcd_config!{
        //    clk,
        //    regs
        //    pc
        });
    board.add_clock_wire(&[mcu.pin("CLK")]);

    let led = Led::new();
    let led = board.add_component(
        led, "led", 
        &VcdConfig::Enable);
    board.add_wire(&[mcu.pin("PB7"), led.pin("LED")]);

    board.simulate(10000000);
}
