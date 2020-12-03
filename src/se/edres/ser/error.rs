use crate::se::error::Error;
use serde::ser;
use std::fmt::Display;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum SerError {
    #[error("cannot serialize '{0}'")]
    InvalidType(&'static str),
    #[error("variant too large '{0}'")]
    LargeVariant(u32),
    #[error("{0}")]
    Other(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        SerError::Other(msg.to_string()).into()
    }
}
