use crate::*;

use super::OneOf;

#[derive(Debug, Clone, PartialEq)]
/// AnyOf1 is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, bytes::AnyOf1};
/// let mut buffer = "-_-.".bytes().into_buf();
/// let c: Vec<u8> = AnyOf1::<b"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, b"-_-");
/// ```
pub struct AnyOf1<const BYTES: &'static [u8]>(Vec<u8>);

impl<const BYTES: &'static [u8]> From<AnyOf1<BYTES>> for Vec<u8> {
    fn from(v: AnyOf1<BYTES>) -> Self {
        v.0
    }
}

impl<const BYTES: &'static [u8]> Peek<u8> for AnyOf1<BYTES> {
    fn peek(input: &mut impl Buffer<u8>) -> bool {
        if !OneOf::<BYTES>::peek(input) {
            return false;
        }
        loop {
            let mut cursor = input.cursor();
            if !OneOf::<BYTES>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
        true
    }
}

impl<const BYTES: &'static [u8]> Parse<u8> for AnyOf1<BYTES> {
    fn parse(input: &mut impl Buffer<u8>) -> eyre::Result<Self> {
        let mut output = Vec::new();

        while OneOf::<BYTES>::peek(&mut input.cursor()) {
            output.push(
                OneOf::<BYTES>::parse(input)
                    .expect("peek succeeded but parse failed")
                    .into(),
            );
        }

        if output.is_empty() {
            Err(eyre::eyre!("no characters found"))
        } else {
            Ok(AnyOf1(output))
        }
    }
}
