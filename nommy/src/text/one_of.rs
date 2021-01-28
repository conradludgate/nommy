use crate::*;
use std::fmt;

#[derive(Debug, PartialEq)]
/// Error type returned by [OneOf]'s [parse](Parse::parse) function
pub struct OneOfError<const CHARS: &'static str>(Option<char>);

impl<const CHARS: &'static str> std::error::Error for OneOfError<CHARS> {}
impl<const CHARS: &'static str> fmt::Display for OneOfError<CHARS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(c) => write!(f, "error parsing one of {:?}, found {:?}", CHARS, c),
            None => write!(f, "error parsing one of {:?}, EOF", CHARS),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneOf is a generic type that implements Parse to match one character within the given string
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::OneOf};
/// let mut buffer = Buffer::new("-".chars());
/// let c = OneOf::<"-_">::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), '-');
/// ```
pub struct OneOf<const TAG: &'static str>(char);

impl<const CHARS: &'static str> Process for OneOf<CHARS> {
    type Output = char;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const CHARS: &'static str> Peek<char> for OneOf<CHARS> {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        match input.next() {
            Some(c) => CHARS.contains(c),
            None => false,
        }
    }
}

impl<const CHARS: &'static str> Parse<char> for OneOf<CHARS> {
    type Error = OneOfError<CHARS>;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        match input.next() {
            Some(c) => {
                if CHARS.contains(c) {
                    Ok(OneOf(c))
                } else {
                    Err(OneOfError(Some(c)))
                }
            }
            None => Err(OneOfError(None)),
        }
    }
}
