//! Implemtations of [`Parse`] and [`Peek`] for types in
//! the rust standard library
use crate::{eyre, Buffer, Context, Parse};
use std::mem::MaybeUninit;

/// Result is `None` if parsing `P` fails, otherwise, result is `Some(p)`
impl<P: Parse<T>, T> Parse<T> for Option<P> {
    type Args = P::Args;

    fn parse(input: &mut impl Buffer<T>, args: &Self::Args) -> eyre::Result<Self> {
        let mut cursor = input.cursor();
        match P::parse(&mut cursor, args) {
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

    fn peek(input: &mut impl Buffer<T>, args: &Self::Args) -> bool {
        let mut cursor = input.cursor();

        if P::peek(&mut cursor, args) {
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

/// Parse `P` `N` times into `[P; N]`, fails if any step fails
///
/// ```
/// use nommy::{parse_terminated, text::Tag};
/// let _: [Tag<".">; 3] = parse_terminated("...".chars()).unwrap();
/// ```
impl<P: Parse<T>, T, const N: usize> Parse<T> for [P; N]
where
    P::Args: Default,
{
    type Args = ();

    fn parse(input: &mut impl Buffer<T>, _: &()) -> eyre::Result<Self> {
        let args = Default::default();
        // safety: we only return the new data if no errors occured,
        // and if no errors occured, then we definitely filled all N spaces
        // therefore the array was initialised.
        unsafe {
            let mut output = MaybeUninit::uninit_array();
            for (i, output) in output.iter_mut().enumerate() {
                *output.as_mut_ptr() = P::parse(input, &args)
                    .wrap_err_with(|| format!("could not parse element {}", i))?;
            }

            Ok(MaybeUninit::array_assume_init(output))
        }
    }

    fn peek(input: &mut impl Buffer<T>, _: &()) -> bool {
        let args = Default::default();
        for _ in 0..N {
            if !P::peek(input, &args) {
                return false;
            }
        }

        true
    }
}

/// Parse
impl<P: Parse<T>, T> Parse<T> for Box<P> {
    type Args = P::Args;
    fn parse(input: &mut impl Buffer<T>, args: &Self::Args) -> eyre::Result<Self> {
        Ok(Box::new(P::parse(input, &args)?))
    }

    fn peek(input: &mut impl Buffer<T>, args: &Self::Args) -> bool {
        P::peek(input, &args)
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
        assert!(Vec::<Tag<".">>::peek_def(&mut cursor));
        assert_eq!(cursor.next(), Some('-'));
    }

    #[test]
    fn sequence2_peek() {
        let mut input = "-...-".chars().into_buf();
        let mut cursor = input.cursor();

        assert!(Tag::<"-">::peek_def(&mut cursor));
        assert!(Vec::<Tag<".">>::peek_def(&mut cursor));
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
}
