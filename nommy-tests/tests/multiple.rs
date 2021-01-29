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
        format!("{:?}", res.unwrap_err()),
        "could not parse field `left`

Caused by:
    failed to parse tag \"(\", found \".\"

Location:
    /home/oon/code/rust/parser-proc-macro/nommy/src/text/tag.rs:35:17"
    );

    let res: Result<Multiple> = parse("(.".chars());
    assert_eq!(
        format!("{:?}", res.unwrap_err()),
        "could not parse field `right`

Caused by:
    failed to parse tag \")\", found \".\"

Location:
    /home/oon/code/rust/parser-proc-macro/nommy/src/text/tag.rs:35:17"
    );
}
