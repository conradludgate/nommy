use nommy::{Parse, parse, token::{LParen, RParen}};

#[derive(Debug, Parse, PartialEq)]
struct Multiple {
    left: LParen,
    right: RParen,
}

fn main() {
    let output: Multiple = parse("().".chars()).unwrap();
    assert_eq!(
        output,
        Multiple {
            left: LParen,
            right: RParen
        }
    );

    let res: Result<Multiple, _> = parse(".".chars());
    assert_eq!(
        format!("{}", res.unwrap_err()),
        "could not parse field `left`: error parsing tag `(`"
    );

    let res: Result<Multiple, _> = parse("(.".chars());
    assert_eq!(
        format!("{}", res.unwrap_err()),
        "could not parse field `right`: error parsing tag `)`"
    );
}
