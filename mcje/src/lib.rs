#![feature(array_try_from_fn)]
#![feature(specialization)]

extern crate core;

use thiserror::Error;

pub use iokum_mcje_derive::{Decode, Encode};

pub mod codec;
pub mod packet;
pub mod types;

#[derive(Error, Debug)]
pub enum Error {
    // delegated marshalling errors
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Json error")]
    Json(#[from] serde_json::Error),
    #[error("NBT error")]
    Nbt(#[from] iokum_nbt::error::Error),

    // marshalling errors
    #[error("Invalid length")]
    InvalidLength,
    #[error("Unknown variant: {0}")]
    UnknownVariant(i32),

    // processing errors
    #[error("Unexpected end")]
    UnexpectedEnd,
    #[error("Unexpected")]
    Unexpected,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Encode {
    fn encode(&self, output: &mut impl std::io::Write) -> Result<()>;
}

pub trait Decode<'a>: Sized {
    fn decode(input: &mut &'a [u8]) -> Result<Self>;
}
