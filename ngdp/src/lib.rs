use std::io::Read;

use byteorder::ReadBytesExt;
use md5::{Digest, Md5};
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

fn read_asciiz<R: Read>(input: &mut R) -> Result<String> {
    let mut data = Vec::new();
    loop {
        let value = input.read_u8()?;
        if value == 0 {
            break;
        }
        data.push(value as char);
    }
    Ok(data.iter().collect())
}

fn md5(value: impl AsRef<[u8]>) -> u128 {
    let mut md5 = Md5::new();
    md5.update(value);
    u128::from_be_bytes(md5.finalize().try_into().unwrap())
}
