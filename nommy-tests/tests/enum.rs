use nommy::{Buffer, Parse, text::token::{Dot, LParen, RParen}};

#[derive(Debug, Parse, PartialEq)]
enum Enum {
    Open(LParen),
    Dot{
        dot1: Dot,
        dot2: Dot,
        dot3: Dot,
    },
    Close(RParen),
}

fn main() {
    let mut input = Buffer::new("(...)".chars());

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Open(LParen));

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Dot{
        dot1: Dot,
        dot2: Dot,
        dot3: Dot,
    });

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Close(RParen));

    assert_eq!(input.next(), None);
}
