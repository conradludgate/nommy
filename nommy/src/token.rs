
use std::{error::Error, fmt};

use crate::{Parse, Process};
#[derive(Debug, PartialEq)]
pub struct TokenError{
    pub expected: &'static str,
}
impl Error for TokenError {}
impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <TokenError as fmt::Debug>::fmt(&self, f)
    }
}

macro_rules! Tag {
    ($($name:ident:$expected:literal,)*) => {
        $(

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct $name;
impl Process for $name {
    type Output = Self;
    fn process(self) -> Self::Output {
        self
    }
}
impl Parse for $name {
    type Error = TokenError;
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        const EXPECTED: &'static str = $expected;
        if input.len() < EXPECTED.len() {
            Err(TokenError{expected: EXPECTED})
        } else {
            let (a, b) = input.split_at(EXPECTED.len());
            if a == EXPECTED {
                Ok(($name, b))
            } else {
                Err(TokenError{expected: EXPECTED})
            }
        }
    }
}

        )*
    };
}

Tag![
    LParen: "(",
    RParen: ")",
    LBrace: "{",
    RBrace: "}",
    LBracket: "[",
    RBracket: "]",
    LThan: "<",
    GThan: ">",
];

#[cfg(test)]
mod tests {
    use crate::{Parse};
    use super::*;

    #[test]
    fn test_parse_matches() {
        let input = "(){}[]<>";
        let (_, input) = LParen::parse(input).unwrap();
        let (_, input) = RParen::parse(input).unwrap();
        let (_, input) = LBrace::parse(input).unwrap();
        let (_, input) = RBrace::parse(input).unwrap();
        let (_, input) = LBracket::parse(input).unwrap();
        let (_, input) = RBracket::parse(input).unwrap();
        let (_, input) = LThan::parse(input).unwrap();
        let (_, input) = GThan::parse(input).unwrap();
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_errors() {
        assert_eq!(LParen::parse("1"), Err(TokenError{expected: "("}));
        assert_eq!(RParen::parse("1"), Err(TokenError{expected: ")"}));
        assert_eq!(LBrace::parse("1"), Err(TokenError{expected: "{"}));
        assert_eq!(RBrace::parse("1"), Err(TokenError{expected: "}"}));
        assert_eq!(LBracket::parse("1"), Err(TokenError{expected: "["}));
        assert_eq!(RBracket::parse("1"), Err(TokenError{expected: "]"}));
        assert_eq!(LThan::parse("1"), Err(TokenError{expected: "<"}));
        assert_eq!(GThan::parse("1"), Err(TokenError{expected: ">"}));
    }
}
