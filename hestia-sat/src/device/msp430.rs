use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use log::warn;
use crate::heater::{Heater, HeaterMode};
use crate::device::i2c::*;
use crate::{ReadResult, sensors};
use crate::ReadError::ValueOutOfRange;
use crate::sensors::{adc_val_to_temp, ReadableSensor};

const MSP430_I2C_ADDR: I2cAddr = I2cAddr(0x08);
const MSP430_READ_HEATER_MODE: I2cReg = I2cReg(0x20);
const MSP430_READ_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x21);
const MSP430_READ_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x22);
const MSP430_READ_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x23);
const MSP430_WRITE_HEATER_MODE: I2cReg = I2cReg(0x40);
const MSP430_WRITE_HEATER_TARGET_TEMP: I2cReg = I2cReg(0x41);
const MSP430_WRITE_HEATER_TARGET_SENSOR: I2cReg = I2cReg(0x42);
const MSP430_WRITE_HEATER_PWM_DUTY_CYCLE: I2cReg = I2cReg(0x43);

const MSP430_ADC_RESOLUTION: u16 = 1 << 12;
const MSP430_ADC_V_REF: f32 = 3.35; // measured via multimeter with VCC at 5.0V
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
    fn read_mode(&self) -> ReadResult<HeaterMode> {
        let mode = self.read_mode_raw()?;
        mode.try_into().or_else(|_| {
            warn!("{}: Invalid heater mode: {:?}", self.device, mode);
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

    fn read_duty(&self) -> ReadResult<u8> {
        Ok(self.read_duty_raw()? as u8)
    }

    fn read_duty_raw(&self) -> ReadResult<u16> {
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
    fn read_raw(&self) -> ReadResult<u16> {
        self.device.read_register(self.reg, &*self.name)
    }

    fn read_display(&self) -> ReadResult<f32> {
        let adc_val = self.read_raw()?;
        adc_val_to_temp(adc_val, MSP430_ADC_RESOLUTION)
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
    fn read_raw(&self) -> ReadResult<u16> {
        self.device.read_register(self.reg, &*self.name)
    }

    fn read_display(&self) -> ReadResult<f32> {
        let adc_val = self.read_raw()? as f32;
        Ok(adc_val / (MSP430_ADC_RESOLUTION as f32) * MSP430_ADC_V_REF * MSP430_V_DIVIDER_FACTOR)
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
    fn read_raw(&self) -> ReadResult<u16> {
        self.device.read_register(self.reg, &*self.name)
    }

    fn read_display(&self) -> ReadResult<f32> {
        let adc_val = self.read_raw()? as f32;
        Ok(adc_val / (MSP430_ADC_RESOLUTION as f32) * MSP430_ADC_V_REF)
    }
}
