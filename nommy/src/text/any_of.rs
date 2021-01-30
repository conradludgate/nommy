use std::ops::RangeInclusive;

use text::OneInRange;

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
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> eyre::Result<Self> {
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


#[derive(Debug, Clone, PartialEq)]
/// AnyOf1 is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyOf1};
/// let mut buffer = Buffer::new("-_-.".chars());
/// let c = AnyOf1::<"-_">::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "-_-");
/// ```
pub struct AnyOf1<const CHARS: &'static str>(String);

impl<const CHARS: &'static str> Process for AnyOf1<CHARS> {
    type Output = String;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const CHARS: &'static str> Peek<char> for AnyOf1<CHARS> {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        if !OneOf::<CHARS>::peek(input) {
            return false;
        }
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

impl<const CHARS: &'static str> Parse<char> for AnyOf1<CHARS> {
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> eyre::Result<Self> {
        let mut output = String::new();

        while OneOf::<CHARS>::peek(&mut input.cursor()) {
            output.push(
                OneOf::<CHARS>::parse(input)
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

#[derive(Debug, Clone, PartialEq)]
/// AnyInRange is a generic type that implements Parse to match many characters within the given range
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyInRange};
/// let mut buffer = Buffer::new("hello world".chars());
/// let c = AnyInRange::<{'a'..='z'}>::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "hello");
/// ```
pub struct AnyInRange<const CHAR_RANGE: RangeInclusive<char>>(String);

impl<const CHAR_RANGE: RangeInclusive<char>> Process for AnyInRange<CHAR_RANGE> {
    type Output = String;
    fn process(self) -> Self::Output {
        self.0
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Peek<char> for AnyInRange<CHAR_RANGE> {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !OneInRange::<CHAR_RANGE>::peek(&mut cursor) {
                break;
            }
            let skip = cursor.close();
            input.fast_forward(skip);
        }
        true
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Parse<char> for AnyInRange<CHAR_RANGE> {
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> eyre::Result<Self> {
        let mut output = String::new();

        while OneInRange::<CHAR_RANGE>::peek(&mut input.cursor()) {
            output.push(
                OneInRange::<CHAR_RANGE>::parse(input)
                    .expect("peek succeeded but parse failed")
                    .process(),
            );
        }

        Ok(AnyInRange(output))
    }
}

/// AnyLowercase parses any length of lower ascii letters
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyLowercase};
/// let mut buffer = Buffer::new("helloWorld".chars());
/// let c = AnyLowercase::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "hello");
/// ```
pub type AnyLowercase = AnyInRange<{ 'a'..='z' }>;

/// AnyUppercase parses any length of upper ascii letters
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyUppercase};
/// let mut buffer = Buffer::new("HELLOworld".chars());
/// let c = AnyUppercase::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "HELLO");
/// ```
pub type AnyUppercase = AnyInRange<{ 'A'..='Z' }>;

/// AnyDigits parses any length of ascii digits
///
/// ```
/// use nommy::{Parse, Process, Buffer, text::AnyDigits};
/// let mut buffer = Buffer::new("1024$".chars());
/// let c = AnyDigits::parse(&mut buffer).unwrap();
/// assert_eq!(c.process(), "1024");
/// ```
pub type AnyDigits = AnyInRange<{ '0'..='9' }>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_of_peek() {
        let mut buffer = Buffer::new("1024$".chars());
        let mut cursor = buffer.cursor();
        assert!(AnyOf::<"0123456789">::peek(&mut cursor));
        assert_eq!(cursor.next(), Some('$'));
    }
}
