use crate::*;
use std::fmt;

#[derive(Debug, PartialEq)]
/// Error type returned by [Tag]'s [parse](Parse::parse) function
pub struct TagParseError<const TAG: &'static [u8]>;

impl<const TAG: &'static [u8]> std::error::Error for TagParseError<TAG> {}
impl<const TAG: &'static [u8]> fmt::Display for TagParseError<TAG> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing tag {:?}", TAG)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Tag is a generic type that implements Parse to match the given string exactly
///
/// ```
/// use nommy::{Parse, Buffer, bytes::Tag};
/// let mut buffer = Buffer::new("foobarbaz".bytes());
/// Tag::<b"foobar">::parse(&mut buffer).unwrap();
/// Tag::<b"baz">::parse(&mut buffer).unwrap();
/// ```
pub struct Tag<const TAG: &'static [u8]>;

impl<const TAG: &'static [u8]> Process for Tag<TAG> {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl<const TAG: &'static [u8]> Peek<u8> for Tag<TAG> {
    fn peek(input: &mut Cursor<impl Iterator<Item = u8>>) -> bool {
        TAG.iter().cloned().eq(input.take(TAG.len()))
    }
}

impl<const TAG: &'static [u8]> Parse<u8> for Tag<TAG> {
    fn parse(input: &mut Buffer<impl Iterator<Item = u8>>) -> eyre::Result<Self> {
        let b: Vec<u8> = input.take(TAG.len()).collect();
        if TAG == &b {
            Ok(Tag)
        } else {
            Err(eyre::eyre!("failed to parse tag {:?}, found {:?}", TAG, b))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, Buffer, Parse};

    #[test]
    fn test_parse_matches() {
        let mut input = Buffer::new("(){}[]<>".bytes());
        Tag::<b"(">::parse(&mut input).unwrap();
        Tag::<b")">::parse(&mut input).unwrap();
        Tag::<b"{">::parse(&mut input).unwrap();
        Tag::<b"}">::parse(&mut input).unwrap();
        Tag::<b"[">::parse(&mut input).unwrap();
        Tag::<b"]">::parse(&mut input).unwrap();
        Tag::<b"<">::parse(&mut input).unwrap();
        Tag::<b">">::parse(&mut input).unwrap();
        assert!(input.next().is_none())
    }

    #[test]
    fn test_parse_errors() {
        let res: Result<Tag<b"(">, _> = parse("1".bytes());
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag [40], found [49]");

        let res: Result<Tag<b")">, _> = parse("1".bytes());
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag [41], found [49]");
    }
}
