use std::fmt::{Display, Formatter};
use crate::device::i2c::*;
use crate::reading::{ReadableSensor, SensorReading};
use crate::ReadResult;

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
    fn read(&self) -> ReadResult<SensorReading<f32>> {
        let raw_value = self.device.read_register(MAX31725_REG_TEMP, "temp")?;
        let display_value = f32::from(raw_value as i16) * MAX31725_CF_LSB;
        Ok(SensorReading::new(raw_value, display_value))
    }
}

#[cfg(test)]
#[cfg(not(target_os = "linux"))]
mod tests {
    use crate::device::i2c::{I2cAddr, I2cBus};
    use crate::device::max31725::Max31725Sensor;
    use crate::reading::ReadableSensor;

    #[test]
    fn test_max31725_temp_conversion() {
        let sensor = Max31725Sensor::new(I2cBus::from(2), String::from("MAX31725"), I2cAddr(0x48));
        assert_eq!(25.5625, sensor.read().unwrap().display_value);
        assert_eq!((25 << 8) + (0x48 << 1), sensor.read().unwrap().raw_value);
    }
}