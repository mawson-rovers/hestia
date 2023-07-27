use std::fmt::Formatter;
use std::fs;
use std::slice::Iter;
use std::sync::Mutex;
use chrono::Duration;
use serde::Deserialize;
use duration_str::deserialize_duration_chrono;
use lazy_static::lazy_static;
use serial_int::{SerialGenerator};
use uts_ws1::board::Board;
use uts_ws1::payload::Config;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Programs {
    programs: Vec<Program>,

    #[serde(default, alias="loop")]
    pub run_loop: bool,
}

impl Programs {
    pub fn load(config: &Config) -> Self {
        let file = config.program_file.as_ref()
            .expect("Set UTS_PROGRAM_FILE to location of uts-programs.toml");
        let str = fs::read_to_string(&file).expect("uts-programs.toml not found");
        toml::from_str(&str).expect("Failed to parse uts-programs.toml")
    }

    pub fn iter(&self) -> Iter<Program> {
        self.programs.iter()
    }
}

lazy_static! {
    static ref PROGRAM_ID_GEN: Mutex<SerialGenerator<u8>> = Mutex::new(SerialGenerator::new());
}

fn generate_program_id() -> u8 {
    PROGRAM_ID_GEN.lock().unwrap().generate()
}

fn default_heat_duty() -> f32 { 1.0 }

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Program {
    #[serde(default = "generate_program_id")]
    pub id: u8,
    pub name: String,
    pub heat_board: HeatBoard,
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    pub heat_time: Duration,
    #[serde(default = "default_heat_duty")]
    pub heat_duty: f32,
    pub temp_sensor: String,
    pub temp_abort: f32,
    pub thermostat: Option<f32>,
    pub cool_temp: f32,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program {{ id: {}, name: \"{}\" }}", self.id, self.name)
    }
}

/// u8 repr corresponds to I2C bus ID (1 = i2c1, 2 = i2c2)
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Deserialize, Copy)]
pub enum HeatBoard {
    #[serde(alias="top", alias="TOP")]
    Top = 1,
    #[serde(alias="bottom", alias="BOTTOM")]
    Bottom = 2,
}

impl std::fmt::Display for HeatBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HeatBoard::Top => write!(f, "top"),
            HeatBoard::Bottom => write!(f, "bottom"),
        }
    }
}

impl From<&Board> for HeatBoard {
    fn from(board: &Board) -> Self {
        match board.bus.id {
            1 => HeatBoard::Top,
            2 => HeatBoard::Bottom,
            _ => panic!("Unknown board ID: {}", board.bus.id),
        }
    }
}
