use std::convert::From;
use std::io;
use std::path::Path;

use cubeos_error::Error;
use failure::Fail;
use serde::{Deserialize, Serialize};

// public modules
pub mod board;
pub mod sensors;
pub mod config;

// private modules
mod heater;
mod i2c;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct I2cBus {
    pub id: u8,
}

impl From<u8> for I2cBus {
    fn from(id: u8) -> Self {
        I2cBus { id }
    }
}

impl I2cBus {
    pub fn path(&self) -> String {
        format!("/dev/i2c-{}", self.id)
    }
    
    pub fn exists(&self) -> bool {
        Path::new(&self.path()).exists()
    }
}

pub const I2C_BUS1: I2cBus = I2cBus { id: 1 };
pub const I2C_BUS2: I2cBus = I2cBus { id: 2 };

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


