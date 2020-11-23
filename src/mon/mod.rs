#![allow(
    unused_imports,
//     dead_code,
//     unreachable_code,
//     unused_variables,
//     unused_mut
)]

mod errors;
mod primitives;

// use nom::bytes::complete::take;
// use nom::multi::{count, many0};
// use nom::number::complete::be_u16;
// use nom::sequence::tuple;
// use nom::Finish;
pub use errors::{Error, Input, Result};
pub use primitives::{Parse, ParseB, VarInt, VarLong};
// use ToUSize;

// fn main() {
//     #![allow(unused_must_use)]
//     dbg!(tuple((
//         take(2usize),
//         VarInt::parse,
//         String::parse,
//         be_u16,
//         VarInt::parse,
//         take(2usize),
//     ))(&DATA[..]));
// }
