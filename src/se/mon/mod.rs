#[macro_use]
mod error;
mod primitives;

pub use error::{Error, Result};
pub use primitives::{Parse, ParseB};
