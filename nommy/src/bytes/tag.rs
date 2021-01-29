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
    type Error = TagParseError<TAG>;
    fn parse(input: &mut Buffer<impl Iterator<Item = u8>>) -> Result<Self, Self::Error> {
        if TAG.iter().cloned().eq(input.take(TAG.len())) {
            Ok(Tag)
        } else {
            Err(TagParseError)
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
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag [40]");

        let res: Result<Tag<b")">, _> = parse("1".bytes());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag [41]");
    }
}
