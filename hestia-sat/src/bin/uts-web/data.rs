use std::collections::LinkedList;

use chrono::Local;
use linked_hash_map::LinkedHashMap;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use uts_ws1::board::BoardId;

use uts_ws1::csv::TIMESTAMP_FORMAT;
use uts_ws1::sensors::SensorId;

use crate::status::{BoardStatus, SystemStatus};

#[derive(Debug, Clone)]
pub struct TimeTempData {
    timestamp: String,
    temp: String,
}

impl Serialize for TimeTempData {
    /// Serialize as an array: [timestamp, temp]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.timestamp)?;
        seq.serialize_element(&self.temp)?;
        seq.end()
    }
}

impl TimeTempData {
    pub fn new(timestamp: &str, temp: &str) -> Self {
        Self { timestamp: String::from(timestamp), temp: String::from(temp) }
    }

    pub fn new_f32(timestamp: &str, value: f32) -> Self {
        let value = if value > 10.0 {
            format!("{:0.1}", value)
        } else {
            format!("{:0.2}", value)
        };
        Self::new(timestamp, value.as_str())
    }

    fn singleton(timestamp: &str, temp: Option<f32>) -> LinkedList<Self> {
        match temp {
            None => LinkedList::new(),
            Some(temp) => LinkedList::from([
                Self::new_f32(timestamp, temp)
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoardTimeTempData(pub LinkedHashMap<SensorId, LinkedList<TimeTempData>>);

impl Serialize for BoardTimeTempData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        self.0.serialize(serializer)
    }
}

impl BoardTimeTempData {
    fn new() -> Self {
        BoardTimeTempData(LinkedHashMap::new())
    }

    pub fn add(&mut self, sensor_id: SensorId, data: TimeTempData) {
        let entry = self.0.entry(sensor_id).or_insert(LinkedList::new());
        entry.push_back(data);
        if entry.len() > 1500 {  // include up to 2 hours of data
            entry.pop_front();
        }
    }

    fn from(timestamp: &str, status: BoardStatus) -> Self {
        let mut result = LinkedHashMap::<SensorId, LinkedList<TimeTempData>>::with_capacity(
            status.sensor_values.len());
        for (sensor_id, value) in status.sensor_values {
            result.insert(sensor_id, TimeTempData::singleton(timestamp, value));
        }
        result.insert("target_temp", TimeTempData::singleton(timestamp, status.target_temp));
        result.insert("heater_duty", TimeTempData::singleton(timestamp, status.heater_duty));
        result.insert("heater_power", TimeTempData::singleton(timestamp, status.heater_power));
        BoardTimeTempData(result)
    }
}

#[derive(Debug, Clone)]
pub struct SystemTimeTempData(pub LinkedHashMap<BoardId, BoardTimeTempData>);

impl Serialize for SystemTimeTempData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        self.0.serialize(serializer)
    }
}

impl From<SystemStatus> for SystemTimeTempData {
    fn from(status: SystemStatus) -> Self {
        let timestamp = Local::now().format(TIMESTAMP_FORMAT).to_string();
        let mut result = LinkedHashMap::<BoardId, BoardTimeTempData>::with_capacity(status.0.len());
        for (board_id, board_status) in status.0 {
            result.insert(board_id, match board_status {
                Some(board_status) => BoardTimeTempData::from(&timestamp, board_status),
                None => BoardTimeTempData(LinkedHashMap::new()),
            });
        }
        SystemTimeTempData(result)
    }
}

impl SystemTimeTempData {
    pub fn new() -> Self {
        Self(LinkedHashMap::new())
    }

    pub fn add(&mut self, board_id: BoardId, sensor_id: SensorId, data: TimeTempData) {
        let entry = self.0.entry(board_id).or_insert(BoardTimeTempData::new());
        entry.add(sensor_id, data);
    }
}
