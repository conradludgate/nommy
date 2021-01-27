use std::{error::Error, fmt::Debug};
use thiserror::Error;
use crate::{Parse, Process};

#[derive(Debug, Error)]
#[error("error should not have occured. This is probably a bug with nommy")]
pub struct NeverError;

/// Define Parse for Option<P>.
/// Result is None if parsing P fails
/// Otherwise, result is Some(p)
impl<P: Parse> Parse for Option<P> {
    type Error = NeverError;
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        match P::parse(input) {
            Ok((p, input)) => Ok((Some(p), input)),
            _ => Ok((None, input)),
        }
    }
}

impl<P: Process> Process for Option<P> {
    type Output = Option<P::Output>;
    fn process(self) -> Self::Output {
        self.map(P::process)
    }
}

/// Define Parse for Vec<P>.
/// Repeatedly attempt to parse P,
/// Result is all successful attempts
impl<P: Parse> Parse for Vec<P> {
    type Error = NeverError;
    fn parse(mut input: &str) -> Result<(Self, &str), Self::Error> {
        let mut output = vec![];
        loop {
            match P::parse(input) {
                Ok((p, next)) => {
                    input = next;
                    output.push(p);
                }
                _ => break,
            }
        }
        Ok((output, input))
    }
}

impl<P: Process> Process for Vec<P> {
    type Output = Vec<P::Output>;
    fn process(self) -> Self::Output {
        self.into_iter().map(P::process).collect()
    }
}

/// Vec1 is similar to `Vec` but implements `Parse` such that it will error if it fails to parse at least once
#[derive(Debug, Clone, PartialEq)]
pub struct Vec1<P>(Vec<P>);

impl<P> AsRef<Vec<P>> for Vec1<P> {
    fn as_ref(&self) -> &Vec<P> {
        &self.0
    }
}

impl<P> AsMut<Vec<P>> for Vec1<P> {
    fn as_mut(&mut self) -> &mut Vec<P> {
        &mut self.0
    }
}

impl<P> Vec1<P> {
    pub fn into_inner(self) -> Vec<P> {
        self.0
    }
}

/// Define Parse for Vec1<P>.
/// Repeatedly attempt to parse P,
/// Result is all successful attempts
impl<P: Parse> Parse for Vec1<P> {
    type Error = P::Error;
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        let (first, mut input) = P::parse(input)?;

        let mut output = vec![first];
        loop {
            match P::parse(input) {
                Ok((p, next)) => {
                    input = next;
                    output.push(p);
                }
                _ => break,
            }
        }
        Ok((Vec1(output), input))
    }
}

impl<P: Process> Process for Vec1<P> {
    type Output = Vec<P::Output>;
    fn process(self) -> Self::Output {
        self.0.into_iter().map(P::process).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixedBy<Prefix, P> {
    pub prefix: Prefix,
    pub parsed: P,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PrefixedByParseError<PrefixParseError, ParseError>
where
    PrefixParseError: Error,
    ParseError: Error,
{
    #[error("could not parse prefix: {0}")]
    Prefix(Box<PrefixParseError>),
    #[error("could not parse body: {0}")]
    Parsed(Box<ParseError>),
}

/// Define Parse for PrefixedBy<P>.
/// Parse Prefix then parse P
impl<Prefix: Parse, P: Parse> Parse for PrefixedBy<Prefix, P> {
    type Error = PrefixedByParseError<Prefix::Error, P::Error>;
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        let (prefix, input) =
            Prefix::parse(input).map_err(|err| PrefixedByParseError::Prefix(Box::new(err)))?;
        let (parsed, input) =
            P::parse(input).map_err(|err| PrefixedByParseError::Parsed(Box::new(err)))?;
        Ok((PrefixedBy { prefix, parsed }, input))
    }
}

impl<Prefix, P: Process> Process for PrefixedBy<Prefix, P> {
    type Output = P::Output;
    fn process(self) -> Self::Output {
        self.parsed.process()
    }
}

#[cfg(test)]
mod tests {
    use crate::token::*;
    use crate::Parse;

    use super::Vec1;

    #[test]
    fn option() {
        let (output, input) = Option::<Dot>::parse(".").unwrap();
        assert_eq!(input, "");
        assert_eq!(output, Some(Dot))
    }

    #[test]
    fn option_none() {
        let (output, input) = Option::<Dot>::parse("").unwrap();
        assert_eq!(input, "");
        assert_eq!(output, None)
    }

    #[test]
    fn sequence() {
        let (output, input) = Vec::<Dot>::parse("...").unwrap();
        assert_eq!(input, "");
        assert_eq!(output, vec![Dot, Dot, Dot])
    }

    #[test]
    fn sequence_none() {
        let (output, input) = Vec::<Dot>::parse("").unwrap();
        assert_eq!(input, "");
        assert_eq!(output, vec![])
    }

    #[test]
    fn sequence_at_least_one() {
        let (output, input) = Vec1::<Dot>::parse("...").unwrap();
        assert_eq!(input, "");
        assert_eq!(output.into_inner(), vec![Dot, Dot, Dot])
    }

    #[test]
    fn sequence_at_least_one_but_none() {
        let err = Vec1::<Dot>::parse("").unwrap_err();
        assert_eq!(format!("{}", err), "error parsing tag. expected: `.`, got: ``");
    }
}
