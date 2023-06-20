use std::convert::TryInto;
use std::rc::Rc;
use log::error;

use crate::{ReadResult};
use crate::heater::{Heater, HeaterMode, TargetSensor};
use crate::device::ads7828::Ads7828Sensor;
use crate::device::i2c::I2cBus;
use crate::device::max31725::Max31725Sensor;
use crate::device::msp430::{Msp430, Msp430CurrentSensor, Msp430TempSensor, Msp430VoltageSensor};
use crate::reading::{ReadableSensor, SensorReading};
use crate::sensors::{Sensor, SensorInterface};

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

pub static ALL_SENSORS: &[Sensor; 20] = &[
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
    pub bus: I2cBus,
    pub heater: Rc<dyn Heater>,
    pub sensors: Vec<Box<dyn ReadableSensor>>,
    pub check_sensor: Box<dyn ReadableSensor>,
}

impl Board {
    pub fn init(bus: u8, check_sensor: &String) -> Self {
        let sensors = Board::get_readable_sensors(bus.into(), ALL_SENSORS);
        let check_sensor = Board::sensor_by_id(check_sensor)
            .expect("Check sensor not found");
        let check_sensor = Board::create_sensor(bus.into(), *check_sensor);
        let msp430 = Msp430::new(bus.into());
        Board {
            bus: bus.into(),
            heater: Rc::new(msp430),
            sensors,
            check_sensor,
        }
    }

    fn get_readable_sensors(bus: I2cBus, sensors: &[Sensor]) -> Vec<Box<dyn ReadableSensor>> {
        sensors.iter().map(|s| Board::create_sensor(bus, *s)).collect()
    }

    fn create_sensor(bus: I2cBus, s: Sensor) -> Box<dyn ReadableSensor> {
        let name = s.to_string();
        let reg = s.addr.into();
        match s.iface {
            SensorInterface::MSP430 => Box::new(Msp430TempSensor::new(bus, name, reg)),
            SensorInterface::MSP430Voltage => Box::new(Msp430VoltageSensor::new(bus, name, reg)),
            SensorInterface::MSP430Current => Box::new(Msp430CurrentSensor::new(bus, name, reg)),
            SensorInterface::ADS7828 => Box::new(Ads7828Sensor::new(bus, name, s.addr)),
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
        let sensor = Board::create_sensor(self.bus, sensor);
        sensor.read()
    }

    pub fn write_target_sensor(&self, target_sensor: TargetSensor) {
        self.heater.write_target_sensor(target_sensor)
    }

    pub fn read_heater_duty(&self) -> ReadResult<SensorReading<u16>> {
        self.heater.read_duty()
    }

    pub fn write_heater_duty(&self, pwm_duty_cycle: u16) {
        self.heater.write_duty(pwm_duty_cycle)
    }

    fn sensor_by_id(id: &String) -> Option<&'static Sensor> {
        for sensor in ALL_SENSORS {
            if id.eq_ignore_ascii_case(sensor.id) {
                return Some(sensor);
            }
        }
        None
    }

    fn read_sensors(&self) -> Vec<ReadResult<SensorReading<f32>>> {
        self.sensors.iter()
            .map(|s| s.read())
            .collect::<Vec<ReadResult<SensorReading<f32>>>>()
    }
}

pub trait BoardDataProvider {
    fn read_data(&self) -> Option<BoardData>;
}

impl BoardDataProvider for Board {
    fn read_data(&self) -> Option<BoardData> {
        // fail fast if bus isn't found or check sensor is not readable
        if !self.bus.exists() {
            error!("I2C bus device not found: {}", self.bus);
            return None
        }
        let test_read = self.check_sensor.read();
        if test_read.is_err() {
            error!("Failed to read check sensor {} on I2C bus {}", self.check_sensor, self.bus);
            return None
        }

        let sensors = self.read_sensors();
        let sensors: [_; 20] = sensors.try_into().expect("Oversize");
        return Some(BoardData {
            sensors,
            heater_mode: self.heater.read_mode(),
            target_temp: self.heater.read_target_temp(),
            target_sensor: self.heater.read_target_sensor(),
            heater_duty: self.heater.read_duty(),
        });
    }
}

pub struct BoardData {
    pub sensors: [ReadResult<SensorReading<f32>>; 20],
    pub heater_mode: ReadResult<SensorReading<HeaterMode>>,
    pub target_temp: ReadResult<SensorReading<f32>>,
    pub target_sensor: ReadResult<SensorReading<Sensor>>,
    pub heater_duty: ReadResult<SensorReading<u16>>,
}

impl BoardData {
    pub fn get_raw_data(self) -> [ReadResult<u16>; 24] {
        let readings = &self.sensors;
        let mut result = Vec::with_capacity(24);
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
        ]);
        result.try_into().expect("Sizes didn't match")
    }
}

