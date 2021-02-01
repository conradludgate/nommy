#![feature(trivial_bounds)]

use nommy::{text::*, *};

#[derive(Debug, PartialEq, Parse)]
#[nommy(suffix = Tag<",">, parse_type = char)]
#[nommy(ignore_whitespace)]
enum JSON {
    #[nommy(prefix = Tag<"null">)]
    Null,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    Object(Vec<Record>),

    #[nommy(prefix = Tag<"[">, suffix = Tag<"]">)]
    List(Vec<JSON>),

    String(#[nommy(parser = StringParser)] String),
    // Num(f64),
}

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore_whitespace, parse_type = char)]
struct Record {
    #[nommy(parser = StringParser)]
    #[nommy(suffix = Tag<":">)]
    name: String,

    value: JSON,
}

struct StringParser(String);
impl Peek<char> for StringParser {
    fn peek(input: &mut impl Buffer<char>) -> bool {
        if input.next() != Some('\"') {
            return false;
        }

        let mut escaped = false;
        for c in input {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\"' {
                break;
            }
        }

        true
    }
}
impl Parse<char> for StringParser {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        if input.next() != Some('\"') {
            return Err(eyre::eyre!("starting quote not found"));
        }

        let mut output = String::new();
        let mut escaped = false;
        for c in input {
            match (c, escaped) {
                ('\"', true) => output.push('\"'),
                ('n', true) => output.push('\n'),
                ('r', true) => output.push('\r'),
                ('t', true) => output.push('\t'),
                ('\\', true) => output.push('\\'),
                (c, true) => return Err(eyre::eyre!("unknown escaped character code \\{}", c)),
                ('\\', false) => {
                    escaped = true;
                    continue;
                }
                (c, false) => output.push(c),
            }
            escaped = false;
        }

        Ok(StringParser(output))
    }
}
impl Process for StringParser {
    type Output = String;
    fn process(self) -> Self::Output {
        self.0
    }
}

fn main() {
    // let fake_json = r#"
    //     {
    //         "foo": "bar",
    //         "baz": {
    //             "hello": [
    //                 "world",
    //             ],
    //         },
    //     },
    // "#;

    // let json: JSON = parse(fake_json.chars()).unwrap();
    // println!("{:?}", json);
}
