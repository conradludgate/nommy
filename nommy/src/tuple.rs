use crate::{Buffer, Cursor, Parse, Peek, Process};
use std::{error::Error, fmt};

macro_rules! Tuple {
    ($error:ident: $($T:ident),*) => {

#[derive(Debug, Copy, Clone, PartialEq)]
/// Error type returned from failing to parse a tuple.
/// Variant will correspond to the parser that failed
pub enum $error<$($T),*> where $($T: Error),* {
    $(
        $T($T),
    )*
}

impl<$($T: Error),*> Error for $error<$($T),*>  {}
impl<$($T: Error),*> fmt::Display for $error<$($T),*>  {
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

/// Implements Peek over a tuple. Returns true if all elements
/// of the tuple return true, in order
impl<T, $($T),*> Peek<T> for ($($T),*) where $($T: Peek<T>),* {
    fn peek(input: &mut impl Buffer<T>) -> bool {
        $(
            $T::peek(input)
        )&&*
    }
}

/// Implements Parse over a tuple. Parses all elements of the tuple
/// in order
impl<T, $($T),*> Parse<T> for ($($T),*) where $($T: Parse<T>),* {
    type Error = $error<$($T::Error),*>;

    fn parse(input: &mut impl Buffer<T>) -> Result<Self, Self::Error> {
        Ok(($(
            $T::parse(input).map_err($error::$T)?,
        )*))
    }
}

/// Implements Process over a tuple. Result is [Process::process] applied to all the elements of the tuple
impl<$($T),*> Process for ($($T),*) where $($T: Process),* {
    type Output = ($($T::Output),*);

    fn process(self) -> Self::Output {
        #[allow(non_snake_case)]
        let (
            $($T,)*
        ) = self;
        (
            $($T.into(),)*
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
    use crate::{parse, text::Tag, Buffer, Parse};

    #[test]
    fn test_parse_matches_pairs() {
        let mut input = Buffer::new("(){}".chars());
        <(Tag<"(">, Tag<")">)>::parse(&mut input).unwrap();
        <(Tag<"{">, Tag<"}">)>::parse(&mut input).unwrap();
        assert!(input.next().is_none());
    }

    #[test]
    fn test_parse_matches_oct() {
        let mut input = Buffer::new("(){}".chars());
        <(
            Tag<"(">,
            Tag<")">,
            Tag<"{">,
            Tag<"}">,
        )>::parse(&mut input)
        .unwrap();
        assert!(input.next().is_none());
    }

    #[test]
    fn test_parse_matches_oct_error() {
        let mut input = Buffer::new("(){.".chars());
        let res: Result<
            (
                Tag<"(">,
                Tag<")">,
                Tag<"{">,
                Tag<"}">,
            ),
            _,
        > = parse(&mut input);
        assert_eq!(
            format!("{}", res.unwrap_err()),
            "error parsing term 4: error parsing tag \"}\""
        );
    }
}
