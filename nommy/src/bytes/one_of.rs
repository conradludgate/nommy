use crate::*;

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneOf is a generic type that implements Parse to match one character within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, bytes::OneOf};
/// let mut buffer = "-".bytes().into_buf();
/// let c: u8 = OneOf::<b"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, b'-');
/// ```
pub struct OneOf<const BYTES: &'static [u8]>(u8);

impl<const BYTES: &'static [u8]> From<OneOf<BYTES>> for u8 {
    fn from(v: OneOf<BYTES>) -> Self {
        v.0
    }
}

impl<const BYTES: &'static [u8]> Peek<u8> for OneOf<BYTES> {
    fn peek(input: &mut impl Buffer<u8>) -> bool {
        match input.next() {
            Some(c) => BYTES.contains(&c),
            None => false,
        }
    }
}

impl<const BYTES: &'static [u8]> Parse<u8> for OneOf<BYTES> {
    fn parse(input: &mut impl Buffer<u8>) -> eyre::Result<Self> {
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
