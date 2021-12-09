//! Complex vec parsing functions

use std::{convert::TryInto, error::Error, marker::PhantomData};

use crate::{Buffer, Parse};

#[derive(Debug, Copy, Clone, Default)]
/// Arguments for parsing a Vec<P>
pub struct VecArgs<PArgs> {
    /// Minimum number of fields that must parse to be valid
    pub min: usize,
    /// Maximum number of fields to parse - stop parsing when this is reached
    pub max: Option<usize>,
    /// Extra args that will be given to P::parse
    pub extra: PArgs,
}

/// Repeatedly attempts to parse `P`, Result is all successful attempts
impl<P: Parse<T>, T> Parse<T> for Vec<P> {
    type Args = VecArgs<P::Args>;

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
    fn parse(input: &mut impl Buffer<T>, args: &Self::Args) -> eyre::Result<Self> {
        let mut output = Self::new();
        let max = args.max.unwrap_or(usize::MAX);
        loop {
            if output.len() == max {
                break;
            }

            let mut cursor = input.cursor();
            match P::parse(&mut cursor, &args.extra) {
                Ok(p) => output.push(p),
                Err(_) => break,
            }
            let pos = cursor.position();
            if cfg!(debug_assertions) && pos == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            input.fast_forward(pos);
        }

        if output.len() < args.min {
            Err(eyre::eyre!("could not parse minimum required elements"))
        } else {
            Ok(output)
        }
    }

    #[track_caller]
    fn peek(input: &mut impl Buffer<T>, args: &Self::Args) -> bool {
        let mut count: usize = 0;
        let max = args.max.unwrap_or(usize::MAX);
        loop {
            if count == max {
                break;
            }

            let mut cursor = input.cursor();
            if !P::peek(&mut cursor, &args.extra) {
                break;
            }
            let pos = cursor.position();
            if cfg!(debug_assertions) && pos == 0 {
                panic!("parsing succeeded with 0 elements read. infinite loop detected");
            }
            input.fast_forward(pos);
            count += 1;
        }

        count >= args.min
    }
}

/// Parses P many times into a Vec<Q>, seperated by SeperatedBy
pub struct VecSeperated<P, Q, SeperatedBy>(Vec<Q>, PhantomData<(P, SeperatedBy)>);
impl<P, Q, SeperatedBy> Into<Vec<Q>> for VecSeperated<P, Q, SeperatedBy> {
    fn into(self) -> Vec<Q> {
        self.0
    }
}

/// Repeatedly attempts to parse `P`, Result is all successful attempts
impl<P: Parse<T> + TryInto<Q>, SeperatedBy: Parse<T>, T, Q> Parse<T>
    for VecSeperated<P, Q, SeperatedBy>
where
    <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
    SeperatedBy::Args: Default,
{
    type Args = VecArgs<P::Args>;

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
    fn parse(input: &mut impl Buffer<T>, args: &Self::Args) -> eyre::Result<Self> {
        let max = args.max.unwrap_or(usize::MAX);
        assert!(max >= args.min);
        if max == 0 {
            return Ok(Self(vec![], PhantomData));
        }

        let mut cursor = input.cursor();
        let first: Q = match P::parse(&mut cursor, &args.extra) {
            Ok(p) => p.try_into()?,
            Err(_) => {
                return if args.min == 0 {
                    Ok(Self(vec![], PhantomData))
                } else {
                    Err(eyre::eyre!("could not parse minimum required elements"))
                }
            }
        };
        let pos = cursor.position();
        input.fast_forward(pos);

        let mut output = vec![first];
        loop {
            if max == output.len() {
                break;
            }

            let mut cursor = input.cursor();

            if !SeperatedBy::peek_def(&mut cursor) {
                break;
            }

            match P::parse(&mut cursor, &args.extra) {
                Ok(p) => output.push(p.try_into()?),
                Err(_) => break,
            };

            let pos = cursor.position();
            input.fast_forward(pos);
        }

        if output.len() < args.min {
            Err(eyre::eyre!("could not parse minimum required elements"))
        } else {
            Ok(Self(output, PhantomData))
        }
    }

    #[track_caller]
    fn peek(input: &mut impl Buffer<T>, args: &Self::Args) -> bool {
        let mut count: usize = 0;
        let max = args.max.unwrap_or(usize::MAX);
        assert!(max >= args.min);
        if max == 0 {
            return true;
        }

        let mut cursor = input.cursor();
        if !P::peek(&mut cursor, &args.extra) {
            return args.min == 0;
        };
        let pos = cursor.position();
        input.fast_forward(pos);

        loop {
            if max == count {
                break;
            }

            let mut cursor = input.cursor();
            if !SeperatedBy::peek_def(&mut cursor) {
                break;
            }

            if !P::peek(&mut cursor, &args.extra) {
                break;
            };

            let pos = cursor.position();
            input.fast_forward(pos);
            count += 1;
        }

        count >= args.min
    }
}

// /// Parses buffer into a vector, with each value being seperated by `SeperatedBy` and ignoreing any `Ignore`
// pub fn parse_vec<P, Q, Ignore, T, B>(max: usize, input: &mut B) -> eyre::Result<Vec<Q>>
// where
//     Ignore: Parse<T>,
//     P: Parse<T>,
//     P: TryInto<Q>,
//     <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
//     B: Buffer<T>,
// {
//     if max == 0 {
//         return Ok(vec![]);
//     }

//     let mut cursor = input.cursor();
//     let first: Q = match P::parse(&mut cursor) {
//         Ok(p) => p.try_into()?,
//         Err(_) => return Ok(vec![]),
//     };
//     let pos = cursor.position();
//     input.fast_forward(pos);

//     let mut output = vec![first];
//     loop {
//         if max == output.len() {
//             break;
//         }

//         let mut cursor = input.cursor();
//         Vec::<Ignore>::peek(&mut cursor);

//         match P::parse(&mut cursor) {
//             Ok(p) => output.push(p.try_into()?),
//             Err(_) => break,
//         };

//         let pos = cursor.position();
//         input.fast_forward(pos);
//     }

//     Ok(output)
// }

// /// Parses buffer into a vector, with each value being seperated by `SeperatedBy` and ignoreing any `Ignore`
// pub fn parse_vec_seperated_by<P, Q, SeperatedBy, Ignore, T, B>(
//     max: usize,
//     input: &mut B,
// ) -> eyre::Result<Vec<Q>>
// where
//     SeperatedBy: Parse<T>,
//     Ignore: Parse<T>,
//     P: Parse<T>,
//     P: TryInto<Q>,
//     <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
//     B: Buffer<T>,
// {
//     if max == 0 {
//         return Ok(vec![]);
//     }

//     let mut cursor = input.cursor();
//     let first: Q = match P::parse(&mut cursor) {
//         Ok(p) => p.try_into()?,
//         Err(_) => return Ok(vec![]),
//     };
//     let pos = cursor.position();
//     input.fast_forward(pos);

//     let mut output = vec![first];
//     loop {
//         if max == output.len() {
//             break;
//         }

//         let mut cursor = input.cursor();

//         Vec::<Ignore>::peek(&mut cursor, &());
//         if !SeperatedBy::peek(&mut cursor) {
//             break;
//         }
//         Vec::<Ignore>::peek(&mut cursor);

//         match P::parse(&mut cursor) {
//             Ok(p) => output.push(p.try_into()?),
//             Err(_) => break,
//         };

//         let pos = cursor.position();
//         input.fast_forward(pos);
//     }

//     Ok(output)
// }

// /// Parses buffer into a vector, with each value being seperated and trailed by `SeperatedBy` and ignoreing any `Ignore`
// pub fn parse_vec_seperated_by_trailing<P, Q, SeperatedBy, Ignore, T, B>(
//     max: usize,
//     input: &mut B,
// ) -> eyre::Result<Vec<Q>>
// where
//     SeperatedBy: Parse<T>,
//     Ignore: Parse<T>,
//     P: Parse<T>,
//     P: TryInto<Q>,
//     <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
//     B: Buffer<T>,
// {
//     let mut output = vec![];
//     loop {
//         if max == output.len() {
//             break;
//         }

//         let mut cursor = input.cursor();
//         let q: Q = match P::parse(&mut cursor) {
//             Ok(p) => p.try_into()?,
//             Err(_) => break,
//         };

//         Vec::<Ignore>::peek(&mut cursor);
//         if !SeperatedBy::peek(&mut cursor) {
//             break;
//         }
//         Vec::<Ignore>::peek(&mut cursor);
//         let pos = cursor.position();
//         input.fast_forward(pos);

//         output.push(q);
//     }

//     Ok(output)
// }

// /// Parses buffer into a vector, with each value being seperated and trailed by `SeperatedBy` and ignoreing any `Ignore`
// pub fn parse_vec_seperated_by_maybe_trailing<P, Q, SeperatedBy, Ignore, T, B>(
//     max: usize,
//     input: &mut B,
// ) -> eyre::Result<Vec<Q>>
// where
//     SeperatedBy: Parse<T>,
//     SeperatedBy::Args: Default,
//     Ignore: Parse<T>,
//     P: Parse<T> + TryInto<Q>,
//     P::Args: Default,
//     <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
//     B: Buffer<T>,
// {
//     let mut output = vec![];
//     let args = Default::default();
//     let args2 = Default::default();
//     loop {
//         if max == output.len() {
//             break;
//         }

//         let mut cursor = input.cursor();
//         match P::parse(&mut cursor, &args) {
//             Ok(p) => output.push(p.try_into()?),
//             Err(_) => break,
//         };
//         let pos = cursor.position();
//         input.fast_forward(pos);

//         Vec::<Ignore>::peek(input, &());

//         let mut cursor = input.cursor();
//         if !SeperatedBy::peek(&mut cursor, &args2) {
//             break;
//         }
//         let pos = cursor.position();
//         input.fast_forward(pos);

//         Vec::<Ignore>::peek(input, &());
//     }

//     Ok(output)
// }

#[cfg(test)]
mod tests {
    use crate::{
        text::{AnyOf1, WhiteSpace},
        IntoBuf, Parse,
    };

    // use super::{
    //     parse_vec, parse_vec_seperated_by, parse_vec_seperated_by_maybe_trailing,
    //     parse_vec_seperated_by_trailing,
    // };
    use super::{VecArgs, VecSeperated};

    #[test]
    fn sequence1() {
        let mut input = "foo bar baz...".chars().into_buf();
        let res: Vec<String> = VecSeperated::<
            AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
            String,                               // into a string
            WhiteSpace,                           // seperated by whitespace
        >::parse(
            &mut input,
            &VecArgs {
                max: Some(2),
                ..Default::default()
            },
        )
        .unwrap()
        .into();
        assert_eq!(res, vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(input.collect::<String>(), " baz...".to_string())
    }

    // #[test]
    // fn sequence() {
    //     let mut input = "foo, bar , baz,...".chars().into_buf();
    //     let res = parse_vec_seperated_by::<
    //         AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
    //         String,                               // into a string
    //         Tag<",">,                             // seperated by commas
    //         WhiteSpace,                           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(usize::MAX, &mut input) // parse as many elements that can be found
    //     .unwrap();
    //     assert_eq!(
    //         res,
    //         vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
    //     );
    //     assert_eq!(input.collect::<String>(), ",...".to_string())
    // }

    // #[test]
    // fn sequence_max() {
    //     let mut input = "foo, bar , baz,...".chars().into_buf();
    //     let res = parse_vec_seperated_by::<
    //         AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
    //         String,                               // into a string
    //         Tag<",">,                             // seperated by commas
    //         WhiteSpace,                           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(2, &mut input) // parse up to 2 elements and no more
    //     .unwrap();
    //     assert_eq!(res, vec!["foo".to_string(), "bar".to_string()]);
    //     assert_eq!(input.collect::<String>(), " , baz,...".to_string())
    // }

    // #[test]
    // fn sequence_try_into() {
    //     let mut input = "123, 321 , 0,...".chars().into_buf();
    //     let res = parse_vec_seperated_by::<
    //         AnyOf1<"0123456789">, // parsing numbers
    //         usize,                // into a usize
    //         Tag<",">,             // seperated by commas
    //         WhiteSpace,           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(usize::MAX, &mut input)
    //     .unwrap();
    //     assert_eq!(res, vec![123, 321, 0]);
    //     assert_eq!(input.collect::<String>(), ",...".to_string())
    // }

    // #[test]
    // fn sequence_try_into_trailing() {
    //     let mut input = "123, 321 , 0...".chars().into_buf();
    //     let res = parse_vec_seperated_by_trailing::<
    //         AnyOf1<"0123456789">, // parsing numbers
    //         usize,                // into a usize
    //         Tag<",">,             // seperated by commas
    //         WhiteSpace,           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(usize::MAX, &mut input)
    //     .unwrap();
    //     assert_eq!(res, vec![123, 321]);
    //     assert_eq!(input.collect::<String>(), "0...".to_string())
    // }

    // #[test]
    // fn sequence_try_into_trailing_maybe1() {
    //     let mut input = "123, 321 , 0...".chars().into_buf();
    //     let res = parse_vec_seperated_by_maybe_trailing::<
    //         AnyOf1<"0123456789">, // parsing numbers
    //         usize,                // into a usize
    //         Tag<",">,             // seperated by commas
    //         WhiteSpace,           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(usize::MAX, &mut input)
    //     .unwrap();
    //     assert_eq!(res, vec![123, 321, 0]);
    //     assert_eq!(input.collect::<String>(), "...".to_string())
    // }

    // #[test]
    // fn sequence_try_into_trailing_maybe2() {
    //     let mut input = "123, 321 , 0,...".chars().into_buf();
    //     let res = parse_vec_seperated_by_maybe_trailing::<
    //         AnyOf1<"0123456789">, // parsing numbers
    //         usize,                // into a usize
    //         Tag<",">,             // seperated by commas
    //         WhiteSpace,           // ignoring any whitespaces
    //         _,
    //         _,
    //     >(usize::MAX, &mut input)
    //     .unwrap();
    //     assert_eq!(res, vec![123, 321, 0]);
    //     assert_eq!(input.collect::<String>(), "...".to_string())
    // }
}
