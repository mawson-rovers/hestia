use std::fmt::{Display, Formatter};
use log::debug;
use crate::board::BoardVersion;
use crate::device::i2c::*;
use crate::reading::{ReadableSensor, SensorReading};
use crate::ReadResult;
use crate::sensors::{adc_val_to_temp};

const ADS7828_I2C_ADDR_V1: I2cAddr = I2cAddr(0x48);
const ADS7828_I2C_ADDR_V2: I2cAddr = I2cAddr(0x4A);
pub(crate) const ADS7828_ADC_RESOLUTION: u16 = 1 << 12;

/// ADS7828 is a discrete multiplexing ADC on the Hestia board.
/// This represents one of the individual sensors on the ADC.
#[derive(Debug, Clone)]
pub struct Ads7828Sensor {
    device: LoggingI2cDevice,
    name: String,
    reg: I2cReg,
}

impl Ads7828Sensor {
    pub fn new(version: BoardVersion, bus: I2cBus, name: String, reg: I2cAddr) -> Self {
        let name = format!("ads7828/{}", name);
        let addr = match version {
            BoardVersion::V1_1 => ADS7828_I2C_ADDR_V1,
            BoardVersion::V2 => ADS7828_I2C_ADDR_V2,
        };
        let device = LoggingI2cDevice::new(
            name.clone(), I2cDevice::big_endian(bus, addr));
        let reg = Self::adc7828_command(reg);
        debug!("{}: Converted addr {} to ADS7828 command: {:b}", name, reg, reg.0);
        Ads7828Sensor { device, name, reg }
    }

    fn adc7828_command(addr: I2cAddr) -> I2cReg {
        // set SD = 1, PD0 = 1 (see ADS7828 datasheet, p11)
        let result = I2cReg(0x84 | (Self::ads7828_channel_select(addr.0) << 4));
        result
    }

    fn ads7828_channel_select(addr: u8) -> u8 {
        // implement crazy channel select - top bit is odd/even, low bits are floor(addr/2)
        // see ADS7828 datasheet for more details
        ((addr & 0x01) << 2) | (addr >> 1)
    }
}

impl Display for Ads7828Sensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ReadableSensor for Ads7828Sensor {
    fn read(&self) -> ReadResult<SensorReading<f32>> {
        let raw_value = self.device.read_register(self.reg, &*self.name)?;
        let display_value = adc_val_to_temp(raw_value, ADS7828_ADC_RESOLUTION)?;
        Ok(SensorReading::new(raw_value, display_value))
    }
}