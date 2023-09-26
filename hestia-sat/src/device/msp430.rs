use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use log::warn;
use crate::heater::{Heater, HeaterMode, TargetSensor};
use crate::device::i2c::*;
use crate::{ReadResult, sensors};
use crate::board::{TH1, TH2, TH3, J7, J8, BoardFlags};
use crate::ReadError::ValueOutOfRange;
use crate::reading::{ReadableSensor, SensorReading};
use crate::sensors::{adc_val_to_temp, Sensor};

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_VERSION: I2cReg = I2cReg(0x10);
const MSP430_READ_FLAGS: I2cReg = I2cReg(0x11);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x21);
const MSP430_READ_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x22);
const MSP430_READ_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x23);
const MSP430_READ_HEATER_MAX_TEMP: I2cReg = I2cReg(0x24);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x41);
const MSP430_WRITE_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x42);
const MSP430_WRITE_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x43);
const MSP430_WRITE_HEATER_MAX_TEMP: I2cReg = I2cReg(0x44);

const MSP430_ADC_RESOLUTION: u16 = 1 << 12;
const MSP430_ADC_V_REF: f32 = 3.35;
// measured via multimeter with VCC at 5.0V
const MSP430_V_DIVIDER_FACTOR: f32 = 2.0;

/// Texas Instruments MSP430 is the microcontroller for the Hestia board.
///
/// It controls the heater and connects to sensors for temperature, voltage and current
/// which can be read/managed via an I2C interface.
#[derive(Debug, Clone)]
pub struct Msp430 {
    device: LoggingI2cDevice,
}

impl Msp430 {
    pub fn new(bus: I2cBus) -> Self {
        let device = LoggingI2cDevice::new(
            String::from("msp430"),
            I2cDevice::little_endian(bus, MSP430_I2C_ADDR));
        Msp430 { device }
    }

    fn read_register(&self, reg: I2cReg, desc: &str) -> ReadResult<u16> {
        self.device.read_register(reg, desc)
    }

    fn write_register(&self, reg: I2cReg, desc: &str, value: u16) {
        self.device.write_register(reg, desc, value)
    }
}

impl Heater for Msp430 {
    fn read_mode(&self) -> ReadResult<SensorReading<HeaterMode>> {
        let raw = self.read_register(MSP430_READ_HEATER_MODE, "heater mode")?;
        let display: HeaterMode = raw.try_into().map_err(|_| {
            warn!("{}: Invalid heater mode: {:?}", self.device, raw);
            ValueOutOfRange
        })?;
        Ok(SensorReading::new(raw, display))
    }

    fn write_mode(&self, mode: HeaterMode) {
        self.write_register(MSP430_WRITE_HEATER_MODE, "heater mode",
                            mode as u16)
    }

    fn read_duty(&self) -> ReadResult<SensorReading<u16>> {
        let raw_value = self.read_register(MSP430_READ_HEATER_PWM_DUTY_CYCLE, "heater duty")?;
        Ok(SensorReading::new(raw_value, raw_value))
    }

    fn write_duty(&self, duty: u16) {
        self.write_register(MSP430_WRITE_HEATER_PWM_DUTY_CYCLE, "heater duty", duty)
    }

    fn read_target_temp(&self) -> ReadResult<SensorReading<f32>> {
        let raw = self.read_register(MSP430_READ_HEATER_TARGET_TEMP, "target temp")?;
        let display = adc_val_to_temp(raw, sensors::MSP430_ADC_RESOLUTION)?;
        Ok(SensorReading::new(raw, display))
    }

    fn write_target_temp(&self, temp: f32) {
        let adc_val = sensors::temp_to_adc_val(temp);
        self.write_register(MSP430_WRITE_HEATER_TARGET_TEMP, "target temp", adc_val)
    }

    fn read_target_sensor(&self) -> ReadResult<SensorReading<Sensor>> {
        let raw_value = self.read_register(MSP430_READ_HEATER_TARGET_SENSOR, "target sensor")?;
        let display_value = match raw_value {
            0 => TH1,
            1 => TH2,
            2 => TH3,
            3 => J7,
            4 => J8,
            _ => return Err(ValueOutOfRange),
        };
        Ok(SensorReading::new(raw_value, display_value))

    }

    fn write_target_sensor(&self, target_sensor: TargetSensor) {
        self.write_register(MSP430_WRITE_HEATER_TARGET_SENSOR, "target sensor",
                            target_sensor as u16)
    }

    fn read_max_temp(&self) -> ReadResult<SensorReading<f32>> {
        let raw = self.read_register(MSP430_READ_HEATER_MAX_TEMP, "max temp")?;
        let display = adc_val_to_temp(raw, sensors::MSP430_ADC_RESOLUTION)?;
        Ok(SensorReading::new(raw, display))
    }

    fn write_max_temp(&self, temp: f32) {
        let adc_val = sensors::temp_to_adc_val(temp);
        self.write_register(MSP430_WRITE_HEATER_MAX_TEMP, "max temp", adc_val)
    }

    fn read_version(&self) -> ReadResult<SensorReading<String>> {
        let raw = self.read_register(MSP430_READ_VERSION, "version")?;
        let display = format!("{:0.1}", f32::from(raw) / 100.0);
        Ok(SensorReading::new(raw, display))
    }

    fn read_flags(&self) -> ReadResult<SensorReading<BoardFlags>> {
        let raw = self.read_register(MSP430_READ_FLAGS, "flags")?;
        let display: BoardFlags = raw.try_into().map_err(|_| {
            warn!("{}: Invalid flags: {:?}", self.device, raw);
            ValueOutOfRange
        })?;
        Ok(SensorReading::new(raw, display))
    }
}

/// Represents a temperature sensor read via the MSP430 ADC
pub struct Msp430TempSensor {
    device: Msp430,
    name: String,
    reg: I2cReg,
}

impl Msp430TempSensor {
    pub fn new(bus: I2cBus, name: String, reg: I2cReg) -> Self {
        let device = Msp430::new(bus);
        Msp430TempSensor { device, name, reg }
    }
}

impl Display for Msp430TempSensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ReadableSensor for Msp430TempSensor {
    fn read(&self) -> ReadResult<SensorReading<f32>> {
        let raw_value = self.device.read_register(self.reg, &self.name)?;
        let display_value = adc_val_to_temp(raw_value, MSP430_ADC_RESOLUTION)?;
        Ok(SensorReading::new(raw_value, display_value))
    }
}

/// Represents a voltage sensor read via the MSP430 ADC
pub struct Msp430VoltageSensor {
    device: Msp430,
    name: String,
    reg: I2cReg,
}

impl Msp430VoltageSensor {
    pub fn new(bus: I2cBus, name: String, reg: I2cReg) -> Self {
        let device = Msp430::new(bus);
        Msp430VoltageSensor { device, name, reg }
    }
}

impl Display for Msp430VoltageSensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ReadableSensor for Msp430VoltageSensor {
    fn read(&self) -> ReadResult<SensorReading<f32>> {
        let raw_value = self.device.read_register(self.reg, &self.name)?;
        let display_value = raw_value as f32 / (MSP430_ADC_RESOLUTION as f32) *
            MSP430_ADC_V_REF * MSP430_V_DIVIDER_FACTOR;
        Ok(SensorReading::new(raw_value, display_value))
    }
}

/// Represents a current sensor read via the MSP430 ADC
pub struct Msp430CurrentSensor {
    device: Msp430,
    name: String,
    reg: I2cReg,
}

impl Msp430CurrentSensor {
    pub fn new(bus: I2cBus, name: String, reg: I2cReg) -> Self {
        let device = Msp430::new(bus);
        Msp430CurrentSensor { device, name, reg }
    }
}

impl Display for Msp430CurrentSensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ReadableSensor for Msp430CurrentSensor {
    fn read(&self) -> ReadResult<SensorReading<f32>> {
        let raw_value = self.device.read_register(self.reg, &self.name)?;
        let display_value = raw_value as f32 / (MSP430_ADC_RESOLUTION as f32) * MSP430_ADC_V_REF;
        Ok(SensorReading::new(raw_value, display_value))
    }
}
