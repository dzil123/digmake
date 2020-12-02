use super::error::SerError;
use crate::se::{
    error::{Error, Result},
    Input, VarInt, VarLong,
};
use serde::{ser, Serialize};

pub struct Serializer {
    output: Vec<u8>,
}
