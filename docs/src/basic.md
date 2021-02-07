# Basic Parsers

`nommy` provides a set of basic parsers to handle a lot of standard situations. A lot of these makes use of rust's new [`const generics`]

## Tag

[`Tag`] matches an exact string or byte slice in the input buffer.

```rust
#use nommy::{IntoBuf, Parse, text::Tag};
let mut buffer = "foobar".chars().into_buf();
assert!(Tag::<"foo">::peek(&mut buffer));
assert!(Tag::<"bar">::peek(&mut buffer));
assert!(buffer.next().is_none());
```

## OneOf

[`OneOf`] matches one character or byte that is contained within the pattern string.

```rust
#use nommy::{IntoBuf, Parse, text::OneOf};
let mut buffer = "bC".chars().into_buf();
assert_eq!(OneOf::<"abcd">::parse(&mut buffer).unwrap().into(), 'b');
assert_eq!(OneOf::<"ABCD">::parse(&mut buffer).unwrap().into(), 'C');
assert!(buffer.next().is_none());
```

## AnyOf

[`AnyOf`] matches as many characters or bytes that are contained within the pattern string as possible.

```rust
#use nommy::{IntoBuf, Parse, text::AnyOf};
let mut buffer = "dbacCBAD".chars().into_buf();
assert_eq!(&AnyOf::<"abcd">::parse(&mut buffer).unwrap().into(), "dbac");
assert_eq!(&AnyOf::<"ABCD">::parse(&mut buffer).unwrap().into(), "CBAD");
assert!(buffer.next().is_none());
```

## AnyOf1

[`AnyOf1`] matches as many characters or bytes that are contained within the pattern string as possible,
requiring at least 1 value to match.

```rust
#use nommy::{IntoBuf, Parse, text::AnyOf1};
let mut buffer = "dbacCBAD".chars().into_buf();
assert_eq!(&AnyOf1::<"abcd">::parse(&mut buffer).unwrap().into(), "dbac");
assert_eq!(&AnyOf1::<"ABCD">::parse(&mut buffer).unwrap().into(), "CBAD");
assert!(buffer.next().is_none());
```

## WhileNot1

[`WhileNot1`] matches as many characters or bytes that are **not** contained within the pattern string as possible,
requiring at least 1 value to match.

```rust
#use nommy::{IntoBuf, Parse, text::WhileNot1};
let mut buffer = "hello world!".chars().into_buf();
assert_eq!(&WhileNot1::<".?!">::parse(&mut buffer).unwrap().into(), "hello world");
assert_eq!(buffer.next(), Some('!'));
```

## Vec

`Vec` parses `P` as many times as it can.

```rust
#use nommy::{IntoBuf, Parse, text::Tag};
let mut buffer = "...!".chars().into_buf();
assert_eq!(Vec::<Tag<".">>::parse(&mut buffer).unwrap().len(), 3);
assert_eq!(buffer.next(), Some('!'));
```

## Vec1

[`Vec1`] parses `P` as many times as it can, requiring at least 1 match.

```rust
#use nommy::{IntoBuf, Parse, Vec1, text::Tag};
let mut buffer = "...!".chars().into_buf();
assert_eq!(Vec1::<Tag<".">>::parse(&mut buffer).unwrap().len(), 3);

// assert_eq!(buffer.next(), Some('!'));
Vec1::<Tag<".">>::parse(&mut buffer).unwrap_err()
```

[`Tag`]: https://docs.rs/nommy/latest/nommy/text/struct.Tag.html
[`OneOf`]: https://docs.rs/nommy/latest/nommy/text/struct.OneOf.html
[`AnyOf`]: https://docs.rs/nommy/latest/nommy/text/struct.AnyOf.html
[`AnyOf1`]: https://docs.rs/nommy/latest/nommy/text/struct.AnyOf1.html
[`WhileNot1`]: https://docs.rs/nommy/latest/nommy/text/struct.WhileNot1.html
[`Vec1`]: https://docs.rs/nommy/latest/nommy/struct.Vec1.html
[`const generics`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/const-generics.html
