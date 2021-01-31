use crate::*;
use std::{fmt, ops::RangeInclusive};

#[derive(Debug, PartialEq)]
/// Error type returned by [OneOf]'s [parse](Parse::parse) function
pub struct OneOfError<const BYTES: &'static [u8]>(Option<u8>);

impl<const BYTES: &'static [u8]> std::error::Error for OneOfError<BYTES> {}
impl<const BYTES: &'static [u8]> fmt::Display for OneOfError<BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(c) => write!(f, "error parsing one of {:?}, found {:?}", BYTES, c),
            None => write!(f, "error parsing one of {:?}, EOF", BYTES),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneOf is a generic type that implements Parse to match one character within the given string
///
/// ```
/// use nommy::{Parse, Process, Buffer, bytes::OneOf};
/// let mut buffer = Buffer::new("-".bytes());
/// let c = OneOf::<b"-_">::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), b'-');
/// ```
pub struct OneOf<const BYTES: &'static [u8]>(u8);

impl<const BYTES: &'static [u8]> Process for OneOf<BYTES> {
    type Output = u8;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const BYTES: &'static [u8]> Peek<u8> for OneOf<BYTES> {
    fn peek(input: &mut Cursor<impl Iterator<Item = u8>>) -> bool {
        match input.next() {
            Some(c) => BYTES.contains(&c),
            None => false,
        }
    }
}

impl<const BYTES: &'static [u8]> Parse<u8> for OneOf<BYTES> {
    fn parse(input: &mut Buffer<impl Iterator<Item = u8>>) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if BYTES.contains(&c) {
                    Ok(OneOf(c))
                } else {
                    Err(eyre::eyre!(
                        "error parsing one of {:?}, found {:?}",
                        BYTES,
                        c
                    ))
                }
            }
            None => Err(eyre::eyre!("error parsing one of {:?}, reached EOF", BYTES)),
        }
    }
}


#[derive(Debug, PartialEq)]
/// Error type returned by [OneInRange]'s [parse](Parse::parse) function
pub struct OneInRangeError<const BYTE_RANGE: RangeInclusive<u8>>(Option<u8>);

impl<const BYTE_RANGE: RangeInclusive<u8>> std::error::Error for OneInRangeError<BYTE_RANGE> {}
impl<const BYTE_RANGE: RangeInclusive<u8>> fmt::Display for OneInRangeError<BYTE_RANGE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(c) => write!(f, "error parsing one char in {:?}, found {:?}", BYTE_RANGE, c),
            None => write!(f, "error parsing one char in {:?}, EOF", BYTE_RANGE),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneInRange is a generic type that implements Parse to match one character within the given range
///
/// ```
/// use nommy::{Parse, Process, Buffer, bytes::OneInRange};
/// let mut buffer = Buffer::new(5..);
/// let c = OneInRange::<{0..=10}>::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), 5);
/// ```
pub struct OneInRange<const BYTE_RANGE: RangeInclusive<u8>>(u8);

impl<const BYTE_RANGE: RangeInclusive<u8>> Process for OneInRange<BYTE_RANGE> {
    type Output = u8;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const BYTE_RANGE: RangeInclusive<u8>> Peek<u8> for OneInRange<BYTE_RANGE> {
    fn peek(input: &mut Cursor<impl Iterator<Item = u8>>) -> bool {
        match input.next() {
            Some(c) => BYTE_RANGE.contains(&c),
            None => false,
        }
    }
}

impl<const BYTE_RANGE: RangeInclusive<u8>> Parse<u8> for OneInRange<BYTE_RANGE> {
    fn parse(input: &mut Buffer<impl Iterator<Item = u8>>) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if BYTE_RANGE.contains(&c) {
                    Ok(OneInRange(c))
                } else {
                    Err(eyre::eyre!("could not parse byte in range {:?}, found {:?}", BYTE_RANGE, c))
                }
            }
            None => Err(eyre::eyre!("could not parse byte in range {:?}, reached EOF", BYTE_RANGE)),
        }
    }
}
