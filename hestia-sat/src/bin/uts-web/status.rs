use std::fmt;
use std::iter::zip;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;
use uts_ws1::board::{Board, BoardData, BoardDataProvider, HEATER_CURR, HEATER_V_HIGH, HEATER_V_LOW};
use uts_ws1::heater::{HeaterMode, TargetSensor};
use uts_ws1::reading::SensorReading;
use uts_ws1::{board, ReadResult};
use uts_ws1::config::Config;
use uts_ws1::sensors::{Sensor, SensorInterface};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorInfo {
    id: String,
    label: String,
    unit: String,
    iface: String,
    addr: String,
    pos_x: f32,
    pos_y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct BoardStatus {
    pub sensor_info: LinkedHashMap<String, SensorInfo>,

    #[serde(serialize_with = "serialize_sensor_values")]
    pub sensor_values: LinkedHashMap<String, Option<f32>>,

    pub heater_mode: Option<HeaterMode>,

    #[serde(serialize_with = "serialize_f32")]
    pub target_temp: Option<f32>,
    pub target_sensor: Option<String>,
    pub heater_duty: Option<u8>,

    #[serde(serialize_with = "serialize_f32")]
    pub heater_power: Option<f32>,

    #[serde(serialize_with = "serialize_f32")]
    pub target_sensor_temp: Option<f32>,
}

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
    Some(heater_power(v_high, v_low, curr))
}

pub fn heater_power(v_high: f32, v_low: f32, curr: f32) -> f32 {
    let voltage_drop = (v_high - v_low).max(0.0);
    voltage_drop * curr
}

fn to_sensor_info<const N: usize>(sensors: &[Sensor; N]) -> LinkedHashMap<String, SensorInfo> {
    let mut sensor_info = LinkedHashMap::with_capacity(sensors.len());
    for sensor in sensors {
        sensor_info.insert(sensor.id.to_string(), SensorInfo {
            id: sensor.id.to_string(),
            label: sensor.label.to_string(),
            unit: match sensor.iface {
                SensorInterface::MSP430 |
                SensorInterface::ADS7828 |
                SensorInterface::MAX31725 => String::from("°C"),
                SensorInterface::MSP430Voltage => String::from("V"),
                SensorInterface::MSP430Current => String::from("A"),
            },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SystemStatus(pub LinkedHashMap<String, Option<BoardStatus>>);

impl SystemStatus {
    pub(crate) fn read(config: &Config) -> Self {
        let mut result = SystemStatus::new();
        for board in config.create_boards() {
            result.read_status(&board);
        }
        result
    }

    fn new() -> Self {
        SystemStatus(LinkedHashMap::new())
    }

    fn read_status(&mut self, board: &Board) {
        let data = board.read_data();
        let data = data.map(|d| BoardStatus::from_data(board, d));
        self.0.insert(board.bus.id.to_string(), data);
    }
}

#[derive(Deserialize)]
pub(crate) struct BoardStatusUpdate {
    pub board_id: u8,
    pub heater_mode: Option<HeaterMode>,
    pub heater_duty: Option<u8>,
    pub target_temp: Option<f32>,
    pub target_sensor: Option<TargetSensor>,
}

impl BoardStatusUpdate {
    pub fn apply(&self, board: &Board) {
        if let Some(heater_mode) = self.heater_mode {
            board.write_heater_mode(heater_mode);
        }
        if let Some(heater_duty) = self.heater_duty {
            board.write_heater_pwm(heater_duty);
        }
        if let Some(target_temp) = self.target_temp {
            board.write_target_temp(target_temp);
        }
        if let Some(target_sensor) = self.target_sensor {
            board.write_target_sensor(target_sensor);
        }
    }
}

