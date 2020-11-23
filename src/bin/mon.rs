#![allow(
    // unused_imports,
    dead_code,
    unreachable_code,
    unused_variables,
    unused_mut
)]

static DATA: [u8; 19] = [
    0x10, 0x00, 0xf2, 0x05, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x63, 0xdd,
    0x01, 0x01, 0x00,
];

// use nom::combinator::map;
// use nom::combinator::map_res;

// use nom::number::streaming::be_u16;
// use nom::number::streaming::be_u8;
// use nom::{bytes, character, Err, IResult, Needed};

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
    }

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
}

mod primitives {
    use super::errors::{Error, Input, Result};
    use nom::error::context;
    use nom::number::complete::be_u8;

    // pub struct VarInt();

    // impl VarInt {
    //     pub fn parse(mut input: Input) -> Result<i32> {
    //         let original_input = input;

    //         let mut bytes_read = 0u8;
    //         let mut result = 0i32;
    //         loop {
    //             let (rest, read) = context("VarInt byte", be_u8)(input)?;
    //             input = rest;

    //             let mut temp = (read & 0b01111111) as i32;
    //             result |= temp << (7 * bytes_read);

    //             bytes_read += 1;

    //             if read & 0b10000000 == 0 {
    //                 break;
    //             }

    //             if bytes_read >= 5 {
    //                 let msg = format!(
    //                     "varint must finish within 5 bytes, got 0x{0:02x} 0b{0:08b} instead",
    //                     read
    //                 );

    //                 use nom::Offset;
    //                 let err_slice = &original_input[..original_input.offset(input)];

    //                 return Error::custom(err_slice, msg);
    //             }
    //         }

    //         Ok((input, result))
    //     }
    // }

    // trait VarNumType {
    //     // type NumType;
    //     fn max_size() -> u8;
    // }

    // impl VarNumType for i32 {
    //     fn max_size() -> u8 {
    //         5
    //     }
    // }

    macro_rules! var_num {
        ($name:ident => ($type:ty, $size:expr)) => {
            pub struct $name();
            impl $name {
                pub fn parse(mut input: Input) -> Result<$type> {
                    let original_input = input;
                    let mut bytes_read = 0u8;
                    let mut result: $type = 0;
                    loop {
                        let (rest, read) =
                            context(concat!(stringify!($name), " byte"), be_u8)(input)?;
                        input = rest;
                        let mut temp = (read & 0b01111111) as $type;
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
                            use nom::Offset;
                            let err_slice = &original_input[..original_input.offset(input)];
                            return Error::custom(err_slice, msg);
                        }
                    }
                    Ok((input, result))
                }
            }
        };
    }

    var_num!(VarInt => (i32, 5));
    var_num!(VarLong => (i64, 10));
}

use primitives::VarInt;

fn main() {
    dbg!(VarInt::parse(&[
        0b10000000, 0b10000000, 0b10000000, 0b10000000 //, 0b10011111,
    ]));
}
