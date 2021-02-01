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

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"struct">)]
#[nommy(ignore_whitespace)]
struct StructUnnamed {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<"(">, suffix = Tag<")">)]
    fields: Vec<UnnamedField>,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(suffix = Option<Tag<",">>)]
#[nommy(ignore_whitespace = "all")]
struct UnnamedField {
    #[nommy(parser = Letters)]
    ty: String,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(prefix = Tag<"enum">)]
#[nommy(ignore_whitespace)]
struct Enum {
    #[nommy(parser = Letters)]
    name: String,

    #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
    variants: Vec<Variant>,
}

#[derive(Debug, Parse, PartialEq)]
#[nommy(ignore_whitespace = "all")]
struct Variant {
    #[nommy(parser = Letters)]
    name: String,

    ty: VariantType,
}
#[derive(Debug, Parse, PartialEq)]
enum VariantType {
    #[nommy(ignore_whitespace = "all")]
    Struct(#[nommy(prefix = Tag<"{">, suffix = Tag<"}">)] Vec<NamedField>),

    #[nommy(suffix = Tag<",">)]
    #[nommy(ignore_whitespace = "all")]
    Tuple(#[nommy(prefix = Tag<"(">, suffix = Tag<")">)] Vec<UnnamedField>),
    
    #[nommy(suffix = Tag<",">)]
    #[nommy(ignore_whitespace = "all")]
    Unit,
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

    let input = "struct Foo (Abc, Xyz,)";

    let struct_: StructUnnamed = parse(input.chars()).unwrap();
    assert_eq!(
        struct_,
        StructUnnamed {
            name: "Foo".to_string(),
            fields: vec![
                UnnamedField {
                    ty: "Abc".to_string(),
                },
                UnnamedField {
                    ty: "Xyz".to_string(),
                },
            ]
        }
    );

    let input = "enum Foo {
        Abc(Bar, Baz),
        Xyz{
            bar: Bar,
            baz: Baz,
        }
        Unit,
    }";

    let enum_: Enum = parse(input.chars()).unwrap();
    assert_eq!(
        enum_,
        Enum {
            name: "Foo".to_string(),
            variants: vec![
                Variant {
                    name: "Abc".to_string(),
                    ty: VariantType::Tuple(vec![
                        UnnamedField {
                            ty: "Bar".to_string(),
                        },
                        UnnamedField {
                            ty: "Baz".to_string(),
                        },
                    ])
                },
                Variant {
                    name: "Xyz".to_string(),
                    ty: VariantType::Struct(vec![
                        NamedField {
                            name: "bar".to_string(),
                            ty: "Bar".to_string(),
                        },
                        NamedField {
                            name: "baz".to_string(),
                            ty: "Baz".to_string(),
                        },
                    ])
                },
                Variant {
                    name: "Unit".to_string(),
                    ty: VariantType::Unit,
                },
            ],
        }
    );
}
