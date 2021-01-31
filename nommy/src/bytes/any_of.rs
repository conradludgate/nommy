use crate::*;

use super::OneOf;

#[derive(Debug, Clone, PartialEq)]
/// AnyOf1 is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, Process, Buffer, bytes::AnyOf1};
/// let mut buffer = Buffer::new("-_-.".bytes());
/// let c = AnyOf1::<b"-_">::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), b"-_-");
/// ```
pub struct AnyOf1<const BYTES: &'static [u8]>(Vec<u8>);

impl<const BYTES: &'static [u8]> Process for AnyOf1<BYTES> {
    type Output = Vec<u8>;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const BYTES: &'static [u8]> Peek<u8> for AnyOf1<BYTES> {
    fn peek(input: &mut Cursor<impl Iterator<Item = u8>>) -> bool {
        if !OneOf::<BYTES>::peek(input) {
            return false;
        }
        loop {
            let mut cursor = input.cursor();
            if !OneOf::<BYTES>::peek(&mut cursor) {
                break;
            }
            let skip = cursor.close();
            input.fast_forward(skip);
        }
        true
    }
}

impl<const BYTES: &'static [u8]> Parse<u8> for AnyOf1<BYTES> {
    fn parse(input: &mut Buffer<impl Iterator<Item = u8>>) -> eyre::Result<Self> {
        let mut output = Vec::new();

        while OneOf::<BYTES>::peek(&mut input.cursor()) {
            output.push(
                OneOf::<BYTES>::parse(input)
                    .expect("peek succeeded but parse failed")
                    .process(),
            );
        }

        if output.len() == 0 {
            Err(eyre::eyre!("no characters found"))
        } else {
            Ok(AnyOf1(output))
        }
    }
}
