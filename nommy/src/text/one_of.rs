use crate::{eyre, Buffer, Parse};

#[derive(Debug, Copy, Clone, PartialEq)]
/// `OneOf` is a generic type that implements [`Parse`] to match one character within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, text::OneOf};
/// let mut buffer = "-".chars().into_buf();
/// let c: char = OneOf::<"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, '-');
/// ```
pub struct OneOf<const CHARS: &'static str>(char);

impl<const CHARS: &'static str> From<OneOf<CHARS>> for char {
    fn from(v: OneOf<CHARS>) -> Self {
        v.0
    }
}

impl<const CHARS: &'static str> Parse<char> for OneOf<CHARS> {
    type Args = ();
    fn parse(input: &mut impl Buffer<char>, _: &()) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if CHARS.contains(c) {
                    Ok(Self(c))
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

    fn peek(input: &mut impl Buffer<char>, _: &()) -> bool {
        match input.next() {
            Some(c) => CHARS.contains(c),
            None => false,
        }
    }
}
