
use std::{error::Error, fmt, iter::Peekable};

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

macro_rules! Token {
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
    fn parse(input: &mut Peekable<impl Iterator<Item=char>>) -> Result<Self, Self::Error> {
        const EXPECTED: &'static str = $expected;
        let mut chars = EXPECTED.chars();
        loop {
            match (chars.next(), input.peek()) {
                (Some(c1), Some(&c2)) => {
                    if c1 != c2 {
                        return Err(TokenError{expected: EXPECTED})
                    }
                }
                (Some(_), None) => return Err(TokenError{expected: EXPECTED}),
                (None, _) => return Ok($name),
            }
            input.next();
        }
    }
}

        )*
    };
}

Token![
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
        let mut input = "(){}[]<>".chars().peekable();
        assert_eq!(LParen::parse(&mut input), Ok(LParen));
        assert_eq!(RParen::parse(&mut input), Ok(RParen));
        assert_eq!(LBrace::parse(&mut input), Ok(LBrace));
        assert_eq!(RBrace::parse(&mut input), Ok(RBrace));
        assert_eq!(LBracket::parse(&mut input), Ok(LBracket));
        assert_eq!(RBracket::parse(&mut input), Ok(RBracket));
        assert_eq!(LThan::parse(&mut input), Ok(LThan));
        assert_eq!(GThan::parse(&mut input), Ok(GThan));
        assert_eq!(input.next(), None)
    }

    #[test]
    fn test_parse_errors() {
        let mut input = "1".chars().peekable();
        assert_eq!(LParen::parse(&mut input), Err(TokenError{expected: "("}));
        assert_eq!(RParen::parse(&mut input), Err(TokenError{expected: ")"}));
        assert_eq!(LBrace::parse(&mut input), Err(TokenError{expected: "{"}));
        assert_eq!(RBrace::parse(&mut input), Err(TokenError{expected: "}"}));
        assert_eq!(LBracket::parse(&mut input), Err(TokenError{expected: "["}));
        assert_eq!(RBracket::parse(&mut input), Err(TokenError{expected: "]"}));
        assert_eq!(LThan::parse(&mut input), Err(TokenError{expected: "<"}));
        assert_eq!(GThan::parse(&mut input), Err(TokenError{expected: ">"}));
        assert_eq!(input.next(), Some('1'));
    }
}
