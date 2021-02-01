# nommy
A type based parsing library with convenient macros

```rust
use nommy::{parse, text::*, Parse};

type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"struct">)]
#[nommy(ignore_whitespace)]
struct StructNamed {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    fields: Vec<NamedField>,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(suffix = Tag<",">)]
#[nommy(ignore_whitespace = "all")]
struct NamedField {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<":">, parser = Letters)]
    ty: String,
}
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
```
