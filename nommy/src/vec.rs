//! Complex vec parsing functions

use std::{convert::TryInto, error::Error};

use crate::{Buffer, Parse};

/// Parses buffer into a vector, with each value being seperated by `SeperatedBy` and ignoreing any `Ignore`
pub fn parse_vec<P, Q, Ignore, T, B>(max: usize, input: &mut B) -> eyre::Result<Vec<Q>>
where
    Ignore: Parse<T>,
    P: Parse<T>,
    P: TryInto<Q>,
    <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
    B: Buffer<T>,
{
    if max == 0 {
        return Ok(vec![]);
    }

    let mut cursor = input.cursor();
    let first: Q = match P::parse(&mut cursor) {
        Ok(p) => p.try_into()?,
        Err(_) => return Ok(vec![]),
    };
    let pos = cursor.position();
    input.fast_forward(pos);

    let mut output = vec![first];
    loop {
        if max == output.len() {
            break;
        }

        let mut cursor = input.cursor();
        Vec::<Ignore>::peek(&mut cursor);

        match P::parse(&mut cursor) {
            Ok(p) => output.push(p.try_into()?),
            Err(_) => break,
        };

        let pos = cursor.position();
        input.fast_forward(pos);
    }

    Ok(output)
}

/// Parses buffer into a vector, with each value being seperated by `SeperatedBy` and ignoreing any `Ignore`
pub fn parse_vec_seperated_by<P, Q, SeperatedBy, Ignore, T, B>(
    max: usize,
    input: &mut B,
) -> eyre::Result<Vec<Q>>
where
    SeperatedBy: Parse<T>,
    Ignore: Parse<T>,
    P: Parse<T>,
    P: TryInto<Q>,
    <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
    B: Buffer<T>,
{
    if max == 0 {
        return Ok(vec![]);
    }

    let mut cursor = input.cursor();
    let first: Q = match P::parse(&mut cursor) {
        Ok(p) => p.try_into()?,
        Err(_) => return Ok(vec![]),
    };
    let pos = cursor.position();
    input.fast_forward(pos);

    let mut output = vec![first];
    loop {
        if max == output.len() {
            break;
        }

        let mut cursor = input.cursor();

        Vec::<Ignore>::peek(&mut cursor);
        if !SeperatedBy::peek(&mut cursor) {
            break;
        }
        Vec::<Ignore>::peek(&mut cursor);

        match P::parse(&mut cursor) {
            Ok(p) => output.push(p.try_into()?),
            Err(_) => break,
        };

        let pos = cursor.position();
        input.fast_forward(pos);
    }

    Ok(output)
}

/// Parses buffer into a vector, with each value being seperated and trailed by `SeperatedBy` and ignoreing any `Ignore`
pub fn parse_vec_seperated_by_trailing<P, Q, SeperatedBy, Ignore, T, B>(
    max: usize,
    input: &mut B,
) -> eyre::Result<Vec<Q>>
where
    SeperatedBy: Parse<T>,
    Ignore: Parse<T>,
    P: Parse<T>,
    P: TryInto<Q>,
    <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
    B: Buffer<T>,
{
    let mut output = vec![];
    loop {
        if max == output.len() {
            break;
        }

        let mut cursor = input.cursor();
        let q: Q = match P::parse(&mut cursor) {
            Ok(p) => p.try_into()?,
            Err(_) => break,
        };

        Vec::<Ignore>::peek(&mut cursor);
        if !SeperatedBy::peek(&mut cursor) {
            break;
        }
        Vec::<Ignore>::peek(&mut cursor);
        let pos = cursor.position();
        input.fast_forward(pos);

        output.push(q);
    }

    Ok(output)
}

/// Parses buffer into a vector, with each value being seperated and trailed by `SeperatedBy` and ignoreing any `Ignore`
pub fn parse_vec_seperated_by_maybe_trailing<P, Q, SeperatedBy, Ignore, T, B>(
    max: usize,
    input: &mut B,
) -> eyre::Result<Vec<Q>>
where
    SeperatedBy: Parse<T>,
    Ignore: Parse<T>,
    P: Parse<T>,
    P: TryInto<Q>,
    <P as TryInto<Q>>::Error: Error + Send + Sync + 'static,
    B: Buffer<T>,
{
    let mut output = vec![];
    loop {
        if max == output.len() {
            break;
        }

        let mut cursor = input.cursor();
        match P::parse(&mut cursor) {
            Ok(p) => output.push(p.try_into()?),
            Err(_) => break,
        };
        let pos = cursor.position();
        input.fast_forward(pos);

        Vec::<Ignore>::peek(input);

        let mut cursor = input.cursor();
        if !SeperatedBy::peek(&mut cursor) {
            break;
        }
        let pos = cursor.position();
        input.fast_forward(pos);

        Vec::<Ignore>::peek(input);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::{
        text::{AnyOf1, Tag, WhiteSpace},
        IntoBuf,
    };

    use super::{
        parse_vec, parse_vec_seperated_by, parse_vec_seperated_by_maybe_trailing,
        parse_vec_seperated_by_trailing,
    };

    #[test]
    fn sequence1() {
        let mut input = "foo bar baz...".chars().into_buf();
        let res = parse_vec::<
            AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
            String,                               // into a string
            WhiteSpace,                           // ignoring any whitespaces
            _,
            _,
        >(2, &mut input)
        .unwrap();
        assert_eq!(res, vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(input.collect::<String>(), " baz...".to_string())
    }

    #[test]
    fn sequence() {
        let mut input = "foo, bar , baz,...".chars().into_buf();
        let res = parse_vec_seperated_by::<
            AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
            String,                               // into a string
            Tag<",">,                             // seperated by commas
            WhiteSpace,                           // ignoring any whitespaces
            _,
            _,
        >(usize::MAX, &mut input) // parse as many elements that can be found
        .unwrap();
        assert_eq!(
            res,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
        assert_eq!(input.collect::<String>(), ",...".to_string())
    }

    #[test]
    fn sequence_max() {
        let mut input = "foo, bar , baz,...".chars().into_buf();
        let res = parse_vec_seperated_by::<
            AnyOf1<"abcdefghijklmnopqrstuvwxyz">, // parsing lowercase ascii characters
            String,                               // into a string
            Tag<",">,                             // seperated by commas
            WhiteSpace,                           // ignoring any whitespaces
            _,
            _,
        >(2, &mut input) // parse up to 2 elements and no more
        .unwrap();
        assert_eq!(res, vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(input.collect::<String>(), " , baz,...".to_string())
    }

    #[test]
    fn sequence_try_into() {
        let mut input = "123, 321 , 0,...".chars().into_buf();
        let res = parse_vec_seperated_by::<
            AnyOf1<"0123456789">, // parsing numbers
            usize,                // into a usize
            Tag<",">,             // seperated by commas
            WhiteSpace,           // ignoring any whitespaces
            _,
            _,
        >(usize::MAX, &mut input)
        .unwrap();
        assert_eq!(res, vec![123, 321, 0]);
        assert_eq!(input.collect::<String>(), ",...".to_string())
    }

    #[test]
    fn sequence_try_into_trailing() {
        let mut input = "123, 321 , 0...".chars().into_buf();
        let res = parse_vec_seperated_by_trailing::<
            AnyOf1<"0123456789">, // parsing numbers
            usize,                // into a usize
            Tag<",">,             // seperated by commas
            WhiteSpace,           // ignoring any whitespaces
            _,
            _,
        >(usize::MAX, &mut input)
        .unwrap();
        assert_eq!(res, vec![123, 321]);
        assert_eq!(input.collect::<String>(), "0...".to_string())
    }

    #[test]
    fn sequence_try_into_trailing_maybe1() {
        let mut input = "123, 321 , 0...".chars().into_buf();
        let res = parse_vec_seperated_by_maybe_trailing::<
            AnyOf1<"0123456789">, // parsing numbers
            usize,                // into a usize
            Tag<",">,             // seperated by commas
            WhiteSpace,           // ignoring any whitespaces
            _,
            _,
        >(usize::MAX, &mut input)
        .unwrap();
        assert_eq!(res, vec![123, 321, 0]);
        assert_eq!(input.collect::<String>(), "...".to_string())
    }

    #[test]
    fn sequence_try_into_trailing_maybe2() {
        let mut input = "123, 321 , 0,...".chars().into_buf();
        let res = parse_vec_seperated_by_maybe_trailing::<
            AnyOf1<"0123456789">, // parsing numbers
            usize,                // into a usize
            Tag<",">,             // seperated by commas
            WhiteSpace,           // ignoring any whitespaces
            _,
            _,
        >(usize::MAX, &mut input)
        .unwrap();
        assert_eq!(res, vec![123, 321, 0]);
        assert_eq!(input.collect::<String>(), "...".to_string())
    }
}
