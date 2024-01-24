use std::fmt::Formatter;
use std::fs;
use std::slice::Iter;
use std::sync::Mutex;

use chrono::Duration;
use duration_str::deserialize_duration_chrono;
use lazy_static::lazy_static;
use serde::Deserialize;
use serial_int::SerialGenerator;

use crate::board::BoardId;
use crate::payload::Config;

pub mod runner;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Programs {
    programs: Vec<Program>,

    #[serde(default, alias = "loop")]
    pub run_loop: bool,
}

impl Programs {
    pub fn load(config: &Config) -> Self {
        let file = config.program_file.as_ref()
            .expect("UTS_PROGRAM_FILE should be set to location of uts-programs.toml");
        Self::load_from_file(file)
    }

    pub fn load_from_file(filename: &str) -> Self {
        let str = fs::read_to_string(filename)
            .unwrap_or_else(|err| panic!("Program file should be readable {}: {}", filename, err));
        toml::from_str(&str)
            .unwrap_or_else(|err| panic!("Program file should contain valid TOML {}: {}", filename, err))
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
    pub heat_board: BoardId,
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
