use std::convert::TryInto;
use log::{debug, info, warn};
use crate::heater::{Heater, HeaterMode};
use crate::i2c::{I2cAddr, I2cDevice, I2cReadWrite, I2cReg};
use crate::{I2cBus, ReadError, ReadResult, sensors};
use crate::ReadError::ValueOutOfRange;

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x21);
const MSP430_READ_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x22);
const MSP430_READ_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x23);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x41);
const MSP430_WRITE_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x42);
const MSP430_WRITE_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x43);

#[derive(Debug, Clone, Copy)]
pub struct Msp430 {
    i2c: I2cDevice,
}

impl std::fmt::Display for Msp430 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/msp430", self.i2c)
    }
}

impl Msp430 {
    pub fn new(bus: I2cBus) -> Self {
        Msp430 { i2c: I2cDevice::LittleEndian { bus } }
    }

    fn read_register(&self, reg: I2cReg, desc: &str) -> Result<u16, ReadError> {
        debug!("{}: Reading {}", self, desc);
        let result = self.i2c.read_u16(MSP430_I2C_ADDR, reg);
        result.or_else(|e| {
            warn!("{}: Could not read {}: {:?}", self, desc, e);
            Err(e.into())
        })
    }

    fn write_register(&self, reg: I2cReg, desc: &str, value: u16) {
        info!("{}: Setting {} to {}", self, desc, value);
        let result = self.i2c.write_u16(MSP430_I2C_ADDR, reg, value as u16);
        if result.is_err() {
            warn!("{}: Failed to set {}: {:?}", self, desc, result.unwrap_err())
        }
    }
}

impl Heater for Msp430 {
    fn read_mode(&self) -> ReadResult<HeaterMode> {
        let mode = self.read_mode_raw()?;
        mode.try_into().or_else(|_| {
            warn!("{}: Invalid heater mode: {:?}", self, mode);
            Err(ValueOutOfRange)
        })
    }

    fn read_mode_raw(&self) -> ReadResult<u16> {
        self.read_register(MSP430_READ_HEATER_MODE, "heater mode")
    }

    fn write_mode(&self, mode: HeaterMode) {
        self.write_register(MSP430_WRITE_HEATER_MODE, "heater mode",
                            mode as u16)
    }

    fn read_duty(&self) -> ReadResult<u16> {
        self.read_register(MSP430_READ_HEATER_PWM_DUTY_CYCLE, "PWM duty")
    }

    fn write_duty(&self, duty: u8) {
        self.write_register(MSP430_WRITE_HEATER_PWM_DUTY_CYCLE, "PWM duty",
                            duty as u16)
    }

    fn read_target_temp(&self) -> ReadResult<f32> {
        sensors::adc_val_to_temp(self.read_target_temp_raw()?,
                                 sensors::MSP430_ADC_RESOLUTION)
    }

    fn read_target_temp_raw(&self) -> ReadResult<u16> {
        self.read_register(MSP430_READ_HEATER_TARGET_TEMP, "target temp")
    }

    fn write_target_temp(&self, temp: f32) {
        let adc_val = sensors::temp_to_adc_val(temp);
        self.write_register(MSP430_WRITE_HEATER_TARGET_TEMP, "target temp", adc_val)
    }

    fn read_target_sensor(&self) -> ReadResult<u16> {
        self.read_register(MSP430_READ_HEATER_TARGET_SENSOR, "target sensor")
    }

    fn write_target_sensor(&self, target_sensor: u8) {
        self.write_register(MSP430_WRITE_HEATER_TARGET_SENSOR, "target sensor",
                            target_sensor as u16)
    }
}
