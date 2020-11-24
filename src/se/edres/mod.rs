mod de;
mod error;
mod ser;
mod varint;

use de::Deserializer;
pub use error::{Error, Result};
use serde::Deserialize;

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(input);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}
