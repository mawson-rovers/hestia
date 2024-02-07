use std::convert::From;
use std::sync::Arc;

// use cubeos_error::Error;
use failure::Fail;
use crate::csv::CsvData;

// public modules
pub mod board;
pub mod payload;
pub mod csv;
pub mod heater;
pub mod host;
pub mod logger;
pub mod reading;
pub mod sensors;
pub mod programs;
pub mod zipper;

// private modules
mod device;

/// Errors reading from the payload - usually can be logged and ignored
#[derive(Debug, Fail, Clone)]
pub enum ReadError {
    /// No error
    #[fail(display = "No error")]
    None,

    /// Value out of acceptable range
    #[fail(display = "Value out of range error")]
    ValueOutOfRange,

    /// I2C Error
    #[fail(display = "I2C Error")]
    I2CError(Arc<std::io::Error>),

    /// Sensor is disabled
    #[fail(display = "Disabled")]
    Disabled,
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
        ReadError::I2CError(Arc::new(io_err))
    }
}

impl PartialEq for ReadError {
    /// Type-based equality for ReadError, only used for testing
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReadError::None, ReadError::None) => true,
            (ReadError::ValueOutOfRange, ReadError::ValueOutOfRange) => true,
            (ReadError::I2CError(_), ReadError::I2CError(_)) => true,
            (ReadError::Disabled, ReadError::Disabled) => true,
            (_, _) => false,
        }
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

impl<T> From<&ReadResult<T>> for CsvData
    where CsvData: From<T>,
          T: Copy {
    fn from(value: &ReadResult<T>) -> Self {
        match value {
            Ok(value) => (*value).into(),
            Err(_) => CsvData::Error
        }
    }
}
