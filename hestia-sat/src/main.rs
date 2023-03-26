use std::iter::zip;
use uts_api::I2C_BUS2;
use uts_api::board::Board;

pub fn main() {
    println!("Hello, world!");
    let board = Board::init(I2C_BUS2);
    for (s, t) in zip(&board.sensors, &board.read_temps()) {
        match t {
            Ok(temp) => println!("{}: {}", s.id, temp),
            Err(e) => panic!("{}: failed with {:?}", s.id, e),
        }
    }
}
