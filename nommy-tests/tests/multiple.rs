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
    assert!(format!("{}", err).starts_with("could not parse field `left`:"));

    let err = Multiple::parse("(.").unwrap_err();
    assert!(format!("{}", err).starts_with("could not parse field `right`:"));
}
