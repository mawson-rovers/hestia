use std::fmt;

use serde::Serialize;

use crate::ReadResult;

#[derive(Debug, Copy, Clone, Serialize)]
pub struct SensorReading<T>
    where T: fmt::Display {
    pub raw_value: u16,
    pub display_value: T,
}

impl<T> SensorReading<T>
    where T: fmt::Display {
    pub fn new(raw_value: u16, display_value: T) -> Self {
        SensorReading { raw_value, display_value }
    }
}

impl<T> From<SensorReading<T>> for u16
    where T: fmt::Display {
    fn from(value: SensorReading<T>) -> Self {
        value.raw_value
    }
}

impl<T> fmt::Display for SensorReading<T>
    where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_value.fmt(f)
    }
}

pub trait ReadableSensor: fmt::Display {
    fn read(&self) -> ReadResult<SensorReading<f32>>;
}
