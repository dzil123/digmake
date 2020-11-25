mod de;
mod error;
mod ser;
mod varint;

use crate::se::Input;
use de::Deserializer;
pub use error::{Error, Result};
use serde::Deserialize;

pub fn from_bytes<'de, T>(input: Input<'de>) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(input);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

// returns unconsumed input in addition to result of from_bytes()
pub fn from_bytes_debug<'de, T>(input: Input<'de>) -> (Input<'de>, Result<T>)
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(input);
    let result = T::deserialize(&mut deserializer);
    (deserializer.input, result)
}
