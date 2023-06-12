use chrono::{DateTime, Local, TimeZone, Utc};
use linked_hash_map::LinkedHashMap;
use std::collections::LinkedList;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use crate::status::{BoardStatus, SystemStatus};

#[derive(Debug, Clone)]
pub struct TimeTempData {
    timestamp: DateTime<Local>,
    temp: String,
}

impl Serialize for TimeTempData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.timestamp.format("%Y-%m-%d %T.%6f").to_string())?;
        seq.serialize_element(&self.temp)?;
        seq.end()
    }
}

impl TimeTempData {
    pub fn new(timestamp: DateTime<Utc>, temp: String) -> Self {
        let timestamp = Local.from_utc_datetime(&timestamp.naive_local());
        Self { timestamp, temp }
    }

    pub fn new_f32(timestamp: DateTime<Utc>, temp: f32) -> Self {
        Self::new(timestamp, format!("{:0.2}", temp))
    }

    fn singleton(timestamp: DateTime<Utc>, temp: Option<f32>) -> LinkedList<Self> {
        match temp {
            None => LinkedList::new(),
            Some(temp) => LinkedList::from([
                Self::new_f32(timestamp, temp)
            ]),
        }
    }
}

#[derive(Debug, Clone)]
struct BoardTimeTempData(LinkedHashMap<String, LinkedList<TimeTempData>>);

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

    pub fn add(&mut self, sensor_id: String, data: TimeTempData) {
        let entry = self.0.entry(sensor_id).or_insert(LinkedList::new());
        entry.push_back(data);
    }

    fn from(timestamp: DateTime<Utc>, status: BoardStatus) -> Self {
        let mut result = LinkedHashMap::<String, LinkedList<TimeTempData>>::with_capacity(
            status.sensor_values.len());
        for (sensor_id, value) in status.sensor_values {
            result.insert(sensor_id.clone(), TimeTempData::singleton(timestamp, value));
        }
        result.insert("target_temp".into(), TimeTempData::singleton(timestamp, status.target_temp));
        result.insert("heater_power".into(), TimeTempData::singleton(timestamp, status.heater_power));
        BoardTimeTempData(result)
    }
}

#[derive(Debug, Clone)]
pub struct SystemTimeTempData(LinkedHashMap<String, BoardTimeTempData>);

impl Serialize for SystemTimeTempData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        self.0.serialize(serializer)
    }
}

impl From<SystemStatus> for SystemTimeTempData {
    fn from(status: SystemStatus) -> Self {
        let timestamp = Utc::now();
        let mut result = LinkedHashMap::<String, BoardTimeTempData>::with_capacity(status.0.len());
        for (board_id, board_status) in status.0 {
            result.insert(board_id, match board_status {
                Some(board_status) => BoardTimeTempData::from(timestamp, board_status),
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

    pub fn add(&mut self, board_id: String, sensor_id: String, data: TimeTempData) {
        let entry = self.0.entry(board_id).or_insert(BoardTimeTempData::new());
        entry.add(sensor_id, data);
    }
}
