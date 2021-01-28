use crate::*;
use std::{convert::Infallible, fmt, mem::MaybeUninit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnumParseError;
impl std::error::Error for EnumParseError{}
impl fmt::Display for EnumParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no variants of the enum could be parsed")
    }
}


impl<P: Peek<T>, T> Peek<T> for Option<P> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        let mut cursor = input.cursor();

        if P::peek(&mut cursor) {
            let skip = cursor.close();
            input.fast_forward(skip);
        }

        // Option should always return true for peek
        true
    }
}

/// Result is None if parsing P fails, otherwise, result is Some(p)
impl<P: Parse<T>, T> Parse<T> for Option<P> {
    type Error = Infallible;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        if P::peek(&mut input.cursor()) {
            Ok(Some(
                P::parse(input).expect("peek succeeded but parse failed"),
            ))
        } else {
            Ok(None)
        }
    }
}

impl<P: Process> Process for Option<P> {
    type Output = Option<P::Output>;
    fn process(self) -> Self::Output {
        self.map(P::process)
    }
}

impl<P: Peek<T>, T> Peek<T> for Vec<P> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !P::peek(&mut cursor) {
                break;
            }
            let skip = cursor.close();
            input.fast_forward(skip);
        }
        true
    }
}

/// Repeatedly attempts to parse P, Result is all successful attempts
impl<P: Parse<T>, T> Parse<T> for Vec<P> {
    type Error = Infallible;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        let mut output = vec![];
        while P::peek(&mut input.cursor()) {
            output.push(P::parse(input).expect("peek succeeded but parse failed"));
        }

        Ok(output)
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

impl<P: Peek<T>, T> Peek<T> for Vec1<P> {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        if !P::peek(input) {
            return false;
        }

        loop {
            let mut cursor = input.cursor();
            if !P::peek(&mut cursor) {
                break;
            }
            let skip = cursor.close();
            input.fast_forward(skip);
        }

        true
    }
}

/// Repeatedly attempt to parse P, Result is all successful attempts
/// Must parse P at least once
impl<P: Parse<T>, T> Parse<T> for Vec1<P> {
    type Error = P::Error;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        let mut output = vec![P::parse(input)?];
        while P::peek(&mut input.cursor()) {
            output.push(P::parse(input).expect("peek succeeded but parse failed"));
        }

        Ok(Vec1(output))
    }
}

impl<P: Process> Process for Vec1<P> {
    type Output = Vec<P::Output>;
    fn process(self) -> Self::Output {
        self.0.into_iter().map(P::process).collect()
    }
}

impl<P: Peek<T>, T, const N: usize> Peek<T> for [P; N] {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        for _ in 0..N {
            if !P::peek(input) {
                return false;
            }
        }

        true
    }
}

/// Parse P N times into [P; N], fails if any step fails
///
/// ```
/// use nommy::{parse, text::Tag};
/// let _: [Tag<".">; 3] = parse("...".chars()).unwrap();
/// ```
impl<P: Parse<T>, T, const N: usize> Parse<T> for [P; N] {
    type Error = P::Error;
    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        // safety: we only return the new data if no errors occured,
        // and if no errors occured, then we definitely filled all N spaces
        // therefore the array was initialised.
        unsafe {
            let mut output = MaybeUninit::uninit_array();
            for i in 0..N {
                *output[i].as_mut_ptr() = P::parse(input)?;
            }

            Ok(MaybeUninit::array_assume_init(output))
        }
    }
}

impl<P: Process, const N: usize> Process for [P; N] {
    type Output = [P::Output; N];
    fn process(self) -> Self::Output {
        self.map(P::process)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, text::Tag};

    use super::Vec1;

    #[test]
    fn option() {
        let res: Option<Tag<".">> = parse(".".chars()).unwrap();
        assert!(res.is_some());
    }

    #[test]
    fn option_none() {
        let res: Option<Tag<".">> = parse("".chars()).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn sequence() {
        let res: Vec<Tag<".">> = parse("...".chars()).unwrap();
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn count() {
        let _: [Tag<".">; 3] = parse("...".chars()).unwrap();
    }

    #[test]
    fn sequence_none() {
        let res: Vec<Tag<".">> = parse("-".chars()).unwrap();
        assert!(res.is_empty())
    }

    #[test]
    fn sequence_at_least_one() {
        let res: Vec1<Tag<".">> = parse("...".chars()).unwrap();
        assert_eq!(res.into_inner().len(), 3);
    }

    #[test]
    fn sequence_at_least_one_but_none() {
        let res: Result<Vec1<Tag<".">>, _> = parse("-".chars());
        assert_eq!(format!("{}", res.unwrap_err()), "error parsing tag `.`");
    }
}
