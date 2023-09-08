use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use log::{debug, error};
use serde::{Deserialize, Serialize, Serializer};

use crate::{ReadResult};
use crate::csv::CSV_FIELD_COUNT;
use crate::heater::{Heater, HeaterMode, TargetSensor};
use crate::device::ads7828::Ads7828Sensor;
use crate::device::i2c::I2cBus;
use crate::device::max31725::Max31725Sensor;
use crate::device::msp430::{Msp430, Msp430CurrentSensor, Msp430TempSensor, Msp430VoltageSensor};
use crate::reading::{DisabledSensor, ReadableSensor, SensorReading};
use crate::sensors::{Sensor, SensorInterface};


#[derive(Debug, Copy, Clone, Deserialize)]
pub enum BoardVersion {
    V1_1 = 110,
    V2_0 = 200,
    V2_2 = 220,
}

impl BoardVersion {
    fn is_sensor_enabled(&self, sensor: &Sensor) -> bool {
        match self {
            BoardVersion::V1_1 => sensor.id != U4.id,
            BoardVersion::V2_0 => true,
            BoardVersion::V2_2 => true,
        }
    }
}

impl Display for BoardVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardVersion::V1_1 => f.write_str("v1.1"),
            BoardVersion::V2_0 => f.write_str("v2.0"),
            BoardVersion::V2_2 => f.write_str("v2.2"),
        }
    }
}

/// u8 repr corresponds to I2C bus ID (1 = i2c1, 2 = i2c2)
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum BoardId {
    #[serde(alias="top", alias="TOP")]
    Top = 1,

    #[serde(alias="bottom", alias="BOTTOM")]
    Bottom = 2,
}

impl Serialize for BoardId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(match self {
            BoardId::Top => "top",
            BoardId::Bottom => "bottom",
        })
    }
}

impl TryFrom<&str> for BoardId {
    type Error = ();

    fn try_from(value: &str) -> Result<BoardId, Self::Error> {
        match value {
            "1" => Ok(Self::Top),
            "2" => Ok(Self::Bottom),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for BoardId {
    type Error = ();

    fn try_from(value: u8) -> Result<BoardId, Self::Error> {
        match value {
            1 => Ok(Self::Top),
            2 => Ok(Self::Bottom),
            _ => Err(()),
        }
    }
}

impl From<&BoardId> for u8 {
    fn from(value: &BoardId) -> Self {
        match value {
            BoardId::Top => 1,
            BoardId::Bottom => 2,
        }
    }
}

impl Into<I2cBus> for BoardId {
    fn into(self) -> I2cBus {
        I2cBus::from(u8::from(&self))
    }
}

impl Display for BoardId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Top => write!(f, "top"),
            Self::Bottom => write!(f, "bottom"),
        }
    }
}

impl From<&Board> for BoardId {
    fn from(board: &Board) -> Self {
        match board.bus.id {
            1 => BoardId::Top,
            2 => BoardId::Bottom,
            _ => panic!("Unknown board ID: {}", board.bus.id),
        }
    }
}

pub const TH1: Sensor = Sensor::new("TH1", SensorInterface::MSP430, 0x01,
                                "Centre", -42.0135, 43.18);
pub const TH2: Sensor = Sensor::new("TH2", SensorInterface::MSP430, 0x02,
                                "Top-left of heater", -35.7124, 54.61);
pub const TH3: Sensor = Sensor::new("TH3", SensorInterface::MSP430, 0x03,
                                "Bottom-right of heater", -53.88, 33.496);

const U4: Sensor = Sensor::new("U4", SensorInterface::MAX31725, 0x48,
                               "Top-left", -15.976, 75.225);
const U5: Sensor = Sensor::new("U5", SensorInterface::MAX31725, 0x4F,
                               "Top-right", 81.788, 75.692);
const U6: Sensor = Sensor::new("U6", SensorInterface::MAX31725, 0x49,
                               "Bottom-right", -82.296, 12.8535);
const U7: Sensor = Sensor::new("U7", SensorInterface::MAX31725, 0x4B,
                               "Centre", 46.228, 47.752);

const TH4: Sensor = Sensor::new("TH4", SensorInterface::ADS7828, 0x00,
                                "Centre", -45.8705, 43.18);
const TH5: Sensor = Sensor::new("TH5", SensorInterface::ADS7828, 0x01,
                                "Top-right", -77.9814, 75.0769);
const TH6: Sensor = Sensor::new("TH6", SensorInterface::ADS7828, 0x02,
                                "Bottom-left of heater", 33.274, 30.226);

pub const J7: Sensor = Sensor::mounted("J7", SensorInterface::MSP430, 0x04);
pub const J8: Sensor = Sensor::mounted("J8", SensorInterface::MSP430, 0x05);

const J12: Sensor = Sensor::mounted("J12", SensorInterface::ADS7828, 0x03);
const J13: Sensor = Sensor::mounted("J13", SensorInterface::ADS7828, 0x04);
const J14: Sensor = Sensor::mounted("J14", SensorInterface::ADS7828, 0x05);
const J15: Sensor = Sensor::mounted("J15", SensorInterface::ADS7828, 0x06);
const J16: Sensor = Sensor::mounted("J16", SensorInterface::ADS7828, 0x07);

pub const HEATER_V_HIGH: Sensor = Sensor::circuit("heater_v_high", SensorInterface::MSP430Voltage, 0x08);
pub const HEATER_V_LOW: Sensor = Sensor::circuit("heater_v_low", SensorInterface::MSP430Voltage, 0x06);
pub const HEATER_CURR: Sensor = Sensor::circuit("heater_curr", SensorInterface::MSP430Current, 0x07);

pub const SENSOR_COUNT: usize = 20;
pub static ALL_SENSORS: &[Sensor; SENSOR_COUNT] = &[
    TH1,
    TH2,
    TH3,
    U4,
    U5,
    U6,
    U7,
    TH4,
    TH5,
    TH6,
    J7,
    J8,
    J12,
    J13,
    J14,
    J15,
    J16,
    HEATER_V_HIGH,
    HEATER_V_LOW,
    HEATER_CURR,
];

pub struct Board {
    pub id: BoardId,
    pub version: BoardVersion,
    pub bus: I2cBus,
    pub heater: Rc<dyn Heater>,
    pub sensors: Vec<Box<dyn ReadableSensor>>,
}

impl Board {
    pub fn new(id: BoardId, version: BoardVersion) -> Self {
        let sensors = Board::get_readable_sensors(version, id.into(), ALL_SENSORS);
        let msp430 = Msp430::new(id.into());
        Board {
            id,
            version,
            bus: id.into(),
            heater: Rc::new(msp430),
            sensors,
        }
    }

    fn get_readable_sensors(version: BoardVersion, bus: I2cBus, sensors: &[Sensor]) -> Vec<Box<dyn ReadableSensor>> {
        sensors.iter()
            .map(|s| Board::create_sensor(version, bus, *s))
            .collect()
    }

    fn create_sensor(version: BoardVersion, bus: I2cBus, s: Sensor) -> Box<dyn ReadableSensor> {
        let name = s.to_string();
        let reg = s.addr.into();
        if !version.is_sensor_enabled(&s) {
            debug!("Disabling sensor: {}", s);
            return Box::new(DisabledSensor::new(name))
        }
        match s.iface {
            SensorInterface::MSP430 => Box::new(Msp430TempSensor::new(bus, name, reg)),
            SensorInterface::MSP430Voltage => Box::new(Msp430VoltageSensor::new(bus, name, reg)),
            SensorInterface::MSP430Current => Box::new(Msp430CurrentSensor::new(bus, name, reg)),
            SensorInterface::ADS7828 => Box::new(Ads7828Sensor::new(version, bus, name, s.addr)),
            SensorInterface::MAX31725 => Box::new(Max31725Sensor::new(bus, name, s.addr)),
        }
    }

    pub fn read_heater_mode(&self) -> ReadResult<SensorReading<HeaterMode>> {
        self.heater.read_mode()
    }

    pub fn write_heater_mode(&self, mode: HeaterMode) {
        self.heater.write_mode(mode)
    }

    pub fn read_target_temp(&self) -> ReadResult<SensorReading<f32>> {
        self.heater.read_target_temp()
    }

    pub fn write_target_temp(&self, temp: f32) {
        self.heater.write_target_temp(temp)
    }

    pub fn get_target_sensor(&self) -> ReadResult<Sensor> {
        self.heater.read_target_sensor().map(|v| v.display_value)
    }

    pub fn read_target_sensor_temp(&self) -> ReadResult<SensorReading<f32>> {
        let sensor = self.get_target_sensor()?;
        let sensor = Board::create_sensor(self.version, self.bus, sensor);
        sensor.read()
    }

    pub fn write_target_sensor(&self, target_sensor: TargetSensor) {
        self.heater.write_target_sensor(target_sensor)
    }

    pub fn read_heater_duty(&self) -> ReadResult<SensorReading<u16>> {
        self.heater.read_duty()
    }

    pub fn write_heater_duty(&self, pwm_duty_cycle: u16) {
        self.heater.write_duty(pwm_duty_cycle);
    }

    pub fn write_max_temp(&self, temp: f32) {
        self.heater.write_max_temp(temp);
    }

    fn read_sensors(&self) -> Vec<ReadResult<SensorReading<f32>>> {
        self.sensors.iter()
            .map(|s| s.read())
            .collect::<Vec<ReadResult<SensorReading<f32>>>>()
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Board{{bus: {:?}}})", self.bus)
    }
}

pub trait BoardDataProvider {
    fn read_data(&self) -> Option<BoardData>;
}

impl BoardDataProvider for Board {
    fn read_data(&self) -> Option<BoardData> {
        // fail fast if bus isn't found
        if !self.bus.exists() {
            error!("I2C bus device not found: {}", self.bus);
            return None;
        }

        let sensors = self.read_sensors();
        if sensors.iter().all(|rr| rr.is_err()) {
            return None;
        }

        let sensors: [_; 20] = sensors.try_into().expect("invalid sensor reading count");
        return Some(BoardData {
            sensors,
            heater_mode: self.heater.read_mode(),
            target_temp: self.heater.read_target_temp(),
            target_sensor: self.heater.read_target_sensor(),
            heater_duty: self.heater.read_duty(),
            max_temp: self.heater.read_max_temp(),
            flags: self.heater.read_flags(),
        });
    }
}

pub struct BoardData {
    pub sensors: [ReadResult<SensorReading<f32>>; SENSOR_COUNT],
    pub heater_mode: ReadResult<SensorReading<HeaterMode>>,
    pub target_temp: ReadResult<SensorReading<f32>>,
    pub target_sensor: ReadResult<SensorReading<Sensor>>,
    pub heater_duty: ReadResult<SensorReading<u16>>,
    pub max_temp: ReadResult<SensorReading<f32>>,
    pub flags: ReadResult<SensorReading<BoardFlags>>,
}

impl BoardData {
    pub fn get_raw_data(self) -> [ReadResult<u16>; CSV_FIELD_COUNT - 2] {
        let readings = &self.sensors;
        let mut result = Vec::with_capacity(CSV_FIELD_COUNT - 2);
        for reading in readings {
            result.push(match reading {
                Ok(reading) => Ok(reading.raw_value),
                Err(e) => Err(e.clone()),
            });
        }
        result.append(&mut vec![
            self.heater_mode.map(|v| v.raw_value),
            self.target_temp.map(|v| v.raw_value),
            self.target_sensor.map(|v| v.raw_value),
            self.heater_duty.map(|v| v.raw_value),
            self.max_temp.map(|v| v.raw_value),
            self.flags.map(|v| v.raw_value),
        ]);
        result.try_into().expect("Sizes didn't match")
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoardFlags {
    on: bool,
    max_temp: bool,
}

impl Display for BoardFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.on, self.max_temp) {
            (true, true) => write!(f, "ERR_MAX_TEMP"),
            (true, false) => write!(f, "OK"),
            (false, _) => write!(f, "ERR_UNKNOWN"),
        }
    }
}

impl TryFrom<u16> for BoardFlags {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let valid_bits = 0b0011;
        if value & (!valid_bits) > 0 {
            return Err(());
        }
        let (on, max_temp) = (
            value & 0b0001 != 0,
            value & 0b0010 != 0,
        );
        Ok(BoardFlags { on, max_temp })
    }
}

