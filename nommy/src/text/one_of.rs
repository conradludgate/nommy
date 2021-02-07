use crate::*;
use std::{fmt, ops::RangeInclusive};

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneOf is a generic type that implements Parse to match one character within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneOf};
/// let mut buffer = "-".chars().into_buf();
/// let c: char = OneOf::<"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, '-');
/// ```
pub struct OneOf<const CHARS: &'static str>(char);

impl<const CHARS: &'static str> Into<char> for OneOf<CHARS> {
    fn into(self) -> char {
        self.0
    }
}

impl<const CHARS: &'static str> Peek<char> for OneOf<CHARS> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        match input.next() {
            Some(c) => CHARS.contains(c),
            None => false,
        }
    }
}

impl<const CHARS: &'static str> Parse<char> for OneOf<CHARS> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if CHARS.contains(c) {
                    Ok(OneOf(c))
                } else {
                    Err(eyre::eyre!(
                        "error parsing one of {:?}, found {:?}",
                        CHARS,
                        c
                    ))
                }
            }
            None => Err(eyre::eyre!("error parsing one of {:?}, reached EOF", CHARS)),
        }
    }
}

#[derive(Debug, PartialEq)]
/// Error type returned by [OneInRange]'s [parse](Parse::parse) function
pub struct OneInRangeError<const CHAR_RANGE: RangeInclusive<char>>(Option<char>);

impl<const CHAR_RANGE: RangeInclusive<char>> std::error::Error for OneInRangeError<CHAR_RANGE> {}
impl<const CHAR_RANGE: RangeInclusive<char>> fmt::Display for OneInRangeError<CHAR_RANGE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(c) => write!(
                f,
                "error parsing one char in {:?}, found {:?}",
                CHAR_RANGE, c
            ),
            None => write!(f, "error parsing one char in {:?}, EOF", CHAR_RANGE),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// OneInRange is a generic type that implements Parse to match one character within the given range
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneInRange};
/// let mut buffer = "12".chars().into_buf();
/// let c: char = OneInRange::<{'0'..='9'}>::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, '1');
/// ```
pub struct OneInRange<const CHAR_RANGE: RangeInclusive<char>>(char);

impl<const CHAR_RANGE: RangeInclusive<char>> Into<char> for OneInRange<CHAR_RANGE> {
    fn into(self) -> char {
        self.0
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Peek<char> for OneInRange<CHAR_RANGE> {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        match input.next() {
            Some(c) => CHAR_RANGE.contains(&c),
            None => false,
        }
    }
}

impl<const CHAR_RANGE: RangeInclusive<char>> Parse<char> for OneInRange<CHAR_RANGE> {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if CHAR_RANGE.contains(&c) {
                    Ok(OneInRange(c))
                } else {
                    Err(eyre::eyre!("could not parse char in range {:?}, found {:?}", CHAR_RANGE, c))
                }
            }
            None => Err(eyre::eyre!("could not parse char in range {:?}, reached EOF", CHAR_RANGE)),
        }
    }
}

/// OneLowercase parses one character that matches any lower ascii letters
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneLowercase};
/// let mut buffer = "helloWorld".chars().into_buf();
/// let c: char = OneLowercase::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, 'h');
/// ```
pub type OneLowercase = OneInRange<{ 'a'..='z' }>;

/// OneUppercase parses one character that matches any upper ascii letters
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneUppercase};
/// let mut buffer = "HELLOworld".chars().into_buf();
/// let c: char = OneUppercase::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, 'H');
/// ```
pub type OneUppercase = OneInRange<{ 'A'..='Z' }>;

/// OneDigits parses one character that matches any ascii digits
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneDigits};
/// let mut buffer = "1024$".chars().into_buf();
/// let c: char = OneDigits::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, '1');
/// ```
pub type OneDigits = OneInRange<{ '0'..='9' }>;
