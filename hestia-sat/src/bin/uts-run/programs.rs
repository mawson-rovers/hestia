use std::fmt::Formatter;
use std::fs;
use std::slice::Iter;
use chrono::Duration;
use serde::Deserialize;
use duration_str::deserialize_duration_chrono;
use uts_ws1::payload::Config;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Programs {
    programs: Vec<Program>,
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
