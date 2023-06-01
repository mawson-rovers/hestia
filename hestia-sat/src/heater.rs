use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use log::{debug, info, warn};

use crate::i2c::*;
use crate::{I2cBus, ReadError, ReadResult};
use crate::ReadError::ValueOutOfRange;
use crate::sensors;

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x21);
const MSP430_READ_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x22);
const MSP430_READ_HEATER_PWM_FREQ: I2cReg = I2cReg(0x23);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x41);
const MSP430_WRITE_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x42);
const MSP430_WRITE_PWM_FREQUENCY: I2cReg = I2cReg(0x43);

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum HeaterMode {
    OFF = 0x00,
    /// temperature controlled
    PID = 0x01,
    /// fixed power input
    PWM = 0x02,
}

impl Display for HeaterMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaterMode::OFF => write!(f, "OFF"),
            HeaterMode::PID => write!(f, "PID"),
            HeaterMode::PWM => write!(f, "PWM"),
        }
    }
}

impl TryFrom<u16> for HeaterMode {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == HeaterMode::OFF as u16 => Ok(HeaterMode::OFF),
            x if x == HeaterMode::PID as u16 => Ok(HeaterMode::PID),
            x if x == HeaterMode::PWM as u16 => Ok(HeaterMode::PWM),
            _ => Err(()),
        }
    }
}

pub fn read_heater_mode(bus: &I2cBus) -> ReadResult<HeaterMode> {
    let mode = read_heater_mode_raw(bus)?;
    HeaterMode::try_from(mode).or_else(|_| {
        warn!("i2c{}: Invalid heater mode: {:?}", bus.id, mode);
        Err(ValueOutOfRange)
    })
}

pub fn read_heater_mode_raw(bus: &I2cBus) -> ReadResult<u16> {
    debug!("i2c{}: Reading heater mode", bus.id);
    let mode = i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE);
    mode.or_else(|e| {
        warn!("i2c{}: Could not read heater mode from MSP430: {:?}", bus.id, e);
        Err(ReadError::from(e))
    })
}

pub fn write_heater_mode(bus: &I2cBus, mode: HeaterMode) {
    info!("i2c{}: Setting heater mode to {}", bus, mode);
    let result = i2c_write_u16_le(bus, MSP430_I2C_ADDR,
                                  MSP430_WRITE_HEATER_MODE, mode as u16);
    if result.is_err() {
        warn!("i2c{}: Failed to set heater mode: {:?}", bus, result.unwrap_err())
    }
}

pub fn read_heater_pwm(bus: &I2cBus) -> ReadResult<u16> {
    debug!("i2c{}: Reading heater power level", bus.id);
    Ok(i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_PWM_FREQ)?)
}

pub fn read_target_temp(bus: &I2cBus) -> ReadResult<f32> {
    sensors::adc_val_to_temp(read_target_temp_raw(bus)?,
                             sensors::MSP430_ADC_RESOLUTION)
}

pub fn write_target_temp(bus: &I2cBus, adc_val: u16) {
    info!("i2c{}: Setting heater target temp to {}", bus, adc_val);
    let result = i2c_write_u16_le(bus, MSP430_I2C_ADDR,
                                  MSP430_WRITE_HEATER_TARGET_TEMP, adc_val);
    if result.is_err() {
        warn!("i2c{}: Failed to set heater target temp: {:?}", bus, result.unwrap_err())
    }
}

pub fn read_target_temp_raw(bus: &I2cBus) -> ReadResult<u16> {
    debug!("i2c{}: Reading heater target temp", bus.id);
    Ok(i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_TARGET_TEMP)?)
}

pub fn read_target_sensor(bus: &I2cBus) -> ReadResult<u16> {
    debug!("i2c{}: Reading heater target sensor", bus.id);
    Ok(i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_TARGET_SENSOR)?)
}

pub fn write_target_sensor(bus: &I2cBus, target_sensor: u8) {
    info!("i2c{}: Setting heater target sensor to {}", bus, target_sensor);
    let result = i2c_write_u16_le(bus, MSP430_I2C_ADDR,
                                  MSP430_WRITE_HEATER_TARGET_SENSOR, target_sensor as u16);
    if result.is_err() {
        warn!("i2c{}: Failed to set heater target sensor: {:?}", bus, result.unwrap_err())
    }
}