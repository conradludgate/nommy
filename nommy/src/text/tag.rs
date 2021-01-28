use crate::*;
use std::fmt;

#[derive(Debug, PartialEq)]
/// Error type returned by [Tag]'s [parse](Parse::parse) function
pub struct TagParseError<const TAG: &'static str>;

impl<const TAG: &'static str> std::error::Error for TagParseError<TAG> {}
impl<const TAG: &'static str> fmt::Display for TagParseError<TAG> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing tag `{}`", TAG)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Tag is a generic type that implements Parse to match the given string exactly
///
/// ```
/// use nommy::{Parse, Buffer, text::Tag};
/// let mut buffer = Buffer::new("foobarbaz".chars());
/// Tag::<"foobar">::parse(&mut buffer).unwrap();
/// Tag::<"baz">::parse(&mut buffer).unwrap();
/// ```
pub struct Tag<const TAG: &'static str>;

impl<const TAG: &'static str> Process for Tag<TAG> {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl<const TAG: &'static str> Peek<char> for Tag<TAG> {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        TAG.chars().eq(input.take(TAG.len()))
    }
}

impl<const TAG: &'static str> Parse<char> for Tag<TAG> {
    type Error = TagParseError<TAG>;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        if TAG.chars().eq(input.take(TAG.len())) {
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
        let mut input = Buffer::new("(){}[]<>".chars());
        Tag::<"(">::parse(&mut input).unwrap();
        Tag::<")">::parse(&mut input).unwrap();
        Tag::<"{">::parse(&mut input).unwrap();
        Tag::<"}">::parse(&mut input).unwrap();
        Tag::<"[">::parse(&mut input).unwrap();
        Tag::<"]">::parse(&mut input).unwrap();
        Tag::<"<">::parse(&mut input).unwrap();
        Tag::<">">::parse(&mut input).unwrap();
        assert!(input.next().is_none())
    }

    #[test]
    fn test_parse_errors() {
        let res: Result<Tag<"(">, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `(`");

        let res: Result<Tag<")">, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `)`");
    }
}
