# Field Attributes

There's currently only 4 supported field attributes

## Parser

`parser` lets you specify how to parse the input into the type specified.

For example, parsing letters into a string:

```rust
#use nommy::{Parse, IntoBuf, text::AnyOf1};

type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyz">;

# #[derive(Debug, PartialEq)]
#[derive(Parse)]
pub struct Word (
    #[nommy(parser = Letters)]
    String,
);

let mut buffer = "foo bar".chars().into_buf();
assert_eq!(Word::parse(&mut buffer).unwrap(), Word("foo".to_string()));
```

This works because `Letters` implements `Into<String>`.

## Prefix/Suffix

`prefix` and `suffix` define the parser that you expect to match before we attempt to parse the value we care about.

```rust
#use nommy::{Parse, IntoBuf, text::{Tag, AnyOf1, Space}};

type Numbers = AnyOf1<"0123456789">;

# #[derive(Debug, PartialEq)]
#[derive(Parse)]
#[nommy(ignore = Space)]
pub struct Add(
    #[nommy(parser = Numbers)]
    String,

    #[nommy(prefix = Tag<"+">)]
    #[nommy(parser = Numbers)]
    String,
);

let mut buffer = "4 + 7".chars().into_buf();
assert_eq!(
    Add::parse(&mut buffer).unwrap(),
    Add("4".to_string(), "7".to_string()),
);
assert!(buffer.next().is_none());
```

## Inner Parser

`inner_parser` lets you specify how to parse the input into the vec type specified.

For example, parsing letters into a string:

```rust
#use nommy::{Parse, IntoBuf, text::OneOf};

type Letter = OneOf<"abcdefghijklmnopqrstuvwxyz">;

# #[derive(Debug, PartialEq)]
#[derive(Parse)]
pub struct Letters (
    #[nommy(inner_parser = Letter)]
    Vec<char>,
);

let mut buffer = "foo bar".chars().into_buf();
assert_eq!(Letters::parse(&mut buffer).unwrap(), Letters(vec!['f', 'o', 'o']));
```

This is necessary because `Vec<P>` **does not** implement `Into<Vec<Q>>` even if `P: Into<Q>`.
