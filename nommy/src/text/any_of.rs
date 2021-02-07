use std::ops::RangeInclusive;

use text::OneInRange;

use crate::*;

use super::OneOf;

#[derive(Debug, Clone, PartialEq)]
/// AnyOf is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyOf};
/// let mut buffer = "-_-.".chars().into_buf();
/// let c: String = AnyOf::<"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "-_-");
/// ```
pub struct AnyOf<const CHARS: &'static str>(String);

impl<const CHARS: &'static str> From<AnyOf<CHARS>> for String {
    fn from(v: AnyOf<CHARS>) -> Self {
        v.0
    }
}

impl<const CHARS: &'static str> Peek<char> for AnyOf<CHARS> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !OneOf::<CHARS>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
        true
    }
}

impl<const CHARS: &'static str> Parse<char> for AnyOf<CHARS> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        let mut output = String::new();

        loop {
            let mut cursor = input.cursor();
            match OneOf::<CHARS>::parse(&mut cursor) {
                Ok(c) => output.push(c.into()),
                _ => break,
            }
            cursor.fast_forward_parent();
        }

        Ok(AnyOf(output))
    }
}

#[derive(Debug, Clone, PartialEq)]
/// AnyOf is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyOf};
/// let mut buffer = "-_-.".chars().into_buf();
/// let c: String = AnyOf::<"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "-_-");
/// ```
pub struct WhileNot1<const CHARS: &'static str>(String);

impl<const CHARS: &'static str> From<WhileNot1<CHARS>> for String {
    fn from(v: WhileNot1<CHARS>) -> Self {
        v.0
    }
}

impl<const CHARS: &'static str> Peek<char> for WhileNot1<CHARS> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        if OneOf::<CHARS>::peek(input) {
            return false;
        }
        loop {
            let mut cursor = input.cursor();
            if OneOf::<CHARS>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
        true
    }
}

impl<const CHARS: &'static str> Parse<char> for WhileNot1<CHARS> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        let mut output = String::new();

        while !OneOf::<CHARS>::peek(&mut input.cursor()) {
            match input.next() {
                None => break,
                Some(c) => output.push(c),
            }
        }

        if output.is_empty() {
            Err(eyre::eyre!("no characters found"))
        } else {
            Ok(WhileNot1(output))
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
/// AnyOf1 is a generic type that implements Parse to match many characters within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyOf1};
/// let mut buffer = "-_-.".chars().into_buf();
/// let c: String = AnyOf1::<"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "-_-");
/// ```
pub struct AnyOf1<const CHARS: &'static str>(String);

impl<const CHARS: &'static str> From<AnyOf1<CHARS>> for String {
    fn from(v: AnyOf1<CHARS>) -> Self {
        v.0
    }
}

impl<const CHARS: &'static str> Peek<char> for AnyOf1<CHARS> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        if !OneOf::<CHARS>::peek(input) {
            return false;
        }
        loop {
            let mut cursor = input.cursor();
            if !OneOf::<CHARS>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
        true
    }
}

impl<const CHARS: &'static str> Parse<char> for AnyOf1<CHARS> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        let mut output = String::new();

        loop {
            let mut cursor = input.cursor();
            match OneOf::<CHARS>::parse(&mut cursor) {
                Ok(c) => output.push(c.into()),
                _ => break,
            }
            cursor.fast_forward_parent();
        }

        if output.is_empty() {
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
/// use nommy::{Parse, IntoBuf, text::AnyInRange};
/// let mut buffer = "hello world".chars().into_buf();
/// let c: String = AnyInRange::<{'a'..='z'}>::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "hello");
/// ```
pub struct AnyInRange<const CHAR_RANGE: RangeInclusive<char>>(String);

impl<const CHAR_RANGE: RangeInclusive<char>> From<AnyInRange<CHAR_RANGE>> for String {
    fn from(v: AnyInRange<CHAR_RANGE>) -> Self {
        v.0
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Peek<char> for AnyInRange<CHAR_RANGE> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !OneInRange::<CHAR_RANGE>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
        true
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Parse<char> for AnyInRange<CHAR_RANGE> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        let mut output = String::new();

        while OneInRange::<CHAR_RANGE>::peek(&mut input.cursor()) {
            output.push(
                OneInRange::<CHAR_RANGE>::parse(input)
                    .expect("peek succeeded but parse failed")
                    .into(),
            );
        }

        Ok(AnyInRange(output))
    }
}

/// AnyLowercase parses any length of lower ascii letters
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyLowercase};
/// let mut buffer = "helloWorld".chars().into_buf();
/// let c: String = AnyLowercase::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "hello");
/// ```
pub type AnyLowercase = AnyInRange<{ 'a'..='z' }>;

/// AnyUppercase parses any length of upper ascii letters
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyUppercase};
/// let mut buffer = "HELLOworld".chars().into_buf();
/// let c: String = AnyUppercase::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "HELLO");
/// ```
pub type AnyUppercase = AnyInRange<{ 'A'..='Z' }>;

/// AnyDigits parses any length of ascii digits
///
/// ```
/// use nommy::{Parse, IntoBuf, text::AnyDigits};
/// let mut buffer = "1024$".chars().into_buf();
/// let c: String = AnyDigits::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, "1024");
/// ```
pub type AnyDigits = AnyInRange<{ '0'..='9' }>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_of_peek() {
        let mut buffer = "1024$".chars().into_buf();
        let mut cursor = buffer.cursor();
        assert!(AnyOf::<"0123456789">::peek(&mut cursor));
        assert_eq!(cursor.next(), Some('$'));
    }
}
