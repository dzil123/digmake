mod edres;
mod mon;
mod packet;
mod types;

pub use edres::{from_bytes, from_bytes_debug};
use packet::Packet;
pub use types::{Position, VarInt, VarLong};

pub type Input<'a> = &'a [u8];
