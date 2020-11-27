use nom::error::{ContextError, ErrorKind as NomErrorKind, ParseError as NomParseError};
use std::fmt;

use crate::se::Input;
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
    fn custom_raw(input: I, msg: String) -> Self {
        Self {
            errors: vec![(input, ErrorKind::Custom(msg))],
        }
    }

    pub fn as_eof(&self) -> Option<NomErrorKind> {
        for (_, kind) in &self.errors {
            match kind {
                ErrorKind::Nom(kind @ NomErrorKind::Eof) => return Some(*kind),
                _ => {}
            }
        }

        None
    }

    pub fn is_eof(&self) -> bool {
        self.as_eof().is_some()
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

// stolen from https://fasterthanli.me/series/making-our-own-ping/part-9
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
macro_rules! handle {
    ($result:expr, $original_input:expr, $input:expr, $msg_func:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                let msg = $msg_func(err);
                return crate::se::mon::Error::custom_slice($original_input, $input, msg);
            }
        }
    };
}
