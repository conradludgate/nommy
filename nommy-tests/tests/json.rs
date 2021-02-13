use nommy::{text::*, *};

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
enum JSON {
    #[nommy(prefix = Tag<"null">)]
    Null,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    Object(
        #[nommy(inner_parser = Record)]
        #[nommy(seperated_by = Tag<",">)]
        Vec<Record>
    ),

    #[nommy(prefix = Tag<"[">, suffix = Tag<"]">)]
    List(
        #[nommy(inner_parser = JSON)]
        #[nommy(seperated_by = Tag<",">)]
        Vec<JSON>
    ),

    String(#[nommy(parser = StringParser)] String),
    // Num(f64),
}

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
struct Record {
    #[nommy(parser = StringParser)]
    #[nommy(suffix = Tag<":">)]
    name: String,

    value: JSON,
}

struct StringParser(String);
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
                ('\"', false) => break,
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

    fn peek(input: &mut impl Buffer<char>) -> bool {
        if input.next() != Some('\"') {
            return false;
        }

        let mut escaped = false;
        for c in input {
            match (c, escaped) {
                ('\"', true) => escaped = false,
                ('n', true) => escaped = false,
                ('r', true) => escaped = false,
                ('t', true) => escaped = false,
                ('\\', true) => escaped = false,
                (_, true) => return false,
                ('\"', false) => return true,
                ('\\', false) => {
                    escaped = true;
                }
                _ => {}
            }
        }

        false
    }
}
impl Into<String> for StringParser {
    fn into(self) -> String {
        self.0
    }
}

fn main() {
    let json_input = r#"{
        "foo": "bar",
        "baz": {
            "hello": ["world"]
        }
    }"#;

    let json: JSON = parse(json_input.chars()).unwrap();
    println!("{:?}", json);
}
