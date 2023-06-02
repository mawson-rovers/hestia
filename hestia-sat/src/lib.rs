use std::convert::From;

// use cubeos_error::Error;
use failure::Fail;
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
    I2CError(std::io::Error),
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

impl From<std::io::Error> for ReadError {
    fn from(io_err: std::io::Error) -> ReadError {
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
