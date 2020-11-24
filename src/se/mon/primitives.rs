use super::errors::{Error, Input, Result};
use crate::se::{VarInt, VarLong};
use nom::bytes::complete::take;
use nom::error::context;
use nom::number::complete::be_u8;
use std::convert::TryFrom;

pub trait Parse<T> {
    fn parse(input: Input) -> Result<T>;
}

pub trait ParseB<T: ?Sized> {
    fn parse<'a>(input: Input<'a>) -> Result<&'a T>;
}

// shared impl for variable length numbers, VarInt and VarLong
macro_rules! var_num {
    ($name:ident => ($type:ty, $size:expr)) => {
        impl Parse<$type> for $name {
            #[allow(dead_code)]
            fn parse(mut input: Input) -> Result<$type> {
                let original_input = input;
                let mut bytes_read = 0u8;
                let mut result: $type = 0;
                loop {
                    let (rest, read) = context(concat!(stringify!($name), " byte"), be_u8)(input)?;
                    input = rest;
                    let temp = (read & 0b01111111) as $type;
                    result |= temp << (7 * bytes_read);
                    bytes_read += 1;
                    if read & 0b10000000 == 0 {
                        break;
                    }
                    if bytes_read >= $size {
                        let msg = format!(
                            concat!(
                                stringify!($name),
                                " must finish within ",
                                $size,
                                " bytes, got 0x{0:02x} 0b{0:08b} instead"
                            ),
                            read
                        );
                        return Error::custom_slice(original_input, input, msg);
                    }
                }
                Ok((input, result))
            }
        }

        impl $name {
            #[allow(dead_code)]
            fn parse_as_usize(input: Input) -> Result<usize> {
                let original_input = input;
                let (input, num) = <$name as Parse<$type>>::parse(input)?;

                let num = handle!(usize::try_from(num), original_input, input, |_| format!(
                    concat!(stringify!($type), " {} cannot fit in usize"),
                    num
                ));

                Ok((input, num))
            }
        }
    };
}

var_num!(VarInt => (i32, 5));
var_num!(VarLong => (i64, 10));

impl ParseB<str> for String {
    fn parse<'a>(input: Input<'a>) -> Result<&'a str> {
        let (input, len) = context("string length", VarInt::parse_as_usize)(input)?;
        let original_input = input;
        let (input, data) = context("string content", take(len))(input)?;
        let text = handle!(
            std::str::from_utf8(data),
            original_input,
            input,
            |err| format!("{}", err)
        );
        // let (input, text) = map_res(context("string content", take(len)), |data| {
        //     std::str::from_utf8(data)
        // })(input)?;
        Ok((input, text))
    }
}

impl Parse<bool> for bool {
    fn parse(input: Input) -> Result<bool> {
        let original_input = input;
        let (input, byte) = context("bool", take(1u8))(input)?;

        let result = match byte[0] {
            0x00 => false,
            0x01 => true,
            invalid => {
                return Error::custom_slice(
                    original_input,
                    input,
                    format!("0x{:02x} not valid bool", invalid),
                )
            }
        };

        Ok((input, result))
    }
}
