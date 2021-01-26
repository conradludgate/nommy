use crate::{Parse, Process};

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
            LParen, RParen,
            LBrace, RBrace,
            LBracket, RBracket,
            LThan, GThan,
        )>::parse(input).unwrap();
        assert_eq!(input, "")
    }
}
