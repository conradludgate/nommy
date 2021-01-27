use nommy::{
    token::{LParen, RParen},
    Parse,
};

#[derive(Debug, Parse, PartialEq)]
struct Multiple {
    left: LParen,
    right: RParen,
}

fn main() {
    let (output, input) = Multiple::parse("().").unwrap();
    assert_eq!(input, ".");
    assert_eq!(
        output,
        Multiple {
            left: LParen,
            right: RParen
        }
    );

    let err = Multiple::parse(".").unwrap_err();
    assert_eq!(format!("{}", err), "could not parse field `left`: error parsing tag. expected: `(`, got: `.`");

    let err = Multiple::parse("(.").unwrap_err();
    assert_eq!(format!("{}", err), "could not parse field `right`: error parsing tag. expected: `)`, got: `.`");
}
