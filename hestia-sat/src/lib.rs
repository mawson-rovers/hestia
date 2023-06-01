use std::convert::From;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;

// use cubeos_error::Error;
use failure::Fail;
use serde::{Deserialize, Serialize};
use crate::csv::CsvData;

// public modules
pub mod board;
pub mod config;
pub mod csv;
pub mod heater;
pub mod logger;

// private modules
mod msp430;
mod i2c;
mod sensors;


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

impl Display for I2cBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
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
// impl From<ReadError> for Error {
//     fn from(e: ReadError) -> Error {
//         match e {
//             ReadError::None => Error::ServiceError(0),
//             ReadError::ValueOutOfRange => Error::ServiceError(1),
//             ReadError::I2CError(io) => cubeos_error::Error::from(io),
//         }
//     }
// }

impl From<io::Error> for ReadError {
    fn from(io_err: io::Error) -> ReadError {
        ReadError::I2CError(io_err)
    }
}

pub type ReadResult<T> = Result<T, ReadError>;

impl<T> From<ReadResult<T>> for CsvData
    where CsvData: From<T> {
    fn from(value: ReadResult<T>) -> Self {
        match value {
            Ok(value) => value.into(),
            Err(_) => CsvData::Error
        }
    }
}
