mod edres;
mod mon;
mod shared;

pub use edres::{from_bytes, from_bytes_debug};
pub use shared::{VarInt, VarLong};

pub type Input<'a> = &'a [u8];
