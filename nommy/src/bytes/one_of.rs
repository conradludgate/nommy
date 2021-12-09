use crate::{eyre, Buffer, Parse};

#[derive(Debug, Copy, Clone, PartialEq)]
/// `OneOf` is a generic type that implements [`Parse`] to match one character within the given string
///
/// ```
/// use nommy::{Parse, IntoBuf, bytes::OneOf};
/// let mut buffer = "-".bytes().into_buf();
/// let c: u8 = OneOf::<b"-_">::parse(&mut buffer).unwrap().into();
/// assert_eq!(c, b'-');
/// ```
pub struct OneOf<const BYTES: &'static [u8]>(u8);

impl<const BYTES: &'static [u8]> From<OneOf<BYTES>> for u8 {
    fn from(v: OneOf<BYTES>) -> Self {
        v.0
    }
}

impl<const BYTES: &'static [u8]> Parse<u8> for OneOf<BYTES> {
    type Args = ();
    fn parse(input: &mut impl Buffer<u8>, _: &()) -> eyre::Result<Self> {
        match input.next() {
            Some(c) => {
                if BYTES.contains(&c) {
                    Ok(Self(c))
                } else {
                    Err(eyre::eyre!(
                        "error parsing one of {:?}, found {:?}",
                        BYTES,
                        c
                    ))
                }
            }
            None => Err(eyre::eyre!("error parsing one of {:?}, reached EOF", BYTES)),
        }
    }

    fn peek(input: &mut impl Buffer<u8>, _: &()) -> bool {
        match input.next() {
            Some(c) => BYTES.contains(&c),
            None => false,
        }
    }
}
