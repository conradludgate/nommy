# nommy

**nommy** is a type based parsing crate that features a derive macro to help you utilise the power of rust and nommy

```rust
use nommy::{parse, text::*, Parse};

type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"struct">)]
#[nommy(ignore = WhiteSpace)]
struct StructNamed {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    fields: Vec<NamedField>,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(suffix = Tag<",">)]
#[nommy(ignore = WhiteSpace)]
struct NamedField {
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
    assert_eq!(
        struct_,
        StructNamed {
            name: "Foo".to_string(),
            fields: vec![
                NamedField {
                    name: "bar".to_string(),
                    ty: "Abc".to_string(),
                },
                NamedField {
                    name: "baz".to_string(),
                    ty: "Xyz".to_string(),
                },
            ]
        }
    );
}
```
