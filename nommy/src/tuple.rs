use crate::{Buffer, Cursor, Parse, Peek, Process};
use std::{error::Error, fmt};

macro_rules! Tuple {
    ($error:ident: $($T:ident),*) => {

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum $error<$($T),*> where $($T: Error),* {
    $(
        $T($T),
    )*
}

impl<$($T: Error),*> Error for $error<$($T),*>  {}
impl<$($T: Error),*>  fmt::Display for $error<$($T),*>  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        $(
            i += 1;
            if let $error::$T(e) = &self {
                return write!(f, "error parsing term {}: {}", i, e)
            }
        )*

        unreachable!()
    }
}

impl<T, $($T),*> Peek<T> for ($($T),*) where $($T: Peek<T>),* {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool {
        $(
            $T::peek(input) &&
        )*
        true
    }
}

impl<T, $($T),*> Parse<T> for ($($T),*) where $($T: Parse<T>),* {
    type Error = $error<$($T::Error),*>;

    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error> {
        Ok(($(
            $T::parse(input).map_err(|e| $error::$T(e))?,
        )*))
    }
}

impl<$($T),*> Process for ($($T),*) where $($T: Process),* {
    type Output = ($($T::Output),*);

    fn process(self) -> Self::Output {
        #[allow(non_snake_case)]
        let (
            $($T,)*
        ) = self;
        (
            $($T.process(),)*
        )
    }
}

    };
}
Tuple!(Tuple2ParseError: T1, T2);
Tuple!(Tuple3ParseError: T1, T2, T3);
Tuple!(Tuple4ParseError: T1, T2, T3, T4);
Tuple!(Tuple5ParseError: T1, T2, T3, T4, T5);
Tuple!(Tuple6ParseError: T1, T2, T3, T4, T5, T6);
Tuple!(Tuple7ParseError: T1, T2, T3, T4, T5, T6, T7);
Tuple!(Tuple8ParseError: T1, T2, T3, T4, T5, T6, T7, T8);
Tuple!(Tuple9ParseError: T1, T2, T3, T4, T5, T6, T7, T8, T9);
Tuple!(Tuple10ParseError: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
Tuple!(Tuple11ParseError: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
Tuple!(Tuple12ParseError: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

#[cfg(test)]
mod tests {
    use crate::{parse, Parse};
    use crate::{token::*, Buffer};

    #[test]
    fn test_parse_matches_pairs() {
        let mut input = Buffer::new("(){}[]<>".chars());
        <(LParen, RParen)>::parse(&mut input).unwrap();
        <(LBrace, RBrace)>::parse(&mut input).unwrap();
        <(LBracket, RBracket)>::parse(&mut input).unwrap();
        <(LThan, GThan)>::parse(&mut input).unwrap();
        assert!(input.next().is_none());
    }

    #[test]
    fn test_parse_matches_oct() {
        let mut input = Buffer::new("(){}[]<>".chars());
        <(
            LParen,
            RParen,
            LBrace,
            RBrace,
            LBracket,
            RBracket,
            LThan,
            GThan,
        )>::parse(&mut input)
        .unwrap();
        assert!(input.next().is_none());
    }

    #[test]
    fn test_parse_matches_oct_error() {
        let mut input = Buffer::new("(){.".chars());
        let res: Result<
            (
                LParen,
                RParen,
                LBrace,
                RBrace,
                LBracket,
                RBracket,
                LThan,
                GThan,
            ),
            _,
        > = parse(&mut input);
        assert_eq!(
            format!("{}", res.unwrap_err()),
            "error parsing term 4: error parsing tag `}`"
        );
    }
}
