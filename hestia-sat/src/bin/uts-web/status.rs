use std::fmt;
use std::iter::zip;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;
use uts_ws1::board::{Board, BoardData, BoardDataProvider, HEATER_CURR, HEATER_V_HIGH, HEATER_V_LOW};
use uts_ws1::heater::HeaterMode;
use uts_ws1::reading::SensorReading;
use uts_ws1::{board, ReadResult};
use uts_ws1::sensors::Sensor;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SensorInfo {
    id: String,
    label: String,
    iface: String,
    addr: String,
    pos_x: f32,
    pos_y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BoardStatus {
    sensor_info: LinkedHashMap<String, SensorInfo>,

    #[serde(serialize_with = "serialize_sensor_values")]
    sensor_values: LinkedHashMap<String, Option<f32>>,

    heater_mode: Option<HeaterMode>,

    #[serde(serialize_with = "serialize_f32")]
    target_temp: Option<f32>,
    target_sensor: Option<String>,
    heater_duty: Option<u8>,

    #[serde(serialize_with = "serialize_f32")]
    heater_power: Option<f32>,

    #[serde(serialize_with = "serialize_f32")]
    target_sensor_temp: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemStatus(LinkedHashMap<String, Option<BoardStatus>>);

fn serialize_f32<S>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match value {
        Some(value) => serializer.serialize_str(format!("{:0.2}", value).as_str()),
        None => serializer.serialize_none(),
    }
}

fn serialize_sensor_values<S>(values: &LinkedHashMap<String, Option<f32>>, serializer: S)
                              -> Result<S::Ok, S::Error>
    where S: Serializer {
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for (key, value) in values {
        match value {
            Some(value) => map.serialize_entry(key, format!("{:0.2}", value).as_str())?,
            None => map.serialize_entry(key, &None::<String>)?,
        }
    }
    map.end()
}

impl BoardStatus {
    pub fn from_data(board: &Board, data: BoardData) -> Self {
        let mut sensor_values = LinkedHashMap::with_capacity(board.sensors.len());
        for (sensor, value) in zip(board::ALL_SENSORS, data.sensors) {
            sensor_values.insert(String::from(sensor.id), from_reading(value));
        }
        let heater_mode = from_reading(data.heater_mode);
        let heater_duty = from_reading(data.pwm_freq);
        let target_temp = from_reading(data.target_temp);
        let target_sensor = from_reading(data.target_sensor).map(|s| s.to_string());
        let target_sensor_temp = get_sensor_value(&target_sensor, &sensor_values);
        let heater_power = calculate_power(&sensor_values);
        BoardStatus {
            sensor_info: to_sensor_info(board::ALL_SENSORS),
            sensor_values,
            heater_mode,
            target_temp,
            target_sensor,
            target_sensor_temp,
            heater_duty,
            heater_power,
        }
    }
}

fn get_sensor_value(sensor_id: &Option<String>, sensor_values: &LinkedHashMap<String, Option<f32>>) -> Option<f32> {
    match sensor_id {
        None => None,
        Some(sensor_id) => sensor_values.get(sensor_id)?.clone()
    }
}

fn calculate_power(sensor_values: &LinkedHashMap<String, Option<f32>>) -> Option<f32> {
    let v_high: f32 = sensor_values.get(HEATER_V_HIGH.id)?.clone()?;
    let v_low: f32 = sensor_values.get(HEATER_V_LOW.id)?.clone()?;
    let curr: f32 = sensor_values.get(HEATER_CURR.id)?.clone()?;
    let voltage_drop = (v_high - v_low).max(0.0);
    Some(voltage_drop * curr)
}

fn to_sensor_info<const N: usize>(sensors: &[Sensor; N]) -> LinkedHashMap<String, SensorInfo> {
    let mut sensor_info = LinkedHashMap::with_capacity(sensors.len());
    for sensor in sensors {
        sensor_info.insert(sensor.id.to_string(), SensorInfo {
            id: sensor.id.to_string(),
            label: sensor.label.to_string(),
            iface: sensor.iface.to_string(),
            addr: sensor.addr.to_string(),
            pos_x: sensor.pos_x,
            pos_y: sensor.pos_y,
        });
    }
    sensor_info
}

fn from_reading<T>(reading: ReadResult<SensorReading<T>>) -> Option<T>
    where T: fmt::Display {
    reading.ok().map(|v| v.display_value)
}

impl SystemStatus {
    pub fn new() -> Self {
        SystemStatus(LinkedHashMap::new())
    }

    pub fn add(&mut self, board: &Board) {
        let data = board.read_data();
        let data = data.map(|d| BoardStatus::from_data(board, d));
        self.0.insert(board.bus.id.to_string(), data);
    }
}
