use std::fmt;
use std::fmt::Formatter;
use log::{debug, warn};
use strum_macros::Display;

use crate::i2c::{i2c_read_u16_be, i2c_read_u16_le, I2cAddr, I2cReg};
use crate::{I2cBus, ReadError, ReadResult};

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
pub(crate) const MSP430_ADC_RESOLUTION: u16 = 1 << 12;
const MSP430_ADC_V_REF: f32 = 3.35; // measured via multimeter with VCC at 5.0V
const MSP430_V_DIVIDER_FACTOR: f32 = 2.0;

// disconnected MSP430 ADC produces low erroneous values
const ADC_MIN_VALUE: u16 = 0x0010;
// ADS ADC can error high, so exclude those values
const ADC_MAX_VALUE: u16 = 0x0FFF;

const ZERO_CELSIUS_IN_KELVIN: f32 = 273.15;
const NB21K00103_REF_TEMP_K: f32 = 25.0 + ZERO_CELSIUS_IN_KELVIN;
const INV_NB21K00103_REF_TEMP_K: f32 = 1.0 / NB21K00103_REF_TEMP_K;
const NB21K00103_B_VALUE: f32 = 3630.0;
const INV_NB21K00103_B_VALUE: f32 = 1.0 / NB21K00103_B_VALUE;

const ADS7828_I2C_ADDR: I2cAddr = I2cAddr(0x4A); // revert to 0x48 for board v1
pub(crate) const ADS7828_ADC_RESOLUTION: u16 = 1 << 12;

const MAX31725_REG_TEMP: I2cReg = I2cReg(0x00);
const MAX31725_CF_LSB: f32 = 0.00390625;

#[derive(Display, Copy, Clone, Debug)]
pub enum SensorInterface {
    MSP430,
    MSP430Voltage,
    MSP430Current,
    ADS7828,
    MAX31725,
}

pub type SensorId = &'static str;

#[derive(Debug, Copy, Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub iface: SensorInterface,
    pub addr: I2cAddr,
    pub label: &'static str,
    pub pos_x: f32,
    pub pos_y: f32,
}

impl fmt::Display for Sensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Sensor {
    pub const fn new(id: SensorId, iface: SensorInterface,
                     addr: u8, label: &'static str,
                     pos_x: f32, pos_y: f32) -> Sensor {
        Sensor { id, iface, addr: I2cAddr(addr), label, pos_x, pos_y }
    }

    /// mounted sensors have no position and a location of "Mounted"
    pub const fn mounted(id: SensorId, iface: SensorInterface,
                         addr: u8) -> Sensor {
        Sensor { id, iface, addr: I2cAddr(addr), label: "Mounted", pos_x: 0.0, pos_y: 0.0 }
    }

    pub fn read_temp(&self, bus: I2cBus) -> ReadResult<f32> {
        match self.iface {
            SensorInterface::MSP430 => self.read_msp430_temp(bus),
            SensorInterface::ADS7828 => self.read_ads7828_temp(bus),
            SensorInterface::MAX31725 => self.read_max31725_temp(bus),
            SensorInterface::MSP430Voltage => self.read_msp430_voltage(bus),
            SensorInterface::MSP430Current => self.read_msp430_current(bus),
        }
    }

    pub fn read_raw(&self, bus: I2cBus) -> ReadResult<u16> {
        match self.iface {
            SensorInterface::MSP430 => self.read_msp430_raw(bus),
            SensorInterface::ADS7828 => self.read_ads7828_raw(bus),
            SensorInterface::MAX31725 => self.read_max31725_raw(bus),
            SensorInterface::MSP430Voltage => self.read_msp430_raw(bus),
            SensorInterface::MSP430Current => self.read_msp430_raw(bus),
        }
    }

    fn read_ads7828_temp(&self, bus: I2cBus) -> ReadResult<f32> {
        match self.read_ads7828_raw(bus) {
            Ok(adc_val) => {
                debug!("i2c{}: Read value <{}> from ADS7828, addr 0x{:02x}",
                    bus.id, adc_val, self.addr.0);
                adc_val_to_temp(adc_val, ADS7828_ADC_RESOLUTION)
            },
            Err(e) => {
                warn!("i2c{}: Could not read ADS7828 sensor 0x{:02x}: {}",
                    bus.id, self.addr.0, e);
                Err(e)
            }
        }
    }

    fn read_ads7828_raw(&self, bus: I2cBus) -> ReadResult<u16> {
        let adc_cmd = adc7828_command(self.addr);
        debug!("i2c{}: Converted addr 0x{:02x} to ADS7828 command: {:b}",
            bus.id, self.addr.0, adc_cmd.0);
        Ok(i2c_read_u16_be(bus, ADS7828_I2C_ADDR, adc_cmd)?)
    }

    fn read_max31725_temp(&self, bus: I2cBus) -> ReadResult<f32> {
        match self.read_max31725_raw(bus) {
            Ok(t) => {
                debug!("i2c{}: Read value <{}> from MAX31725, addr 0x{:02x}",
                    bus.id, t, self.addr.0);
                Ok(f32::from(t as i16) * MAX31725_CF_LSB)
            },
            Err(e) => {
                warn!("i2c{}: Could not read MAX31725 sensor 0x{:02x}: {}",
                    bus.id, self.addr.0, e);
                Err(e)
            }
        }
    }

    fn read_max31725_raw(&self, bus: I2cBus) -> ReadResult<u16> {
        Ok(i2c_read_u16_be(bus, self.addr, MAX31725_REG_TEMP)?)
    }

    fn read_msp430_temp(&self, bus: I2cBus) -> ReadResult<f32> {
        match self.read_msp430_raw(bus) {
            Ok(adc_val) => {
                debug!("i2c{}: Read value <{}> from MSP430, addr 0x{:02x}",
                    bus.id, adc_val, self.addr.0);
                adc_val_to_temp(adc_val, MSP430_ADC_RESOLUTION)
            },
            Err(e) => {
                warn!("i2c{}: Could not read MSP430 input 0x{:02x}: {:?}",
                    bus.id, self.addr.0, e);
                Err(e)
            },
        }
    }

    fn read_msp430_raw(&self, bus: I2cBus) -> ReadResult<u16> {
        let reg = I2cReg(self.addr.0);
        match i2c_read_u16_le(bus, MSP430_I2C_ADDR, reg) {
            Ok(adc_val) => {
                debug!("i2c{}: Read value <{}> from MSP430, addr 0x{:02x}",
                    bus.id, adc_val, self.addr.0);
                Ok(adc_val)
            },
            Err(e) => {
                warn!("i2c{}: Could not read MSP430 input 0x{:02x}: {:?}",
                    bus.id, self.addr.0, e);
                Err(e.into())
            },
        }
    }

    fn read_msp430_voltage(&self, bus: I2cBus) -> ReadResult<f32> {
        let adc_val = self.read_msp430_raw(bus)? as f32;
        Ok(adc_val / (MSP430_ADC_RESOLUTION as f32) * MSP430_ADC_V_REF * MSP430_V_DIVIDER_FACTOR)
    }

    fn read_msp430_current(&self, bus: I2cBus) -> ReadResult<f32> {
        let adc_val = self.read_msp430_raw(bus)? as f32;
        Ok(adc_val / (MSP430_ADC_RESOLUTION as f32) * MSP430_ADC_V_REF)
    }
}

fn adc7828_command(addr: I2cAddr) -> I2cReg {
    // set SD = 1, PD0 = 1 (see ADS7828 datasheet, p11)
    I2cReg(0x84 | (ads7828_channel_select(addr.0) << 4))
}

fn ads7828_channel_select(addr: u8) -> u8 {
    // implement crazy channel select - top bit is odd/even, low bits are floor(addr/2)
    // see ADS7828 datasheet for more details
    ((addr & 0x01) << 2) | (addr >> 1)
}

fn adc_range_check(adc_val: u16) -> ReadResult<u16> {
    if adc_val < ADC_MIN_VALUE || adc_val >= ADC_MAX_VALUE {
        Err(ReadError::ValueOutOfRange)
    } else {
        Ok(adc_val)
    }
}

pub(crate) fn adc_val_to_temp(adc_val: u16, adc_resolution: u16) -> ReadResult<f32> {
    let adc_val = adc_range_check(adc_val);
    Ok(1.0 / (
        INV_NB21K00103_REF_TEMP_K +
            INV_NB21K00103_B_VALUE * f32::ln(adc_resolution as f32 / adc_val? as f32 - 1.0)) -
        ZERO_CELSIUS_IN_KELVIN)
}

pub(crate) fn temp_to_adc_val(temp: f32) -> u16 {
    assert!(temp > -55.0 && temp < 150.0, "temp out of range");
    (MSP430_ADC_RESOLUTION as f32 / (f32::exp((1.0 / (temp + ZERO_CELSIUS_IN_KELVIN) - INV_NB21K00103_REF_TEMP_K) *
                 NB21K00103_B_VALUE) + 1.0)) as u16
}
