//! Basic Parsers over string data

mod tag;
pub use tag::*;
mod one_of;
pub use one_of::*;
mod any_of;
pub use any_of::*;

use crate::*;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Parses newline `"\n"` or carriage return `"\r\n"`
pub struct LineEnding;

impl Process for LineEnding {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl Peek<char> for LineEnding {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        match input.next() {
            Some('\n') => true,
            Some('\r') => input.next() == Some('\n'),
            _ => false,
        }
    }
}

impl Parse<char> for LineEnding {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        match input.next() {
            Some('\n') => Ok(LineEnding),
            Some('\r') => {
                if input.next() == Some('\n') {
                    Ok(LineEnding)
                } else {
                    Err(eyre::eyre!("could not parse lineending"))
                }
            }
            _ => Err(eyre::eyre!("could not parse lineending")),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Space;

impl Process for Space {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl Peek<char> for Space {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        matches!(input.next(), Some(' ') | Some('\t'))
    }
}

impl Parse<char> for Space {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        match input.next() {
            Some(' ') | Some('\t') => Ok(Space),
            _ => Err(eyre::eyre!("could not parse space or tab")),
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WhiteSpace;

impl Process for WhiteSpace {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl Peek<char> for WhiteSpace {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        match input.next() {
            Some(' ') | Some('\t') | Some('\n') => true,
            Some('\r') => input.next() == Some('\n'),
            _ => false,
        }
    }
}

impl Parse<char> for WhiteSpace {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        match input.next() {
            Some(' ') | Some('\t') | Some('\n') => Ok(WhiteSpace),
            Some('\r') => {
                if input.next() == Some('\n') {
                    Ok(WhiteSpace)
                } else {
                    Err(eyre::eyre!("could not parse whitespace"))
                }
            }
            _ => Err(eyre::eyre!("could not parse whitespace")),
        }
    }
}

pub type WhiteSpaces = Vec<WhiteSpace>;
pub type Spaces = Vec<Space>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spaces() {
        let mut input = " \t \t   \t\t  \t.".chars().into_buf();
        let output = Spaces::parse(&mut input).unwrap();
        assert_eq!(output.len(), 12);
        assert_eq!(input.next(), Some('.'));
    }
    #[test]
    fn peek_spaces() {
        let mut input = " \t \t   \t\t  \t.".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(Spaces::peek(&mut cursor));
        assert_eq!(cursor.next(), Some('.'));
    }

    #[test]
    fn parse_newline() {
        let mut input = "\n.\r\n.".chars().into_buf();

        let _ = LineEnding::parse(&mut input).unwrap();
        assert_eq!(input.next(), Some('.'));
        let _ = LineEnding::parse(&mut input).unwrap();
        assert_eq!(input.next(), Some('.'));
    }
}
