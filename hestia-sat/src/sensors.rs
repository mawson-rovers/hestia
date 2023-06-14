use std::fmt;
use std::fmt::Formatter;
use strum_macros::Display;

use crate::device::i2c::*;
use crate::{ReadError, ReadResult};

pub(crate) const MSP430_ADC_RESOLUTION: u16 = 1 << 12;

// disconnected MSP430 ADC produces low erroneous values
const ADC_MIN_VALUE: u16 = 0x0010;
// ADS ADC can error high, so exclude those values
const ADC_MAX_VALUE: u16 = 0x0FFF;

const ZERO_CELSIUS_IN_KELVIN: f32 = 273.15;
const NB21K00103_REF_TEMP_K: f32 = 25.0 + ZERO_CELSIUS_IN_KELVIN;
const INV_NB21K00103_REF_TEMP_K: f32 = 1.0 / NB21K00103_REF_TEMP_K;
const NB21K00103_B_VALUE: f32 = 3630.0;
const INV_NB21K00103_B_VALUE: f32 = 1.0 / NB21K00103_B_VALUE;

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

    /// circuit sensors have no position and a location of "Circuit"
    pub const fn circuit(id: SensorId, iface: SensorInterface,
                         addr: u8) -> Sensor {
        Sensor { id, iface, addr: I2cAddr(addr), label: "Circuit", pos_x: 0.0, pos_y: 0.0 }
    }
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
