use crate::se::{mon::Error as MonError, Input};
use serde::{de, ser};
use std::fmt::{self, Display};

pub fn err<T>(msg: &'static str) -> Result<T> {
    // panic!(msg);
    Err(Error::Message(msg.to_string()))
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    Message(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl From<nom::Err<MonError<Input<'_>>>> for Error {
    fn from(result: nom::Err<MonError<Input<'_>>>) -> Self {
        Self::Message(format!("{}", result))
    }
}

impl std::error::Error for Error {}
