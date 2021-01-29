use std::iter::FromIterator;

use crate::*;

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
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> eyre::Result<Self> {
        let s = String::from_iter(input.take(TAG.len()));
        if TAG == &s {
            Ok(Tag)
        } else {
            Err(eyre::eyre!("failed to parse tag {:?}, found {:?}", TAG, s))
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
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag \"(\", found \"1\"");

        let res: Result<Tag<")">, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "failed to parse tag \")\", found \"1\"");
    }
}
