use std::fmt::Formatter;
use std::fs;
use std::slice::Iter;
use chrono::Duration;
use serde::Deserialize;
use duration_str::deserialize_duration_chrono;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Config {
    programs: Vec<Program>,
}

impl Config {
    pub fn programs(&self) -> Iter<Program> {
        self.programs.iter()
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Program {
    pub heat_board: HeatBoard,
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    pub heat_time: Duration,
    pub heat_duty: f32,
    pub temp_sensor: String,
    pub temp_abort: f32,
    pub thermostat: Option<f32>,
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    pub cool_time: Duration,
}

/// u8 repr corresponds to index into boards array (0 = i2c1, 1 = i2c2)
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

pub fn load() -> Config {
    let str = fs::read_to_string("uts-programs.toml").expect("uts-programs.toml not found");
    toml::from_str(&str).expect("Failed to parse uts-programs.toml")
}
