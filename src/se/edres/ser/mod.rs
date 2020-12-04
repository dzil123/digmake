pub mod error;
mod ser;
mod types;
use crate::se::error::Result;

pub fn serialize<T>(data: T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let mut ser = ser::Serializer {
        output: Vec::new(),
        fake: std::marker::PhantomData,
    };

    data.serialize(&mut ser)?;
    Ok(ser.output)
}
