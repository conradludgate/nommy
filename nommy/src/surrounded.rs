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
    fn peek(input: &mut impl Buffer<T>) -> bool {
        Prefix::peek(input) && P::peek(input)
    }
}

/// Define Parse for PrefixedBy<P>.
/// Parse Prefix then parse P
impl<Prefix: Parse<T>, P: Parse<T>, T> Parse<T> for PrefixedBy<Prefix, P> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        Ok(PrefixedBy {
            prefix: Prefix::parse(input).wrap_err("could not parse prefix")?,
            parsed: P::parse(input).wrap_err("could not parse body")?,
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


impl<Suffix: Peek<T>, P: Peek<T>, T> Peek<T> for SuffixedBy<P, Suffix> {
    fn peek(input: &mut impl Buffer<T>) -> bool {
        P::peek(input) && Suffix::peek(input)
    }
}

/// Define Parse for SuffixedBy<P>.
/// Parse Suffix then parse P
impl<P: Parse<T>, Suffix: Parse<T>, T> Parse<T> for SuffixedBy<P, Suffix> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        Ok(SuffixedBy {
            parsed: P::parse(input).wrap_err("could not parse body")?,
            suffix: Suffix::parse(input).wrap_err("could not parse suffix")?,
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

impl<Prefix: Peek<T>, P: Peek<T>, Suffix: Peek<T>, T> Peek<T> for SurroundedBy<Prefix, P, Suffix> {
    fn peek(input: &mut impl Buffer<T>) -> bool {
        Prefix::peek(input) && P::peek(input) && Suffix::peek(input)
    }
}

/// Define Parse for SurroundedBy<P>.
/// Parse Prefix then parse P
impl<Prefix: Parse<T>, P: Parse<T>, Suffix: Parse<T>, T> Parse<T> for SurroundedBy<Prefix, P, Suffix> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        Ok(SurroundedBy {
            prefix: Prefix::parse(input).wrap_err("could not parse prefix")?,
            parsed: P::parse(input).wrap_err("could not parse body")?,
            suffix: Suffix::parse(input).wrap_err("could not parse suffix")?,
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
    use crate::{parse, text::Tag};
    use super::*;

    #[test]
    fn prefixed_by() {
        let _: PrefixedBy<Tag<"(">, Tag<".">> = parse("(.".chars()).unwrap();
    }

    #[test]
    fn suffixed_by() {
        let _: SuffixedBy<Tag<".">, Tag<")">> = parse(".)".chars()).unwrap();
    }

    #[test]
    fn surrounded_by() {
        let _: SurroundedBy<Tag<"(">, Tag<".">, Tag<")">> = parse("(.)".chars()).unwrap();
    }

    #[test]
    fn prefixed_by_peek() {
        let mut input = "(.".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(PrefixedBy::<Tag<"(">, Tag<".">>::peek(&mut cursor));
        assert!(cursor.next().is_none());
    }

    #[test]
    fn suffixed_by_peek() {
        let mut input = ".)".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(SuffixedBy::<Tag<".">, Tag<")">>::peek(&mut cursor));
        assert!(cursor.next().is_none());
    }

    #[test]
    fn surrounded_by_peek() {
        let mut input = "(.)".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(SurroundedBy::<Tag<"(">, Tag<".">, Tag<")">>::peek(&mut cursor));
        assert!(cursor.next().is_none());
    }
}
