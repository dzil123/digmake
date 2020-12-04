mod edres;
mod error;
mod mon;
mod types;

pub use edres::de::{from_bytes, from_bytes_debug};
pub use edres::ser::serialize;
pub use error::{Error, Result};
pub use mon::Error as MonError;
pub use types::{Position, VarInt, VarLong};

pub type Input<'a> = &'a [u8];
