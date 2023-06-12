use thiserror::Error;

pub mod blte;
pub mod casc;
pub mod tact;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Unsupported")]
    Unsupported,
    #[error("Unknown encoding mode: {0}")]
    UnknownEncodingMode(char),
    #[error("Unknown encryption mode: {0}")]
    UnknownEncryptionMode(char),
    #[error("Integrity error")]
    IntegrityError,

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
