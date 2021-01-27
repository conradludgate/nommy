use crate::{Parse, Process};
use thiserror::Error;
use std::error::Error;

macro_rules! Tuple {
    ($error:ident: $($T:ident),*) => {

#[derive(Debug, Copy, Clone, PartialEq, Error)]
pub enum $error<$($T),*> where $($T: Error),* {
    $(
        #[error("could not parse tuple")]
        $T($T),
    )*
}

impl<$($T),*> Parse for ($($T),*) where $($T: Parse),* {
    type Error = $error<$($T::Error),*>;
    #[allow(non_snake_case)]
    fn parse(input: &str) -> Result<(Self, &str), Self::Error> {
        $(
            let ($T, input) = $T::parse(input).map_err(|e| $error::$T(e))?;
        )*

        Ok((($($T),*), input))
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
    use crate::token::*;
    use crate::Parse;

    #[test]
    fn test_parse_matches_pairs() {
        let input = "(){}[]<>";
        let (_, input) = <(LParen, RParen)>::parse(input).unwrap();
        let (_, input) = <(LBrace, RBrace)>::parse(input).unwrap();
        let (_, input) = <(LBracket, RBracket)>::parse(input).unwrap();
        let (_, input) = <(LThan, GThan)>::parse(input).unwrap();
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_matches_oct() {
        let input = "(){}[]<>";
        let (_, input) = <(
            LParen,
            RParen,
            LBrace,
            RBrace,
            LBracket,
            RBracket,
            LThan,
            GThan,
        )>::parse(input)
        .unwrap();
        assert_eq!(input, "")
    }
}
