use serde::Deserialize;
use crate::I2cBus;
use crate::board::Board;

fn default_i2c_bus() -> Vec<I2cBus> {
    return vec![I2cBus::from(1), I2cBus::from(2)];
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub log_path: String,

    #[serde(default="default_i2c_bus")]
    pub i2c_bus: Vec<I2cBus>,

    pub disabled_sensors: Option<Vec<String>>,
}

impl Config {
    pub fn read() -> Config {
        envy::prefixed("UTS_").from_env().unwrap()
    }

    pub fn create_boards(&self) -> Vec<Board> {
        self.i2c_bus.iter().map(Board::init).collect()
    }
}
