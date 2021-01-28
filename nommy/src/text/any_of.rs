use std::{convert::Infallible};

use crate::*;

use super::OneOf;

#[derive(Debug, Clone, PartialEq)]
/// AnyOf is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyOf};
/// let mut buffer = Buffer::new("-_-.".chars());
/// let c = AnyOf::<"-_">::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "-_-");
/// ```
pub struct AnyOf<const CHARS: &'static str>(String);

impl<const CHARS: &'static str> Process for AnyOf<CHARS> {
    type Output = String;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const CHARS: &'static str> Peek<char> for AnyOf<CHARS> {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !OneOf::<CHARS>::peek(&mut cursor) {
                break;
            }
            let skip = cursor.close();
            input.fast_forward(skip);
        }
        true
    }
}

impl<const CHARS: &'static str> Parse<char> for AnyOf<CHARS> {
    type Error = Infallible;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        let mut output = String::new();

        while OneOf::<CHARS>::peek(&mut input.cursor()) {
            output.push(
                OneOf::<CHARS>::parse(input)
                    .expect("peek succeeded but parse failed")
                    .process(),
            );
        }

        Ok(AnyOf(output))
    }
}
