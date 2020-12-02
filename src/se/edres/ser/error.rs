use crate::se::error::Error;
use serde::ser;
use std::fmt::Display;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum SerError {
    #[error("{0}")]
    Other(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        SerError::Other(msg.to_string()).into()
    }
}
