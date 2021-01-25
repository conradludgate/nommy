use crate::{Parse, Process};
use std::iter::Peekable;

macro_rules! Tuple {
    ($error:ident: $($T:ident),*) => {

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum $error<$($T),*> {
    $(
        $T($T),
    )*
}

impl<$($T),*> Parse for ($($T),*) where $($T: Parse),* {
    type Error = $error<$($T::Error),*>;
    fn parse(input: &mut Peekable<impl Iterator<Item=char>>) -> Result<Self, Self::Error> {
        Ok((
            $(
                match $T::parse(input) {
                    Ok(t) => t,
                    Err(e) => return Err($error::$T(e)),
                },
            )*
        ))
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
Tuple!(Error2: T1, T2);
Tuple!(Error3: T1, T2, T3);
Tuple!(Error4: T1, T2, T3, T4);
Tuple!(Error5: T1, T2, T3, T4, T5);
Tuple!(Error6: T1, T2, T3, T4, T5, T6);
Tuple!(Error7: T1, T2, T3, T4, T5, T6, T7);
Tuple!(Error8: T1, T2, T3, T4, T5, T6, T7, T8);
Tuple!(Error9: T1, T2, T3, T4, T5, T6, T7, T8, T9);
Tuple!(Error10: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
Tuple!(Error11: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
Tuple!(Error12: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
Tuple!(Error13: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
Tuple!(Error14: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
Tuple!(Error15: T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);


#[cfg(test)]
mod tests {
    use crate::{Parse};
    use crate::token::*;

    #[test]
    fn test_parse_matches_pairs() {
        let mut input = "(){}[]<>".chars().peekable();
        assert_eq!(<(LParen, RParen)>::parse(&mut input), Ok((LParen, RParen)));
        assert_eq!(<(LBrace, RBrace)>::parse(&mut input), Ok((LBrace, RBrace)));
        assert_eq!(<(LBracket, RBracket)>::parse(&mut input), Ok((LBracket, RBracket)));
        assert_eq!(<(LThan, GThan)>::parse(&mut input), Ok((LThan, GThan)));
        assert_eq!(input.next(), None)
    }

    #[test]
    fn test_parse_matches_oct() {
        let mut input = "(){}[]<>".chars().peekable();
        assert_eq!(<(
            LParen, RParen,
            LBrace, RBrace,
            LBracket, RBracket,
            LThan, GThan,
        )>::parse(&mut input), Ok((
            LParen, RParen,
            LBrace, RBrace,
            LBracket, RBracket,
            LThan, GThan,
        )));
        assert_eq!(input.next(), None)
    }
}
