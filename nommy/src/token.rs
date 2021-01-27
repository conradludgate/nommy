use crate::{Parse, Process};
use thiserror::Error;
#[derive(Debug, PartialEq, Error)]
#[error("error parsing tag. expected: `{expected}`, got: {got}")]
pub struct TokenParseError {
    pub expected: &'static str,
    pub got: String,
}

#[macro_export]
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
    type Error = TokenParseError;
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        const EXPECTED: &'static str = $expected;
        if input.len() < EXPECTED.len() {
            Err(TokenParseError{expected: EXPECTED, got: format!("`{}`", input)})
        } else {
            let (a, b) = input.split_at(EXPECTED.len());
            if a == EXPECTED {
                Ok(($name, b))
            } else {
                if b.len() == 0 {
                    Err(TokenParseError{expected: EXPECTED, got: format!("`{}`", a)})
                } else {
                    Err(TokenParseError{expected: EXPECTED, got: format!("`{}`...", a)})
                }
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
    Dot: ".",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parse;

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
        assert_eq!(
            format!("{}", LParen::parse("1").unwrap_err()),
            "error parsing tag. expected: `(`, got: `1`"
        );
        assert_eq!(
            format!("{}", RParen::parse("1").unwrap_err()),
            "error parsing tag. expected: `)`, got: `1`"
        );
        assert_eq!(
            format!("{}", LBrace::parse("1").unwrap_err()),
            "error parsing tag. expected: `{`, got: `1`"
        );
        assert_eq!(
            format!("{}", RBrace::parse("1").unwrap_err()),
            "error parsing tag. expected: `}`, got: `1`"
        );
        assert_eq!(
            format!("{}", LBracket::parse("1").unwrap_err()),
            "error parsing tag. expected: `[`, got: `1`"
        );
        assert_eq!(
            format!("{}", RBracket::parse("1").unwrap_err()),
            "error parsing tag. expected: `]`, got: `1`"
        );
        assert_eq!(
            format!("{}", LThan::parse("1").unwrap_err()),
            "error parsing tag. expected: `<`, got: `1`"
        );
        assert_eq!(
            format!("{}", GThan::parse("1").unwrap_err()),
            "error parsing tag. expected: `>`, got: `1`"
        );
    }
}
