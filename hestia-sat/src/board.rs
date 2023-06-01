use chrono::{DateTime, Utc};
use log::error;

use crate::{heater, I2cBus, ReadError, ReadResult};
use crate::heater::HeaterMode;
use crate::sensors::{Sensor, SensorInterface};

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

const HEATER_V_LOW: Sensor = Sensor::mounted("heater_v_low", SensorInterface::MSP430Voltage, 0x06);
const HEATER_CURR: Sensor = Sensor::mounted("heater_curr", SensorInterface::MSP430Current, 0x07);
const HEATER_V_HIGH: Sensor = Sensor::mounted("heater_v_high", SensorInterface::MSP430Voltage, 0x08);

static ALL_SENSORS: &[Sensor] = &[
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


pub struct SensorData {
    pub timestamp: DateTime<Utc>,
    pub board_id: u16,
    pub th1: ReadResult<u16>,
    pub th2: ReadResult<u16>,
    pub th3: ReadResult<u16>,
    pub u4: ReadResult<u16>,
    pub u5: ReadResult<u16>,
    pub u6: ReadResult<u16>,
    pub u7: ReadResult<u16>,
    pub th4: ReadResult<u16>,
    pub th5: ReadResult<u16>,
    pub th6: ReadResult<u16>,
    pub j7: ReadResult<u16>,
    pub j8: ReadResult<u16>,
    pub j12: ReadResult<u16>,
    pub j13: ReadResult<u16>,
    pub j14: ReadResult<u16>,
    pub j15: ReadResult<u16>,
    pub j16: ReadResult<u16>,
    pub heater_mode: ReadResult<u16>,
    pub target_temp: ReadResult<u16>,
    pub target_sensor: ReadResult<u16>,
    pub pwm_freq: ReadResult<u16>,
    pub heater_v_high: ReadResult<u16>,
    pub heater_v_low: ReadResult<u16>,
    pub heater_curr: ReadResult<u16>,
}

pub struct DisplayData {
    pub timestamp: DateTime<Utc>,
    pub board_id: u16,
    pub th1: ReadResult<f32>,
    pub th2: ReadResult<f32>,
    pub th3: ReadResult<f32>,
    pub u4: ReadResult<f32>,
    pub u5: ReadResult<f32>,
    pub u6: ReadResult<f32>,
    pub u7: ReadResult<f32>,
    pub th4: ReadResult<f32>,
    pub th5: ReadResult<f32>,
    pub th6: ReadResult<f32>,
    pub j7: ReadResult<f32>,
    pub j8: ReadResult<f32>,
    pub j12: ReadResult<f32>,
    pub j13: ReadResult<f32>,
    pub j14: ReadResult<f32>,
    pub j15: ReadResult<f32>,
    pub j16: ReadResult<f32>,
    pub heater_mode: ReadResult<HeaterMode>,
    pub target_temp: ReadResult<f32>,
    pub target_sensor: ReadResult<Sensor>,
    pub pwm_freq: ReadResult<u16>,
    pub heater_v_high: ReadResult<f32>,
    pub heater_v_low: ReadResult<f32>,
    pub heater_curr: ReadResult<f32>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub bus: I2cBus,
    pub sensors: Vec<Sensor>,
    pub check_sensor: &'static Sensor,
}

impl Board {
    pub fn init(bus: &I2cBus, check_sensor: &String) -> Board {
        let check_sensor = Board::sensor_by_id(check_sensor)
            .expect("Check sensor not found");
        let sensors = Vec::from(ALL_SENSORS);
        Board {
            bus: bus.to_owned(),
            sensors,
            check_sensor,
        }
    }

    pub fn read_heater_mode(&self) -> ReadResult<HeaterMode> {
        heater::read_heater_mode(&self.bus)
    }

    pub fn write_heater_mode(&self, mode: HeaterMode) {
        heater::write_heater_mode(&self.bus, mode)
    }

    pub fn read_target_temp(&self) -> ReadResult<f32> {
        heater::read_target_temp(&self.bus)
    }

    pub fn write_target_temp(&self, temp: f32) {
        heater::write_target_temp(&self.bus, crate::sensors::temp_to_adc_val(temp))
    }

    pub fn get_target_sensor(&self) -> ReadResult<Sensor> {
        let sensor_id = heater::read_target_sensor(&self.bus)?;
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
        sensor.read_temp(&self.bus)
    }

    pub fn write_target_sensor(&self, target_sensor: u8) {
        heater::write_target_sensor(&self.bus, target_sensor)
    }

    pub fn read_heater_pwm(&self) -> ReadResult<u16> {
        heater::read_heater_pwm(&self.bus)
    }

    pub fn write_heater_pwm(&self, pwm_duty_cycle: u8) {
        heater::write_heater_pwm(&self.bus, pwm_duty_cycle)
    }

    fn sensor_by_id(id: &String) -> Option<&'static Sensor> {
        for sensor in ALL_SENSORS {
            if id.eq_ignore_ascii_case(sensor.id) {
                return Some(sensor);
            }
        }
        None
    }

    pub fn read_sensor_data(&self, timestamp: DateTime<Utc>) -> Option<SensorData> {
        // fail fast if bus isn't found or check sensor is not readable
        if !self.bus.exists() {
            error!("I2C bus device not found: {}", self.bus);
            return None
        }
        let test_read = self.check_sensor.read_raw(&self.bus);
        if test_read.is_err() {
            error!("Failed to read check sensor {} on I2C bus {}", self.check_sensor, self.bus);
            return None
        }

        return Some(SensorData {
            timestamp,
            board_id: self.bus.id as u16,
            th1: TH1.read_raw(&self.bus),
            th2: TH2.read_raw(&self.bus),
            th3: TH3.read_raw(&self.bus),
            u4: U4.read_raw(&self.bus),
            u5: U5.read_raw(&self.bus),
            u6: U6.read_raw(&self.bus),
            u7: U7.read_raw(&self.bus),
            th4: TH4.read_raw(&self.bus),
            th5: TH5.read_raw(&self.bus),
            th6: TH6.read_raw(&self.bus),
            j7: J7.read_raw(&self.bus),
            j8: J8.read_raw(&self.bus),
            j12: J12.read_raw(&self.bus),
            j13: J13.read_raw(&self.bus),
            j14: J14.read_raw(&self.bus),
            j15: J15.read_raw(&self.bus),
            j16: J16.read_raw(&self.bus),
            heater_mode: heater::read_heater_mode_raw(&self.bus),
            target_temp: heater::read_target_temp_raw(&self.bus),
            target_sensor: heater::read_target_sensor(&self.bus),
            pwm_freq: heater::read_heater_pwm(&self.bus),
            heater_v_high: HEATER_V_HIGH.read_raw(&self.bus),
            heater_v_low: HEATER_V_LOW.read_raw(&self.bus),
            heater_curr: HEATER_CURR.read_raw(&self.bus),
        });
    }

    pub fn read_display_data(&self, timestamp: DateTime<Utc>) -> Option<DisplayData> {
        // fail fast if bus isn't found or check sensor is not readable
        if !self.bus.exists() {
            error!("I2C bus device not found: {}", self.bus);
            return None
        }
        let test_read = self.check_sensor.read_raw(&self.bus);
        if test_read.is_err() {
            error!("Failed to read check sensor {} on I2C bus {}", self.check_sensor, self.bus);
            return None
        }

        return Some(DisplayData {
            timestamp,
            board_id: self.bus.id as u16,
            th1: TH1.read_temp(&self.bus),
            th2: TH2.read_temp(&self.bus),
            th3: TH3.read_temp(&self.bus),
            u4: U4.read_temp(&self.bus),
            u5: U5.read_temp(&self.bus),
            u6: U6.read_temp(&self.bus),
            u7: U7.read_temp(&self.bus),
            th4: TH4.read_temp(&self.bus),
            th5: TH5.read_temp(&self.bus),
            th6: TH6.read_temp(&self.bus),
            j7: J7.read_temp(&self.bus),
            j8: J8.read_temp(&self.bus),
            j12: J12.read_temp(&self.bus),
            j13: J13.read_temp(&self.bus),
            j14: J14.read_temp(&self.bus),
            j15: J15.read_temp(&self.bus),
            j16: J16.read_temp(&self.bus),
            heater_mode: heater::read_heater_mode(&self.bus),
            target_temp: self.read_target_temp(),
            target_sensor: self.get_target_sensor(),
            pwm_freq: heater::read_heater_pwm(&self.bus),
            heater_v_high: HEATER_V_HIGH.read_temp(&self.bus),
            heater_v_low: HEATER_V_LOW.read_temp(&self.bus),
            heater_curr: HEATER_CURR.read_temp(&self.bus),
        });
    }
}

