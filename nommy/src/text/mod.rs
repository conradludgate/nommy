pub mod token;

use std::fmt;

use crate::*;

#[derive(Debug, PartialEq)]
pub struct OneOfError {
    pub one_of: &'static str,
}

impl std::error::Error for OneOfError {}
impl fmt::Display for OneOfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing chars. expected one of {:?}", self.one_of)
    }
}

#[derive(Debug, PartialEq)]
pub struct LineEndingError;

impl std::error::Error for LineEndingError {}
impl fmt::Display for LineEndingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not parse line ending")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Parses newline `"\n"` or carriage return `"\r\n"`
struct LineEnding;

impl Process for LineEnding {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl Peek<char> for LineEnding {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        match input.next() {
            Some('\n') => true,
            Some('\r') => input.next() == Some('\n'),
            _ => false,
        }
    }
}

impl Parse<char> for LineEnding {
    type Error = LineEndingError;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        match input.next() {
            Some('\n') => Ok(LineEnding),
            Some('\r') => {
                if input.next() == Some('\n') {
                    Ok(LineEnding)
                } else {
                    Err(LineEndingError)
                }
            }
            _ => Err(LineEndingError),
        }
    }
}

#[derive(Debug, PartialEq)]
/// Parses space `" "` or tab `"\t"`
pub struct SpaceError;

impl std::error::Error for SpaceError {}
impl fmt::Display for SpaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not parse space or tab")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Space;

impl Process for Space {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}

impl Peek<char> for Space {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        match input.next() {
            Some(' ') => true,
            Some('\t') => true,
            _ => false,
        }
    }
}

impl Parse<char> for Space {
    type Error = SpaceError;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        match input.next() {
            Some(' ') => Ok(Space),
            Some('\t') => Ok(Space),
            _ => Err(SpaceError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spaces() {
        let mut input = Buffer::new(" \t \t   \t\t  \t.".chars());
        let output = Vec::<Space>::parse(&mut input).unwrap();
        assert_eq!(output.len(), 12);
        assert_eq!(input.next(), Some('.'));
    }

    #[test]
    fn parse_newline() {
        let mut input = Buffer::new("\n.\r\n.".chars());

        let _ = LineEnding::parse(&mut input).unwrap();
        assert_eq!(input.next(), Some('.'));
        let _ = LineEnding::parse(&mut input).unwrap();
        assert_eq!(input.next(), Some('.'));
    }
}
