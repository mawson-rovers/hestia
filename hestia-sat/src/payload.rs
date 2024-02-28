use std::convert::TryFrom;
use std::ops::Index;
use std::slice::Iter;
use dotenv::dotenv;
use log::LevelFilter;
use serde::Deserialize;
use syslog::Facility;
use crate::board::{Board, BoardId, BoardVersion};

fn default_i2c_bus() -> Vec<u8> { vec![1, 2] }

fn default_log_interval() -> u16 { 5 }

fn default_http_port() -> u16 { 5000 }

fn default_board_version() -> BoardVersion { BoardVersion::V2_2 }

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Log file directory
    pub log_path: Option<String>,

    #[serde(default)]
    pub compress_logs: bool,

    // Log file download path
    pub download_path: Option<String>,

    /// List of active I2C bus numbers
    #[serde(default = "default_i2c_bus")]
    pub i2c_bus: Vec<u8>,

    /// Board version, used for switching some address settings
    #[serde(default = "default_board_version")]
    pub board_version: BoardVersion,

    /// Duration between logging output in seconds
    #[serde(default = "default_log_interval")]
    pub log_interval: u16,

    /// Location of program config file, e.g. /home/debian/uts/uts-programs.toml
    pub program_file: Option<String>,

    /// Port used for HTTP dashboard
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Enable CORS for remote API access, defaults to false
    #[serde(default)]
    pub cors_enable: bool,
    
    /// Installation directory, used for uts-update
    pub install_path: Option<String>,

    /// Send error logging to syslog instead of console
    #[serde(default)]
    pub syslog: bool,
}

impl Config {
    pub fn read() -> Config {
        dotenv().ok();
        let config: Config = envy::prefixed("UTS_").from_env().unwrap();
        if config.syslog {
            syslog::init(Facility::LOG_USER, LevelFilter::Info, None)
                .expect("Failed to initialise syslog");
            eprintln!("Sending log output to syslog");
        } else {
            let _ = env_logger::try_init(); // don't fail if called multiple times
        }
        config
    }
}

#[derive(Debug)]
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
            if let Ok(id) = BoardId::try_from(bus) {
                boards.push(Board::new(id, config.board_version));
            } else {
                panic!("Configured with unknown board ID: {}", bus);
            }
        }
        Self::from_boards(boards)
    }

    fn from_boards(boards: Vec<Board>) -> Payload {
        Self { boards }
    }

    /// board_id is the I2C bus ID, i.e. 1 or 2
    pub fn single_board(board_id: u8) -> Payload {
        Self::from_config(&Config {
            i2c_bus: vec![board_id],
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

/// Indexes on the I2C bus ID, i.e. 1 or 2. Panics if no matching board found.
impl Index<u8> for Payload {
    type Output = Board;

    fn index(&self, bus_id: u8) -> &Self::Output {
        self.boards.iter().find(|b| b.bus.id == bus_id)
            .unwrap_or_else(|| panic!("Bus ID not found: {}", bus_id))
    }
}

/// Returns the first board if None, otherwise indexes on the I2C bus ID, i.e. 1 or 2.
/// Panics if no matching board found.
impl Index<Option<u8>> for Payload {
    type Output = Board;

    fn index(&self, board_id: Option<u8>) -> &Self::Output {
        match board_id {
            Some(board_id) => {
                &self[board_id]
            },
            None => {
                assert!(self.iter().len() <= 1, "Multiple boards found, use -b or specify board");
                self.iter().next().expect("No boards configured")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, BoardId, BoardVersion};
    use crate::payload::Payload;

    #[test]
    #[should_panic(expected = "No boards configured")]
    fn test_payload_index_option_none_fails_when_empty() {
        let payload = Payload::from_boards(vec![]);
        let _ = payload[None];
    }

    #[test]
    #[should_panic(expected = "Bus ID not found: 1")]
    fn test_payload_index_option_some_fails_when_empty() {
        let payload = Payload::from_boards(vec![]);
        let _ = payload[Some(1)];
    }

    #[test]
    fn test_payload_index_option() {
        let top = Board::new(BoardId::Top, BoardVersion::V2_2);
        let payload = Payload::from_boards(vec![top.clone()]);
        assert_eq!(top, payload[Some(1)]);
        assert_eq!(top, payload[None]);
    }

    #[test]
    #[should_panic(expected = "Bus ID not found: 2")]
    fn test_payload_index_option_invalid_index_top() {
        let top = Board::new(BoardId::Top, BoardVersion::V2_2);
        let payload = Payload::from_boards(vec![top]);
        let _ = payload[Some(2)]; // should panic
    }

    #[test]
    #[should_panic(expected = "Bus ID not found: 1")]
    fn test_payload_index_option_invalid_index_bottom() {
        let bottom = Board::new(BoardId::Bottom, BoardVersion::V2_2);
        let payload = Payload::from_boards(vec![bottom]);
        let _ = payload[Some(1)]; // should panic
    }

    #[test]
    fn test_payload_index_with_multiple() {
        let top = Board::new(BoardId::Top, BoardVersion::V2_2);
        let bottom = Board::new(BoardId::Bottom, BoardVersion::V2_2);
        let payload = Payload::from_boards(vec![top.clone(), bottom.clone()]);
        assert_eq!(top, payload[Some(1)]);
        assert_eq!(bottom, payload[Some(2)]);
    }

    #[test]
    #[should_panic(expected = "Multiple boards found, use -b or specify board")]
    fn test_payload_index_none_with_multiple_fails() {
        let top = Board::new(BoardId::Top, BoardVersion::V2_2);
        let bottom = Board::new(BoardId::Bottom, BoardVersion::V2_2);
        let payload = Payload::from_boards(vec![top.clone(), bottom.clone()]);
        let _ = payload[None];
    }
}