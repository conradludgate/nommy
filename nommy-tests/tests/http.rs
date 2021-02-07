use std::str::FromStr;

use nommy::{Buffer, Parse, parse, text::{AnyOf1, Tag, WhileNot1, Space}};

type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_">;
type Path = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ/%-_+1234567890">;
type Digits = AnyOf1<"0123456789">;

pub struct Number(usize);
impl Parse<char> for Number
{
    fn parse(input: &mut impl Buffer<char>) -> nommy::eyre::Result<Self> {
        let digits = Digits::parse(input)?;
        let string: String = digits.into();
        let u = usize::from_str(&string)?;
        Ok(Number(u))
    }

    fn peek(input: &mut impl Buffer<char>) -> bool {
        Digits::peek(input)
    }
}
impl Into<usize> for Number {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Parse)]
struct HTTP {
    #[nommy(parser = Letters)]
    method: String,

    #[nommy(parser = Path)]
    #[nommy(prefix = Tag<" ">, suffix = Tag<" ">)]
    path: String,

    #[nommy(prefix = Tag<"HTTP/">)]
    #[nommy(parser = Number)]
    version_major: usize,
    #[nommy(prefix = Tag<".">)]
    #[nommy(parser = Number)]
    version_minor: usize,

    headers: Vec<Header>,
}


#[derive(Debug, Parse)]
#[nommy(ignore = Space)]
#[nommy(prefix = Tag<"\n">)]
struct Header {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<":">)]
    #[nommy(inner_parser = HeaderValue)]
    values: Vec<String>
}

#[derive(Debug, Parse)]
#[nommy(ignore = Space)]
#[nommy(suffix = Option<Tag<",">>)]
struct HeaderValue {
    #[nommy(parser = WhileNot1<",\n">)]
    value: String,
}

impl Into<String> for HeaderValue {
    fn into(self) -> String {
        self.value
    }
}

fn main() {
    let input = "GET / HTTP/1.1
Host: www.reddit.com
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.8; rv:15.0) Gecko/20100101 Firefox/15.0.1
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
Accept-Language: en-us,en;q=0.5
Accept-Encoding: gzip, deflate
Connection: keep-alive

";

    let http: HTTP = parse(input.chars()).unwrap();
    println!("{:?}", http);
}
