use std::fmt;
use std::iter::zip;

use linked_hash_map::LinkedHashMap;
use log::error;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;

use uts_ws1::{board, ReadResult};
use uts_ws1::board::{Board, BoardData, BoardDataProvider, BoardId, calc_heater_power};
use uts_ws1::board::{V_CURR_AVG, V_HIGH_AVG, V_LOW_AVG};
use uts_ws1::heater::{HeaterMode, TargetSensor};
use uts_ws1::payload::{Config, Payload};
use uts_ws1::reading::SensorReading;
use uts_ws1::sensors::{Sensor, SensorId, SensorInterface};

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

#[derive(Serialize, Debug, Clone)]
pub(crate) struct BoardStatus {
    pub sensor_info: LinkedHashMap<String, SensorInfo>,

    #[serde(serialize_with = "serialize_sensor_values")]
    pub sensor_values: LinkedHashMap<SensorId, Option<f32>>,

    pub heater_mode: Option<HeaterMode>,

    #[serde(serialize_with = "serialize_f32")]
    pub target_temp: Option<f32>,
    pub target_sensor: Option<SensorId>,

    #[serde(serialize_with = "serialize_f32")]
    pub heater_duty: Option<f32>,

    #[serde(serialize_with = "serialize_f32")]
    pub heater_power: Option<f32>,

    #[serde(serialize_with = "serialize_f32")]
    pub target_sensor_temp: Option<f32>,
}

fn serialize_f32<S>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match value {
        Some(value) => serializer.serialize_str(format_f32_3sd(value).as_str()),
        None => serializer.serialize_none(),
    }
}

fn serialize_sensor_values<S>(values: &LinkedHashMap<SensorId, Option<f32>>, serializer: S)
                              -> Result<S::Ok, S::Error>
    where S: Serializer {
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for (key, value) in values {
        match value {
            Some(value) => {
                map.serialize_entry(key, format_f32_3sd(value).as_str())?
            }
            None => map.serialize_entry(key, &None::<String>)?,
        }
    }
    map.end()
}

fn format_f32_3sd(value: &f32) -> String {
    if *value >= 10.0 {
        format!("{:0.1}", value)
    } else {
        format!("{:0.2}", value)
    }
}

impl BoardStatus {
    pub fn from_data(board: &Board, data: BoardData) -> Self {
        let mut sensor_values = LinkedHashMap::with_capacity(board.sensors.len());
        for (sensor, value) in zip(board::ALL_SENSORS, data.sensors) {
            sensor_values.insert(sensor.id, from_reading(value));
        }
        let heater_mode = from_reading(data.heater_mode);
        let heater_duty = from_reading(data.heater_duty);
        let heater_duty = match (heater_mode, heater_duty) {
            (Some(HeaterMode::PID), Some(duty)) => Some(f32::from(duty) / 1000.0),
            (_, Some(duty)) => Some(f32::from(duty) / 255.0),
            _ => None
        };
        let target_temp = from_reading(data.target_temp).map(|t| t.round());
        let target_sensor = from_reading(data.target_sensor).map(|s| s.id);
        let target_sensor_temp = get_sensor_value(&target_sensor, &sensor_values);
        let heater_power = calculate_power(board, &sensor_values);
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

fn get_sensor_value(sensor_id: &Option<SensorId>, sensor_values: &LinkedHashMap<SensorId, Option<f32>>) -> Option<f32> {
    match sensor_id {
        None => None,
        Some(sensor_id) => *sensor_values.get(sensor_id)?
    }
}

fn calculate_power(board: &Board, sensor_values: &LinkedHashMap<SensorId, Option<f32>>) -> Option<f32> {
    let v_high = (*sensor_values.get(V_HIGH_AVG.id)?)?;
    let v_low = (*sensor_values.get(V_LOW_AVG.id)?)?;
    let v_curr = (*sensor_values.get(V_CURR_AVG.id)?)?;
    Some(calc_heater_power(board.version, v_high, v_low, v_curr))
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

#[derive(Serialize, Debug, Clone)]
pub(crate) struct SystemStatus(pub LinkedHashMap<BoardId, Option<BoardStatus>>);

impl SystemStatus {
    pub(crate) fn read(config: &Config) -> Self {
        let payload = Payload::from_config(config);
        let mut result = SystemStatus::new();
        for board in payload {
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
        self.0.insert(board.id, data);
    }
}

#[derive(Deserialize)]
pub(crate) struct BoardStatusUpdate {
    pub board: BoardId,
    pub heater_mode: Option<HeaterMode>,
    pub heater_duty: Option<u16>,
    pub target_temp: Option<f32>,
    pub target_sensor: Option<TargetSensor>,
}

impl BoardStatusUpdate {
    pub fn apply(&self, payload: &Payload) {
        let board = payload.iter().find(|b| b.bus.id == self.board as u8);
        if let Some(board) = board {
            if let Some(heater_mode) = self.heater_mode {
                if heater_mode != HeaterMode::OFF {
                    for other in payload.iter().filter(|b| b.bus.id != self.board as u8) {
                        other.write_heater_mode(HeaterMode::OFF);
                    }
                }
                board.write_heater_mode(heater_mode);
            } else {
                self.update_board(board);
            }
        } else {
            error!("Board ID not found or configured: {}", self.board);
        }
    }

    fn update_board(&self, board: &Board) {
        if let Some(heater_duty) = self.heater_duty {
            board.write_heater_duty(heater_duty);
        }
        if let Some(target_temp) = self.target_temp {
            board.write_target_temp(target_temp);
        }
        if let Some(target_sensor) = self.target_sensor {
            board.write_target_sensor(target_sensor);
        }
    }
}

