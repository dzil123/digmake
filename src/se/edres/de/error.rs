use crate::se::error::Error;
use serde::de;
use std::fmt::Display;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum DeError {
    #[error("cannot deserialize '{0}'")]
    InvalidType(&'static str),
    #[error("VarInt key not found")]
    VarIntKey,
    #[error("VarInt field not found")]
    VarIntField,
    #[error("{0}")]
    Other(String),
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        DeError::Other(msg.to_string()).into()
    }
}
