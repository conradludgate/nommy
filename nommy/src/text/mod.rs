//! Basic Parsers over string data

mod tag;
pub use tag::*;
mod one_of;
pub use one_of::*;
mod many;
pub use many::*;

use crate::{eyre, Buffer, Parse};

#[derive(Debug, Copy, Clone, PartialEq)]
/// Parses newline `"\n"` or carriage return `"\r\n"`
pub struct LineEnding;

impl Parse<char> for LineEnding {
    type Args = ();
    fn parse(input: &mut impl Buffer<char>, args: &()) -> eyre::Result<Self> {
        if Self::peek(input, args) {
            Ok(Self)
        } else {
            Err(eyre::eyre!("could not parse line ending"))
        }
    }

    fn peek(input: &mut impl Buffer<char>, _: &()) -> bool {
        match input.next() {
            Some('\n') => true,
            Some('\r') => input.next() == Some('\n'),
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Type that parses any space characters (tabs, spaces)
pub struct Space;

impl Parse<char> for Space {
    type Args = ();
    fn parse(input: &mut impl Buffer<char>, args: &()) -> eyre::Result<Self> {
        if Self::peek(input, args) {
            Ok(Self)
        } else {
            Err(eyre::eyre!("could not parse space"))
        }
    }

    fn peek(input: &mut impl Buffer<char>, _: &()) -> bool {
        matches!(input.next(), Some(' ') | Some('\t'))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Type that parses any whitespace characters (tabs, spaces, newlines and carriage returns)
pub struct WhiteSpace;

impl Parse<char> for WhiteSpace {
    type Args = ();
    fn parse(input: &mut impl Buffer<char>, args: &()) -> eyre::Result<Self> {
        if Self::peek(input, args) {
            Ok(Self)
        } else {
            Err(eyre::eyre!("could not parse whitespace"))
        }
    }

    fn peek(input: &mut impl Buffer<char>, _: &()) -> bool {
        match input.next() {
            Some(' ') | Some('\t') | Some('\n') => true,
            Some('\r') => input.next() == Some('\n'),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntoBuf;

    #[test]
    fn parse_spaces() {
        let mut input = " \t \t   \t\t  \t.".chars().into_buf();
        let output = Vec::<Space>::parse_def(&mut input).unwrap();
        assert_eq!(output.len(), 12);
        assert_eq!(input.next(), Some('.'));
    }
    #[test]
    fn peek_spaces() {
        let mut input = " \t \t   \t\t  \t.".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(Vec::<Space>::peek_def(&mut cursor));
        assert_eq!(cursor.next(), Some('.'));
    }

    #[test]
    fn parse_newline() {
        let mut input = "\n.\r\n.".chars().into_buf();

        let _ = LineEnding::parse(&mut input, &()).unwrap();
        assert_eq!(input.next(), Some('.'));
        let _ = LineEnding::parse(&mut input, &()).unwrap();
        assert_eq!(input.next(), Some('.'));
    }
}
