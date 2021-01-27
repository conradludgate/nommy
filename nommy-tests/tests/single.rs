use nommy::{
    token::{LParen},
    Parse,
};

#[derive(Debug, Parse, PartialEq)]
struct Single {
    only: LParen,
}

fn main() {
    let (output, input) = Single::parse("(.").unwrap();
    assert_eq!(input, ".");
    assert_eq!(output, Single { only: LParen });

    let err = Single::parse(".").unwrap_err();
    assert_eq!(format!("{}", err), "could not parse field `only`: error parsing tag. expected: `(`, got: `.`");
}
