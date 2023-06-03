use serde::Deserialize;
use crate::board::Board;
use crate::device::i2c::I2cBus;

fn default_i2c_bus() -> Vec<u8> { vec![1, 2] }

fn default_check_sensor() -> String { "U7".to_string() }

fn default_log_interval() -> u16 { 5 }

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Log file directory
    pub log_path: Option<String>,

    /// List of active I2C bus numbers
    #[serde(default = "default_i2c_bus")]
    pub i2c_bus: Vec<u8>,

    /// Sensor that will be checked for the board to be alive
    #[serde(default = "default_check_sensor")]
    pub check_sensor: String,

    /// Duration between logging output in seconds
    #[serde(default = "default_log_interval")]
    pub log_interval: u16,
}

impl Config {
    pub fn read() -> Config {
        env_logger::init();
        envy::prefixed("UTS_").from_env().unwrap()
    }

    pub fn create_boards(&self) -> Vec<Board> {
        let buses: Vec<I2cBus> = self.i2c_bus.clone().into_iter()
            .map(I2cBus::from).collect();
        buses.iter().map(|bus| Board::init(bus, &self.check_sensor)).collect()
    }
}
