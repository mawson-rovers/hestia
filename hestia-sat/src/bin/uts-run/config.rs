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
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    pub heating_time: Duration,
    pub temp_sensor: String,
    pub temp_abort: f32,
    pub thermostat: Option<f32>,
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    pub cooling_time: Duration,
    pub heater_position: HeaterPosition,
    pub heater_duty: f32,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum HeaterPosition {
    #[serde(alias="top", alias="TOP")]
    Top = 1,
    #[serde(alias="bottom", alias="BOTTOM")]
    Bottom = 2,
}

pub fn load() -> Config {
    let str = fs::read_to_string("uts-programs.toml").expect("uts-programs.toml not found");
    toml::from_str(&str).expect("Failed to parse uts-programs.toml")
}
