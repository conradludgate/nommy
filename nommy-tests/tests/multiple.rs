use nommy::{Parse, parse, text::Tag, eyre::Result};

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

    let res: Result<Multiple> = parse(".".chars());
    assert_eq!(
        format!("{}", res.unwrap_err()),
        "failed to parse field `left`"
    );

    let res: Result<Multiple> = parse("(.".chars());
    assert_eq!(
        format!("{}", res.unwrap_err()),
        "failed to parse field `right`"
    );
}
