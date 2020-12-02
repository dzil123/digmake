use super::edres::de::error::DeError;
use super::edres::ser::error::SerError;
use super::{mon, Input};
use std::io;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0:?}")]
    Mon(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("unexpected eof")]
    Eof,

    #[error(transparent)]
    SerdeDe(#[from] DeError),

    #[error(transparent)]
    SerdeSer(#[from] SerError),

    #[error(transparent)]
    NumConversion(#[from] std::num::TryFromIntError),

    #[error("{0}")]
    Packet(String),

    #[error(transparent)]
    Hex(#[from] hex::FromHexError),

    #[error("{0}")]
    Other(String),
}

impl<'a> From<mon::Error<Input<'a>>> for Error {
    fn from(error: mon::Error<Input<'a>>) -> Self {
        Self::Mon(format!("{:?}", error))
    }
}
