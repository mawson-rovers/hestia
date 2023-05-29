use serde::Deserialize;
use crate::I2cBus;
use crate::board::Board;

fn default_i2c_bus() -> Vec<u8> {
    return vec![1, 2];
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub log_path: Option<String>,

    #[serde(default="default_i2c_bus")]
    pub i2c_bus: Vec<u8>,

    pub disabled_sensors: Option<Vec<String>>,
}

impl Config {
    pub fn read() -> Config {
        envy::prefixed("UTS_").from_env().unwrap()
    }

    pub fn create_boards(&self) -> Vec<Board> {
        let buses: Vec<I2cBus> = self.i2c_bus.clone().into_iter()
            .map(I2cBus::from).collect();
        buses.iter().map(Board::init).collect()
    }
}
