use nommy::{Parse, parse, text::Tag};

#[derive(Debug, Parse, PartialEq)]
struct Multiple {
    left: Tag<"(">,
    right: Tag<")">,
}

fn main() {
    let output: Multiple = parse("().".chars()).unwrap();
    assert_eq!(
        output,
        Multiple {
            left: Tag::<"(">,
            right: Tag::<")">,
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
