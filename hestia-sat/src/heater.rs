use std::convert::{TryFrom, TryInto};
use log::{debug, info, warn};

use crate::i2c::*;
use crate::{I2cBus, ReadError, ReadResult};
use crate::ReadError::ValueOutOfRange;

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_PWM_FREQ: I2cReg = I2cReg(0x23);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_PWM_FREQUENCY: I2cReg = I2cReg(0x43);

#[repr(u16)]
pub enum HeaterMode {
    OFF = 0x00,
    /// temperature controlled
    PID = 0x01,
    /// fixed power input
    PWM = 0x02,
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

pub fn is_enabled(bus: &I2cBus) -> bool {
    match read_heater_mode(bus) {
        Ok(mode) => match mode {
            HeaterMode::OFF => false,
            HeaterMode::PID => true,
            HeaterMode::PWM => true,
        },
        Err(_) => false,
    }
}

pub fn enable_heater(bus: &I2cBus) {
    info!("i2c{}: Enabling heater", bus.id);
    let result = i2c_write_u16_le(
        bus, MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode::PWM as u16);
    match result {
        Ok(_) => (),
        Err(e) => warn!("i2c{}: Failed to enable heater: {:?}", bus.id, e),
    };
}

pub fn read_heater_mode(bus: &I2cBus) -> ReadResult<HeaterMode> {
    debug!("i2c{}: Reading heater mode", bus.id);
    let mode = i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE);
    match mode {
        Ok(mode) => match HeaterMode::try_from(mode) {
            Ok(mode) => Ok(mode),
            Err(_) => {
                warn!("i2c{}: Invalid heater mode: {:?}", bus.id, mode);
                Err(ValueOutOfRange)
            }
        },
        Err(e) => {
            warn!("i2c{}: Could not read heater mode from MSP430: {:?}", bus.id, e);
            Err(ReadError::from(e))
        }
    }
}

pub fn read_heater_pwm(bus: &I2cBus) -> ReadResult<u16> {
    debug!("i2c{}: Reading heater power level", bus.id);
    Ok(i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_PWM_FREQ)?)
}