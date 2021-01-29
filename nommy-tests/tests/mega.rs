use nommy::{Parse, Process, parse, text::*};

type Letters = AnyOf<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"struct">)]
#[nommy(ignore_whitespace)]
struct StructNamed {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    fields: Vec<StructNamedField>,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(suffix = Tag<",">)]
#[nommy(ignore_whitespace = "all")]
struct StructNamedField {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<":">, parser = Letters)]
    ty: String,
}

fn main() {
    let input = "struct Foo {
        bar: Abc,
        baz: Xyz,
    }";

    let struct_: StructNamed = parse(input.chars()).unwrap();
    assert_eq!(struct_, StructNamed{
        name: "Foo".to_string(),
        fields: vec![
            StructNamedField{
                name: "bar".to_string(),
                ty: "Abc".to_string(),
            },
            StructNamedField{
                name: "baz".to_string(),
                ty: "Xyz".to_string(),
            },
        ]
    });
}
