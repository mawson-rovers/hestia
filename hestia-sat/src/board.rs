use std::convert::TryInto;
use std::rc::Rc;
use log::error;

use crate::{ReadError, ReadResult};
use crate::heater::{Heater, HeaterMode};
use crate::device::ads7828::Ads7828Sensor;
use crate::device::i2c::I2cBus;
use crate::device::max31725::Max31725Sensor;
use crate::device::msp430::{Msp430, Msp430CurrentSensor, Msp430TempSensor, Msp430VoltageSensor};
use crate::sensors::{ReadableSensor, Sensor, SensorDisplayValue, SensorInterface, SensorReading};

const TH1: Sensor = Sensor::new("TH1", SensorInterface::MSP430, 0x01,
                                "Centre", -42.0135, 43.18);
const TH2: Sensor = Sensor::new("TH2", SensorInterface::MSP430, 0x02,
                                "Top-left of heater", -35.7124, 54.61);
const TH3: Sensor = Sensor::new("TH3", SensorInterface::MSP430, 0x03,
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

const J7: Sensor = Sensor::mounted("J7", SensorInterface::MSP430, 0x04);
const J8: Sensor = Sensor::mounted("J8", SensorInterface::MSP430, 0x05);

const J12: Sensor = Sensor::mounted("J12", SensorInterface::ADS7828, 0x03);
const J13: Sensor = Sensor::mounted("J13", SensorInterface::ADS7828, 0x04);
const J14: Sensor = Sensor::mounted("J14", SensorInterface::ADS7828, 0x05);
const J15: Sensor = Sensor::mounted("J15", SensorInterface::ADS7828, 0x06);
const J16: Sensor = Sensor::mounted("J16", SensorInterface::ADS7828, 0x07);

const HEATER_V_HIGH: Sensor = Sensor::mounted("heater_v_high", SensorInterface::MSP430Voltage, 0x08);
const HEATER_V_LOW: Sensor = Sensor::mounted("heater_v_low", SensorInterface::MSP430Voltage, 0x06);
const HEATER_CURR: Sensor = Sensor::mounted("heater_curr", SensorInterface::MSP430Current, 0x07);

static ALL_SENSORS: &[Sensor; 20] = &[
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
    pub fn init(bus: &I2cBus, check_sensor: &String) -> Self {
        let sensors = Board::get_readable_sensors(bus, ALL_SENSORS);
        let check_sensor = Board::sensor_by_id(check_sensor)
            .expect("Check sensor not found");
        let check_sensor = Board::create_sensor(bus, *check_sensor);
        let msp430 = Msp430::new(bus.clone());
        Board {
            bus: bus.clone(),
            heater: Rc::new(msp430),
            sensors,
            check_sensor,
        }
    }

    fn get_readable_sensors(bus: &I2cBus, sensors: &[Sensor]) -> Vec<Box<dyn ReadableSensor>> {
        sensors.iter().map(move |s| Board::create_sensor(bus, *s)).collect()
    }

    fn create_sensor(bus: &I2cBus, s: Sensor) -> Box<dyn ReadableSensor> {
        let bus = bus.clone();
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

    pub fn read_heater_mode(&self) -> ReadResult<HeaterMode> {
        self.heater.read_mode()
    }

    pub fn write_heater_mode(&self, mode: HeaterMode) {
        self.heater.write_mode(mode)
    }

    pub fn read_target_temp(&self) -> ReadResult<f32> {
        self.heater.read_target_temp()
    }

    pub fn write_target_temp(&self, temp: f32) {
        self.heater.write_target_temp(temp)
    }

    pub fn get_target_sensor(&self) -> ReadResult<Sensor> {
        let sensor_id = self.heater.read_target_sensor()?;
        match sensor_id {
            0 => Ok(TH1),
            1 => Ok(TH2),
            2 => Ok(TH3),
            3 => Ok(J7),
            4 => Ok(J8),
            _ => Err(ReadError::ValueOutOfRange),
        }
    }

    pub fn read_target_sensor_temp(&self) -> ReadResult<f32> {
        let sensor = self.get_target_sensor()?;
        let sensor = Board::create_sensor(&self.bus, sensor);
        match sensor.read()?.display_value {
            SensorDisplayValue::F32(temp) => Ok(temp),
        }
    }

    pub fn write_target_sensor(&self, target_sensor: u8) {
        self.heater.write_target_sensor(target_sensor)
    }

    pub fn read_heater_pwm(&self) -> ReadResult<u8> {
        self.heater.read_duty()
    }

    pub fn write_heater_pwm(&self, pwm_duty_cycle: u8) {
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

    fn read_sensors(&self) -> Vec<ReadResult<SensorReading>> {
        self.sensors.iter()
            .map(|s| s.read())
            .collect::<Vec<ReadResult<SensorReading>>>()
    }
}

impl CsvDataProvider for Board {
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
        let mut raw_data: Vec<ReadResult<u16>> = Vec::with_capacity(sensors.len() + 4);
        let mut display_data: Vec<ReadResult<f32>> = Vec::with_capacity(sensors.len());
        for reading in sensors {
            match reading {
                Ok(reading) => {
                    raw_data.push(Ok(reading.raw_value));
                    match reading.display_value {
                        SensorDisplayValue::F32(temp) => display_data.push(Ok(temp)),
                    }
                }
                Err(e) => {
                    raw_data.push(Err(e.clone()));
                    display_data.push(Err(e.clone()));
                }
            }
        }
        raw_data.append(&mut vec![
            self.heater.read_mode_raw(),
            self.heater.read_target_temp_raw(),
            self.heater.read_target_sensor(),
            self.heater.read_duty_raw(),
        ]);
        let raw_data: [_; 24] = raw_data.try_into().expect("Oversize");
        let sensors: [_; 20] = display_data.try_into().expect("Oversize");
        return Some(BoardData {
            raw_data,
            sensors,
            heater_mode: self.heater.read_mode(),
            target_temp: self.heater.read_target_temp(),
            target_sensor: self.get_target_sensor(),
            pwm_freq: self.heater.read_duty(),
        });
    }
}

pub struct BoardData {
    pub raw_data: [ReadResult<u16>; 24],
    pub sensors: [ReadResult<f32>; 20],
    pub heater_mode: ReadResult<HeaterMode>,
    pub target_temp: ReadResult<f32>,
    pub target_sensor: ReadResult<Sensor>,
    pub pwm_freq: ReadResult<u8>,
}

pub trait CsvDataProvider {
    fn read_data(&self) -> Option<BoardData>;
}

