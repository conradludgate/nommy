use crate::*;
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq)]
/// PrefixedBy implements [Parse], first parsing `Prefix`, then parsing `P`.
pub struct PrefixedBy<Prefix, P> {
    pub prefix: Prefix,
    pub parsed: P,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixedByParseError<PrefixParseError, ParseError>
where
    PrefixParseError: Error,
    ParseError: Error,
{
    Prefix(Box<PrefixParseError>),
    Parsed(Box<ParseError>),
}

impl<PrefixParseError: Error, ParseError: Error> Error
    for PrefixedByParseError<PrefixParseError, ParseError>
{
}
impl<PrefixParseError: Error, ParseError: Error> fmt::Display
    for PrefixedByParseError<PrefixParseError, ParseError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            PrefixedByParseError::Prefix(e) => write!(f, "could not parse prefix: {}", e),
            PrefixedByParseError::Parsed(e) => write!(f, "could not parse body: {}", e),
        }
    }
}

impl<Prefix: Peek<T>, P: Peek<T>, T> Peek<T> for PrefixedBy<Prefix, P> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        Prefix::peek(input) && P::peek(input)
    }
}

/// Define Parse for PrefixedBy<P>.
/// Parse Prefix then parse P
impl<Prefix: Parse<T>, P: Parse<T>, T> Parse<T> for PrefixedBy<Prefix, P> {
    type Error = PrefixedByParseError<Prefix::Error, P::Error>;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        Ok(PrefixedBy {
            prefix: Prefix::parse(input)
                .map_err(|err| PrefixedByParseError::Prefix(Box::new(err)))?,
            parsed: P::parse(input).map_err(|err| PrefixedByParseError::Parsed(Box::new(err)))?,
        })
    }
}

impl<Prefix, P: Process> Process for PrefixedBy<Prefix, P> {
    type Output = P::Output;
    fn process(self) -> Self::Output {
        self.parsed.process()
    }
}

#[derive(Debug, Clone, PartialEq)]
/// SuffixedBy implements [Parse], first parsing `P`, then parsing `Suffix`.
pub struct SuffixedBy<P, Suffix> {
    pub parsed: P,
    pub suffix: Suffix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuffixedByParseError<ParseError, SuffixParseError>
where
    ParseError: Error,
    SuffixParseError: Error,
{
    Parsed(Box<ParseError>),
    Suffix(Box<SuffixParseError>),
}

impl<SuffixParseError: Error, ParseError: Error> Error
    for SuffixedByParseError<SuffixParseError, ParseError>
{
}
impl<SuffixParseError: Error, ParseError: Error> fmt::Display
    for SuffixedByParseError<SuffixParseError, ParseError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            SuffixedByParseError::Parsed(e) => write!(f, "could not parse body: {}", e),
            SuffixedByParseError::Suffix(e) => write!(f, "could not parse suffix: {}", e),
        }
    }
}

impl<Suffix: Peek<T>, P: Peek<T>, T> Peek<T> for SuffixedBy<Suffix, P> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        P::peek(input) && Suffix::peek(input)
    }
}

/// Define Parse for SuffixedBy<P>.
/// Parse Suffix then parse P
impl<P: Parse<T>, Suffix: Parse<T>, T> Parse<T> for SuffixedBy<P, Suffix> {
    type Error = SuffixedByParseError<P::Error, Suffix::Error>;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        Ok(SuffixedBy {
            parsed: P::parse(input).map_err(|err| SuffixedByParseError::Parsed(Box::new(err)))?,
            suffix: Suffix::parse(input)
                .map_err(|err| SuffixedByParseError::Suffix(Box::new(err)))?,
        })
    }
}

impl<P: Process, Suffix> Process for SuffixedBy<P, Suffix> {
    type Output = P::Output;
    fn process(self) -> Self::Output {
        self.parsed.process()
    }
}


#[derive(Debug, Clone, PartialEq)]
/// SurroundedBy implements [Parse], first parsing `Prefix`, then parsing `P`, finally parsing `Suffix`.
pub struct SurroundedBy<Prefix, P, Suffix> {
    pub prefix: Prefix,
    pub parsed: P,
    pub suffix: Suffix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurroundedByParseError<PrefixParseError, ParseError, SuffixParseError>
where
    PrefixParseError: Error,
    ParseError: Error,
    SuffixParseError: Error,
{
    Prefix(Box<PrefixParseError>),
    Parsed(Box<ParseError>),
    Suffix(Box<SuffixParseError>),
}

impl<PrefixParseError: Error, ParseError: Error, SuffixParseError: Error> Error
    for SurroundedByParseError<PrefixParseError, ParseError, SuffixParseError>
{
}
impl<PrefixParseError: Error, ParseError: Error, SuffixParseError: Error> fmt::Display
    for SurroundedByParseError<PrefixParseError, ParseError, SuffixParseError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            SurroundedByParseError::Prefix(e) => write!(f, "could not parse prefix: {}", e),
            SurroundedByParseError::Parsed(e) => write!(f, "could not parse body: {}", e),
            SurroundedByParseError::Suffix(e) => write!(f, "could not parse suffix: {}", e),
        }
    }
}

impl<Prefix: Peek<T>, P: Peek<T>, Suffix: Peek<T>, T> Peek<T> for SurroundedBy<Prefix, P, Suffix> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        Prefix::peek(input) && P::peek(input) && Suffix::peek(input)
    }
}

/// Define Parse for SurroundedBy<P>.
/// Parse Prefix then parse P
impl<Prefix: Parse<T>, P: Parse<T>, Suffix: Parse<T>, T> Parse<T> for SurroundedBy<Prefix, P, Suffix> {
    type Error = SurroundedByParseError<Prefix::Error, P::Error, Suffix::Error>;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        Ok(SurroundedBy {
            prefix: Prefix::parse(input)
                .map_err(|err| SurroundedByParseError::Prefix(Box::new(err)))?,
            parsed: P::parse(input).map_err(|err| SurroundedByParseError::Parsed(Box::new(err)))?,
            suffix: Suffix::parse(input)
                .map_err(|err| SurroundedByParseError::Suffix(Box::new(err)))?,
        })
    }
}

impl<Prefix, P: Process, Suffix> Process for SurroundedBy<Prefix, P, Suffix> {
    type Output = P::Output;
    fn process(self) -> Self::Output {
        self.parsed.process()
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, token::*};
    use super::*;

    #[test]
    fn prefixed_by() {
        let prefixed: PrefixedBy<LParen, Dot> = parse("(.".chars()).unwrap();
        assert_eq!(prefixed.process(), Dot);
    }

    #[test]
    fn suffixed_by() {
        let suffixed: SuffixedBy<Dot, RParen> = parse(".)".chars()).unwrap();
        assert_eq!(suffixed.process(), Dot);
    }

    #[test]
    fn surrounded_by() {
        let surrounded: SurroundedBy<LParen, Dot, RParen> = parse("(.)".chars()).unwrap();
        assert_eq!(surrounded.process(), Dot);
    }
}
