use nommy::{parse, token::LParen, Parse};

#[derive(Debug, Parse, PartialEq)]
struct Single {
    only: LParen,
}

fn main() {
    let output: Single = parse("(.".chars()).unwrap();
    assert_eq!(output, Single { only: LParen });

    let res: Result<Single, _> = parse(".".chars());
    assert_eq!(
        format!("{}", res.unwrap_err()),
        "could not parse field `only`: error parsing tag `(`"
    );
}
