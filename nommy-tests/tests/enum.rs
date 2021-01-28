use nommy::{Buffer, Parse, text::Tag};

#[derive(Debug, Parse, PartialEq)]
enum Enum {
    Open(Tag<"(">),
    Dot{
        dot1: Tag<".">,
        dot2: Tag<".">,
        dot3: Tag<".">,
    },
    Close(Tag<")">),
}

fn main() {
    let mut input = Buffer::new("(...)".chars());

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Open(Tag::<"(">));

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Dot{
        dot1: Tag::<".">,
        dot2: Tag::<".">,
        dot3: Tag::<".">,
    });

    assert_eq!(Enum::parse(&mut input).unwrap(), Enum::Close(Tag::<")">));

    assert_eq!(input.next(), None);
}
