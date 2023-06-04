use std::fmt::{Display, Formatter};
use crate::device::i2c::*;
use crate::ReadResult;
use crate::sensors::{ReadableSensor, SensorReading};

const MAX31725_REG_TEMP: I2cReg = I2cReg(0x00);
const MAX31725_CF_LSB: f32 = 0.00390625;

/// MAX31725 is a discrete I2C temperature sensor on the Hestia boards. Each
/// one has its own configured I2C address on the bus.
#[derive(Debug, Clone)]
pub struct Max31725Sensor {
    name: String,
    device: LoggingI2cDevice
}

impl Max31725Sensor {
    pub fn new(bus: I2cBus, name: String, addr: I2cAddr) -> Self {
        let device = LoggingI2cDevice::new(
            name.clone(),
            I2cDevice::big_endian(bus, addr));
        Max31725Sensor { name, device }
    }
}

impl Display for Max31725Sensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ReadableSensor for Max31725Sensor {
    fn read(&self) -> ReadResult<SensorReading> {
        let raw_value = self.device.read_register(MAX31725_REG_TEMP, "temp")?;
        let display_value = f32::from(raw_value as i16) * MAX31725_CF_LSB;
        Ok(SensorReading::new(raw_value, display_value))
    }
}