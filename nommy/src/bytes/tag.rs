use crate::{eyre, Buffer, Parse};

#[derive(Debug, Copy, Clone, PartialEq)]
/// `Tag` is a generic type that implements [`Parse`] to match the given string exactly
///
/// ```
/// use nommy::{Parse, IntoBuf, bytes::Tag};
/// let mut buffer = "foobarbaz".bytes().into_buf();
/// Tag::<b"foobar">::parse(&mut buffer).unwrap();
/// Tag::<b"baz">::parse(&mut buffer).unwrap();
/// ```
pub struct Tag<const TAG: &'static [u8]>;

impl<const TAG: &'static [u8]> Parse<u8> for Tag<TAG> {
    fn parse(input: &mut impl Buffer<u8>) -> eyre::Result<Self> {
        let b: Vec<u8> = input.take(TAG.len()).collect();
        if TAG == b {
            Ok(Self)
        } else {
            Err(eyre::eyre!("failed to parse tag {:?}, found {:?}", TAG, b))
        }
    }

    fn peek(input: &mut impl Buffer<u8>) -> bool {
        TAG.iter().cloned().eq(input.take(TAG.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, IntoBuf, Parse};

    #[test]
    fn test_parse_matches() {
        let mut input = "(){}[]<>".bytes().into_buf();
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
        assert_eq!(
            format!("{}", res.unwrap_err()),
            "failed to parse tag [40], found [49]"
        );

        let res: Result<Tag<b")">, _> = parse("1".bytes());
        assert_eq!(
            format!("{}", res.unwrap_err()),
            "failed to parse tag [41], found [49]"
        );
    }
}
