use nommy::{text::*, *};

const ENDING_TAG1: &'static str = ">";
const ENDING_TAG: &'static str = "/>";
type Name = AnyOf1<"abcdefghijklmnopqrstuvwxyz-_">;

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
#[nommy(prefix = Tag<"<">, suffix = Tag<ENDING_TAG>)]
struct XML {
    #[nommy(parser = Name)]
    name: String,

    attributes: Vec<Attribute>,

    children: Children,
}

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
enum Children {
    #[nommy(prefix = Tag<ENDING_TAG1>, suffix = Tag<"<">)]
    #[nommy(min = 1)]
    Some(Vec<XML>),
    None,
}

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
struct Attribute {
    #[nommy(parser = Name)]
    #[nommy(suffix = Tag<"=">)]
    name: String,

    #[nommy(parser = Name)]
    #[nommy(prefix = Tag<"\"">, suffix = Tag<"\"">)]
    value: String,
}

fn main() {
    let json_input = r#"<foo bar="baz">
        <hello />
        <world />
    </>
    }"#;

    let xml: XML = parse(json_input.chars()).unwrap();
    println!("{:?}", xml);
}
