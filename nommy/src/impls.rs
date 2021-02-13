//! Implemtations of [`Parse`] and [`Peek`] for types in
//! the rust standard library
use crate::{eyre, Buffer, Context, Parse};
use std::mem::MaybeUninit;

/// Result is `None` if parsing `P` fails, otherwise, result is `Some(p)`
impl<P: Parse<T>, T> Parse<T> for Option<P> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        let mut cursor = input.cursor();
        match P::parse(&mut cursor) {
            Ok(p) => {
                if cfg!(debug_assertions) && cursor.position() == 0 {
                    panic!("parsing succeeded with 0 elements read - fix: remove `Option<_>`");
                }
                let pos = cursor.position();
                input.fast_forward(pos);
                Ok(Some(p))
            }
            Err(_) => Ok(None),
        }
    }

    fn peek(input: &mut impl Buffer<T>) -> bool {
        let mut cursor = input.cursor();

        if P::peek(&mut cursor) {
            if cfg!(debug_assertions) && cursor.position() == 0 {
                panic!("parsing succeeded with 0 elements read - fix: remove `Option<_>`");
            }
            let pos = cursor.position();
            input.fast_forward(pos);
        }

        // Option should always return true for peek
        true
    }
}

/// Repeatedly attempts to parse `P`, Result is all successful attempts
impl<P: Parse<T>, T> Parse<T> for Vec<P> {
    /// # Panics
    /// If `P` is able to parse 0 tokens successfully, it would result in an infinite loop.
    /// In debug builds, we detect this and panic
    ///
    /// ```should_panic
    /// use nommy::{IntoBuf, Parse, text::Tag};
    /// let mut input = "...".chars().into_buf();
    /// Vec::<Option<Tag<".">>>::parse(&mut input);
    /// ```
    #[track_caller]
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        let mut output = Self::new();
        loop {
            let mut cursor = input.cursor();
            match P::parse(&mut cursor) {
                Ok(p) => output.push(p),
                Err(_) => break,
            }
            let pos = cursor.position();
            if cfg!(debug_assertions) && pos == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            input.fast_forward(pos);
        }

        Ok(output)
    }

    #[track_caller]
    fn peek(input: &mut impl Buffer<T>) -> bool {
        loop {
            let mut cursor = input.cursor();
            if !P::peek(&mut cursor) {
                break;
            }
            let pos = cursor.position();
            if cfg!(debug_assertions) && pos == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            input.fast_forward(pos);
        }
        true
    }
}

/// Vec1 is similar to [`Vec`] but implements [`Parse`] such that it will error if it fails to parse at least once
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

impl<P> From<Vec1<P>> for Vec<P> {
    fn from(v: Vec1<P>) -> Self {
        v.0
    }
}

/// Repeatedly attempt to parse `P`, Result is all successful attempts
/// Must parse `P` at least once
impl<P: Parse<T>, T> Parse<T> for Vec1<P> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        let mut output = vec![P::parse(input)?];
        loop {
            let mut cursor = input.cursor();
            match P::parse(&mut cursor) {
                Ok(p) => output.push(p),
                Err(_) => break,
            }
            if cfg!(debug_assertions) && cursor.position() == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            let pos = cursor.position();
            input.fast_forward(pos);
        }

        Ok(Self(output))
    }

    fn peek(input: &mut impl Buffer<T>) -> bool {
        if !P::peek(input) {
            return false;
        }

        loop {
            let mut cursor = input.cursor();
            if !P::peek(&mut cursor) {
                break;
            }
            if cfg!(debug_assertions) && cursor.position() == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            let pos = cursor.position();
            input.fast_forward(pos);
        }

        true
    }
}

/// Parse `P` `N` times into `[P; N]`, fails if any step fails
///
/// ```
/// use nommy::{parse_terminated, text::Tag};
/// let _: [Tag<".">; 3] = parse_terminated("...".chars()).unwrap();
/// ```
impl<P: Parse<T>, T, const N: usize> Parse<T> for [P; N] {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        // safety: we only return the new data if no errors occured,
        // and if no errors occured, then we definitely filled all N spaces
        // therefore the array was initialised.
        unsafe {
            let mut output = MaybeUninit::uninit_array();
            for (i, output) in output.iter_mut().enumerate() {
                *output.as_mut_ptr() =
                    P::parse(input).wrap_err_with(|| format!("could not parse element {}", i))?;
            }

            Ok(MaybeUninit::array_assume_init(output))
        }
    }

    fn peek(input: &mut impl Buffer<T>) -> bool {
        for _ in 0..N {
            if !P::peek(input) {
                return false;
            }
        }

        true
    }
}

/// Parse
impl<P: Parse<T>, T> Parse<T> for Box<P> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self> {
        Ok(Box::new(P::parse(input)?))
    }

    fn peek(input: &mut impl Buffer<T>) -> bool {
        P::peek(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, text::Tag, IntoBuf};

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
    fn sequence_peek() {
        let mut input = "...-".chars().into_buf();
        let mut cursor = input.cursor();
        assert!(Vec::<Tag<".">>::peek(&mut cursor));
        assert_eq!(cursor.next(), Some('-'));
    }

    #[test]
    fn sequence2_peek() {
        let mut input = "-...-".chars().into_buf();
        let mut cursor = input.cursor();

        assert!(Tag::<"-">::peek(&mut cursor));
        assert!(Vec::<Tag<".">>::peek(&mut cursor));
        assert_eq!(cursor.next(), Some('-'));
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
        assert_eq!(res.as_ref().len(), 3);
    }

    #[test]
    fn sequence_at_least_one_but_none() {
        let res: Result<Vec1<Tag<".">>, _> = parse("-".chars());
        assert_eq!(
            format!("{}", res.unwrap_err()),
            "failed to parse tag \".\", found \"-\""
        );
    }
}
