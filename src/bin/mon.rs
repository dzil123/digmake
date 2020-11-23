#![allow(
    unused_imports,
//     dead_code,
//     unreachable_code,
//     unused_variables,
//     unused_mut
)]

static DATA: [u8; 19] = [
    0x10, 0x00, 0xf2, 0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd,
    0x01, 0x01, 0x00,
];

mod errors {
    use nom::error::ContextError;
    use nom::error::{ErrorKind as NomErrorKind, ParseError as NomParseError};
    use std::fmt;

    pub type Input<'a> = &'a [u8];
    pub type Result<'a, T> = nom::IResult<Input<'a>, T, Error<Input<'a>>>;

    #[derive(Debug)]
    pub enum ErrorKind {
        Nom(NomErrorKind),
        Context(&'static str),
        Custom(String),
    }

    pub struct Error<I> {
        pub errors: Vec<(I, ErrorKind)>,
    }

    impl<I> Error<I> {
        pub fn custom_raw(input: I, msg: String) -> Self {
            Self {
                errors: vec![(input, ErrorKind::Custom(msg))],
            }
        }
    }

    impl<'a> Error<Input<'a>> {
        pub fn custom<T>(input: Input<'a>, msg: String) -> Result<'a, T> {
            Err(nom::Err::Error(Error::custom_raw(input, msg)))
        }

        pub fn custom_slice<T>(
            original_input: Input<'a>,
            input: Input<'a>,
            msg: String,
        ) -> Result<'a, T> {
            use nom::Offset;
            let err_slice = &original_input[..original_input.offset(input)];
            Error::custom(err_slice, msg)
        }

        // pub fn map<T, E, F>(
        //     result: std::result::Result<T, E>,
        //     original_input: Input<'a>,
        //     input: Input<'a>,
        //     func: F,
        // ) -> Result<'a, T>
        // where
        //     F: FnOnce(E) -> String,
        // {
        //     match
        // }
    }

    pub trait IntoResult<'a, T> {
        fn into(self) -> Result<'a, T>;
    }

    impl<'a, T: fmt::Debug> IntoResult<'a, T>
        for std::result::Result<T, (Input<'a>, Input<'a>, String)>
    {
        fn into(self) -> Result<'a, T> {
            let (original_input, input, msg) = self.expect_err("why was this called with an Ok()?");
            Error::custom_slice(original_input, input, msg)
        }
    }

    // impl<'a, T> From<std::result::Result<T, (Input<'a>, Input<'a>, String)>> for Error<Input<'a>>
    // impl<'a, T, U> From<std::result::Result<T, (Input<'a>, Input<'a>, String)>> for Result<'a, U>
    // where
    //     T: std::fmt::Debug,
    // {
    //     fn from(result: std::result::Result<T, (Input<'a>, Input<'a>, String)>) -> Self {
    //         let (original_input, input, msg) =
    //             result.expect_err("why was this called with an Ok()?");
    //         Error::custom_slice(original_input, input, msg)
    //     }
    // }

    impl<I> NomParseError<I> for Error<I> {
        fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
            let errors = vec![(input, ErrorKind::Nom(kind))];
            Self { errors }
        }

        fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
            other.errors.push((input, ErrorKind::Nom(kind)));
            other
        }

        fn or(mut self, mut other: Self) -> Self {
            self.errors.append(&mut other.errors);
            self
        }
    }

    impl<I> ContextError<I> for Error<I> {
        fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
            other.errors.push((input, ErrorKind::Context(ctx)));
            other
        }
    }

    impl<'a> fmt::Debug for Error<Input<'a>> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "/!\\ parsing error\n")?;

            let mut shown_input = None;
            let margin_left = 4;
            let margin_str = " ".repeat(margin_left);

            // maximum amount of binary data we'll dump per line
            let maxlen = 60;

            // given a big slice, an offset, and a length, attempt to show
            // some data before, some data after, and highlight which part
            // we're talking about with tildes.
            let print_slice =
                |f: &mut fmt::Formatter, s: &[u8], offset: usize, len: usize| -> fmt::Result {
                    // decide which part of `s` we're going to show.
                    let (s, offset, len) = {
                        // see diagram further in article.
                        // TODO: review for off-by-one errors

                        let avail_after = s.len() - offset;
                        let after = std::cmp::min(avail_after, maxlen / 2);

                        let avail_before = offset;
                        let before = std::cmp::min(avail_before, maxlen / 2);

                        let new_start = offset - before;
                        let new_end = offset + after;
                        let new_offset = before;
                        let new_len = std::cmp::min(new_end - new_start, len);

                        (&s[new_start..new_end], new_offset, new_len)
                    };

                    write!(f, "{}", margin_str)?;
                    for b in s {
                        write!(f, "{:02X} ", b)?;
                    }
                    write!(f, "\n")?;

                    write!(f, "{}", margin_str)?;
                    for i in 0..s.len() {
                        // each byte takes three characters, ie "FF "
                        if i == offset + len - 1 {
                            // ..except the last one
                            write!(f, "~~")?;
                        } else if (offset..offset + len).contains(&i) {
                            write!(f, "~~~")?;
                        } else {
                            write!(f, "   ")?;
                        };
                    }
                    write!(f, "\n")?;

                    Ok(())
                };

            for (input, kind) in self.errors.iter().rev() {
                let prefix = match kind {
                    ErrorKind::Context(ctx) => format!("...in {}", ctx),
                    ErrorKind::Custom(ctx) => format!("...in {}", ctx),
                    ErrorKind::Nom(err) => format!("nom error {:?}", err),
                };

                write!(f, "{}\n", prefix)?;
                match shown_input {
                    None => {
                        shown_input.replace(input);
                        print_slice(f, input, 0, input.len())?;
                    }
                    Some(parent_input) => {
                        // `nom::Offset` is a trait that lets us get the position
                        // of a subslice into its parent slice. This works great for
                        // our error reporting!
                        use nom::Offset;
                        let offset = parent_input.offset(input);
                        print_slice(f, parent_input, offset, input.len())?;
                    }
                };
            }
            Ok(())
        }
    }

    // macro to convert std::Result<T, E> to either T or Result<'a, U>, as a replacement for the ? operator
    #[macro_export]
    macro_rules! handle {
        ($result:expr, $original_input:expr, $input:expr, $msg_func:expr) => {
            match $result {
                Ok(val) => val,
                Err(err) => {
                    let msg = $msg_func(err);
                    return Error::custom_slice($original_input, $input, msg);
                }
            }
        };
    }
}

mod primitives {
    use super::errors::{Error, Input, Result};
    use crate::handle;
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
            pub struct $name();
            impl Parse<$type> for $name {
                #[allow(dead_code)]
                fn parse(mut input: Input) -> Result<$type> {
                    let original_input = input;
                    let mut bytes_read = 0u8;
                    let mut result: $type = 0;
                    loop {
                        let (rest, read) =
                            context(concat!(stringify!($name), " byte"), be_u8)(input)?;
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
}

use nom::bytes::complete::take;
use nom::multi::{count, many0};
use nom::number::complete::be_u16;
use nom::sequence::tuple;
use nom::Finish;
use primitives::{Parse, ParseB, VarInt};
// use ToUSize;

fn main() {
    #![allow(unused_must_use)]
    dbg!(tuple((
        take(2usize),
        VarInt::parse,
        String::parse,
        be_u16,
        VarInt::parse,
        take(2usize),
    ))(&DATA[..]));
}
