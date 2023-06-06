use amber::{board::Board, mcu::avr::{mcu_ticker::McuTicker, mcu_model::Atmega2560}};

fn main() {
    let mut board = Board::new();
    let mcu = McuTicker::<Atmega2560, _>::new();
    let mcu = board.add_component(mcu);
    board.add_clock_wire(&[(mcu, 0)]);

    for _ in 0..10 {
        board.toggle_clock();
    }
}
