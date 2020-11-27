mod edres;
mod error;
mod mon;
mod packet;
mod types;

pub use edres::{from_bytes, from_bytes_debug};
pub use error::{Error, Result};
pub use mon::Error as MonError;
use packet::Packet;
pub use types::{Position, VarInt, VarLong};

pub type Input<'a> = &'a [u8];
