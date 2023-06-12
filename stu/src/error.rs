use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("invalid data")]
    InvalidData,
    #[error("invalid length {length}, expected {expected}")]
    InvalidLength { length: u32, expected: u32 },
    #[error("integrity error")]
    IntegrityError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}
