use super::edres::error::Deserialize;
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
    SerdeDe(#[from] Deserialize),

    #[error(transparent)]
    NumConversion(#[from] std::num::TryFromIntError),

    #[error("{0}")]
    Packet(String),

    #[error("{0}")]
    Other(String),
}

impl<'a> From<mon::Error<Input<'a>>> for Error {
    fn from(error: mon::Error<Input<'a>>) -> Self {
        Self::Mon(format!("{:?}", error))
    }
}
