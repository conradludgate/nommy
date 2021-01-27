use std::fmt;
use crate::{Buffer, Cursor, Parse, Peek, Process};

#[derive(Debug, PartialEq)]
pub struct TokenParseError {
    pub expected: &'static str,
}

impl std::error::Error for TokenParseError {}
impl fmt::Display for TokenParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing tag `{}`", self.expected)
    }
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
impl Peek<char> for $name {
    fn peek(input: &mut Cursor<impl Iterator<Item = char>>) -> bool {
        const EXPECTED: &'static str = $expected;
        EXPECTED.chars().eq(input.take(EXPECTED.len()))
    }
}
impl Parse<char> for $name {
    type Error = TokenParseError;
    fn parse(input: &mut Buffer<impl Iterator<Item = char>>) -> Result<Self, Self::Error> {
        const EXPECTED: &'static str = $expected;
        if EXPECTED.chars().eq(input.take(EXPECTED.len())) {
            Ok($name)
        } else {
            Err(TokenParseError{expected: EXPECTED})
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
    use crate::{parse, Parse};

    #[test]
    fn test_parse_matches() {
        let mut input = Buffer::new("(){}[]<>".chars());
        LParen::parse(&mut input).unwrap();
        RParen::parse(&mut input).unwrap();
        LBrace::parse(&mut input).unwrap();
        RBrace::parse(&mut input).unwrap();
        LBracket::parse(&mut input).unwrap();
        RBracket::parse(&mut input).unwrap();
        LThan::parse(&mut input).unwrap();
        GThan::parse(&mut input).unwrap();
        assert!(input.next().is_none())
    }

    #[test]
    fn test_parse_errors() {
        let res: Result<LParen, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `(`");

        let res: Result<RParen, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `)`");

        let res: Result<LBrace, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `{`");

        let res: Result<RBrace, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `}`");

        let res: Result<LBracket, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `[`");

        let res: Result<RBracket, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `]`");

        let res: Result<LThan, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `<`");

        let res: Result<GThan, _> = parse("1".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `>`");
    }
}
