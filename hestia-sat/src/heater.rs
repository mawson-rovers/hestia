use std::convert::{TryFrom, TryInto};
use byteorder::LittleEndian;
use log::{debug, info, warn};

use crate::i2c::*;
use crate::I2cBus;

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_PWM_FREQ: I2cReg = I2cReg(0x23);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_PWM_FREQUENCY: I2cReg = I2cReg(0x43);

#[repr(u16)]
enum HeaterMode {
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
    debug!("Reading heater mode");
    match i2c_read_u16_le(bus, MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE) {
        Ok(mode) => {
            info!("Read heater mode: {:?}", mode);
            match mode.try_into() {
                Ok(HeaterMode::OFF) => false,
                Ok(HeaterMode::PID) => true,
                Ok(HeaterMode::PWM) => true,
                Err(_) => {
                    warn!("Invalid heater mode: {:?}", mode);
                    false
                },
            }
        },
        Err(e) => {
            warn!("Could not read heater mode from MSP430: {:?}", e);
            false
        }
    }
}

pub fn enable_heater(bus: &I2cBus) {
    info!("Enabling heater");
    let result = i2c_write_u16_le(
        bus, MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode::PWM as u16);
    match result {
        Ok(_) => (),
        Err(e) => warn!("Failed to enable heater: {:?}", e),
    };
}
