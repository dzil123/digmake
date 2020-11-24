#[macro_use]
mod errors;
mod primitives;

pub use errors::{Error, Input, Result};
pub use primitives::{Parse, ParseB};
