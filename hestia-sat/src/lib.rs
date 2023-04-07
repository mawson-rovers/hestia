use std::convert::From;
use std::io;

use cubeos_error::Error;
use failure::Fail;

// public modules
pub mod board;
pub mod sensors;

// private modules
mod heater;
mod i2c;

#[derive(Debug, Copy, Clone)]
pub struct I2cBus {
    pub id: i8,
    pub path: &'static str,
}

pub const I2C_BUS1: I2cBus = I2cBus { id: 1, path: "/dev/i2c-1" };
pub const I2C_BUS2: I2cBus = I2cBus { id: 2, path: "/dev/i2c-2" };

/// Errors reading from the payload - usually can be logged and ignored
#[derive(Debug, Fail)]
pub enum ReadError {
    /// No error
    #[fail(display = "No error")]
    None,
    /// Value out of acceptable range
    #[fail(display = "Value out of range error")]
    ValueOutOfRange,
    /// I2C Error
    #[fail(display = "I2C Error")]
    I2CError(io::Error),
}

/// Convert ReadErrors to cubeos_error::Error::ServiceError(u8)
impl From<ReadError> for Error {
    fn from(e: ReadError) -> Error {
        match e {
            ReadError::None => Error::ServiceError(0),
            ReadError::ValueOutOfRange => Error::ServiceError(1),
            ReadError::I2CError(io) => cubeos_error::Error::from(io),
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(io_err: io::Error) -> ReadError {
        ReadError::I2CError(io_err)
    }
}

pub type ReadResult<T> = Result<T, ReadError>;


