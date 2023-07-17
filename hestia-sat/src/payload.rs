use std::ops::Index;
use std::slice::Iter;
use dotenv::dotenv;
use log::info;
use serde::Deserialize;
use crate::board::{Board, BoardVersion};

fn default_i2c_bus() -> Vec<u8> { vec![1, 2] }

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
}

pub struct Payload {
    boards: Vec<Board>,
}

impl Payload {
    pub fn create() -> Payload {
        Self::from_config(&Config::read())
    }

    pub fn from_config(config: &Config) -> Payload {
        let mut boards = Vec::with_capacity(2);
        for &bus in config.i2c_bus.iter() {
            boards.push(Board::new(config.board_version, bus));
        }
        info!("Reading from {} {} boards: {:?}", boards.len(), config.board_version,
            boards.iter().map(|b| b.bus.path()).collect::<Vec<String>>());
        Self::from_boards(boards)
    }

    fn from_boards(boards: Vec<Board>) -> Payload {
        Self { boards }
    }

    /// board_id is the I2C bus ID, i.e. 1 or 2
    pub fn single_board(board_id: u8) -> Board {
        let payload = Self::from_config(&Config {
            i2c_bus: vec![board_id],
            ..Config::read()
        });
        payload.into_iter().next().expect("Only one board")
    }

    /// Ignore the I2C_BUS config option and create both boards
    pub fn all_boards() -> Payload {
        Self::from_config(&Config {
            i2c_bus: vec![1, 2],
            ..Config::read()
        })
    }

    pub fn iter(&self) -> Iter<Board> {
        self.boards.iter()
    }
}

impl IntoIterator for Payload {
    type Item = Board;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.boards.into_iter()
    }
}

impl<'a> IntoIterator for &'a Payload {
    type Item = &'a Board;
    type IntoIter = Iter<'a, Board>;

    fn into_iter(self) -> Self::IntoIter {
        self.boards.iter()
    }
}

/// indexes on the I2C bus ID, i.e. 1 or 2
impl Index<u8> for Payload {
    type Output = Board;

    fn index(&self, bus_id: u8) -> &Self::Output {
        self.boards.iter().find(|b| b.bus.id == bus_id)
            .unwrap_or_else(|| panic!("Bus ID not found: {}", bus_id))
    }
}