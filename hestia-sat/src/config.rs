use dotenv::dotenv;
use log::info;
use serde::Deserialize;
use crate::board::{Board, BoardVersion};

fn default_i2c_bus() -> Vec<u8> { vec![1, 2] }

fn default_check_sensor() -> String { "U7".to_string() }

fn default_log_interval() -> u16 { 5 }

fn default_http_port() -> u16 { 5000 }

fn default_board_version() -> BoardVersion { BoardVersion::V2 }

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Log file directory
    pub log_path: Option<String>,

    /// List of active I2C bus numbers
    #[serde(default = "default_i2c_bus")]
    pub i2c_bus: Vec<u8>,

    /// Board version, used for switching some address settings
    #[serde(default = "default_board_version")]
    pub board_version: BoardVersion,

    /// Sensor that will be checked for the board to be alive
    #[serde(default = "default_check_sensor")]
    pub check_sensor: String,

    /// Duration between logging output in seconds
    #[serde(default = "default_log_interval")]
    pub log_interval: u16,

    /// Port used for HTTP dashboard
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Enable CORS for remote API access, defaults to false
    #[serde(default)]
    pub cors_enable: bool,
}

impl Config {
    pub fn read() -> Config {
        dotenv().ok();
        let _ = env_logger::try_init(); // don't fail if called multiple times
        envy::prefixed("UTS_").from_env().unwrap()
    }

    pub fn create_boards(&self) -> Vec<Board> {
        let result: Vec<Board> = self.i2c_bus.iter().map(|&bus| {
            Board::init(self.board_version, bus, &self.check_sensor)
        }).collect();
        info!("Reading from {} {} boards: {:?}", result.len(), self.board_version,
            result.iter().map(|b| b.bus.path()).collect::<Vec<String>>());
        result
    }
}
