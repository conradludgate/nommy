use nommy::{parse, text::Tag, Parse};

#[derive(Debug, Parse, PartialEq)]
struct Single {
    only: Tag<"(">,
}

fn main() {
    let output: Single = parse("(.".chars()).unwrap();
    assert_eq!(output, Single { only: Tag::<"("> });

    let res: Result<Single, _> = parse(".".chars());
    assert_eq!(
        format!("{:?}", res.unwrap_err()),
        "could not parse field `only`

Caused by:
    failed to parse tag \"(\"

Location:
    /home/oon/code/rust/parser-proc-macro/nommy/src/text/tag.rs:32:17"
    );
}
