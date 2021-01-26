use nommy::{token::LParen, Parse};

#[derive(Debug, Parse, PartialEq)]
struct Single {
    only: LParen,
}

fn main() {
    let (output, input) = Single::parse("(.").unwrap();
    assert_eq!(input, ".");
    assert_eq!(output, Single { only: LParen });

    let err = Single::parse(".").unwrap_err();
    assert!(format!("{}", err).starts_with("could not parse field `only`:"));
}
