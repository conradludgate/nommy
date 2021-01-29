use nommy::{Parse, parse, text::Tag, eyre::Result};

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"struct">, suffix = Tag<"Suffix">)]
#[nommy(ignore_whitespace = "spaces")]
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
    failed to parse tag \"(\"

Location:
    /home/oon/code/rust/parser-proc-macro/nommy/src/text/tag.rs:32:17"
    );

    let res: Result<Multiple> = parse("(.".chars());
    assert_eq!(
        format!("{:?}", res.unwrap_err()),
        "could not parse field `right`

Caused by:
    failed to parse tag \")\"

Location:
    /home/oon/code/rust/parser-proc-macro/nommy/src/text/tag.rs:32:17"
    );
}
