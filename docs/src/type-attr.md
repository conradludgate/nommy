# Type Attributes

There's currently only 3 supported type attributes

## Ignore

`ignore` lets you specify how to parse the tokens that you don't care about.

For example, ignoreing whitespace:

```rust
#use nommy::{Parse, IntoBuf, text::{Tag, WhiteSpace}};
#[derive(Parse)]
#[nommy(ignore = WhiteSpace)]
pub struct FooBar(
    Tag<"foo">,
    Tag<"bar">,
);

let mut buffer = "foo   bar\t".chars().into_buf();
FooBar::parse(&mut buffer).unwrap();
// ignore also parses the trailing tokens
assert!(buffer.next().is_none());
```

### Warning

If the type you give to `ignore` can parse 0 tokens, then the program will loop forever.
In the future there might be checks in place to automatically exit when empty parsers succeed (or panic?)

## Prefix/Suffix

`prefix` and `suffix` define the parser that you expect to match before we attempt to parse the value we care about.

```rust
#use nommy::{Parse, IntoBuf, text::Tag};

#[derive(Parse)]
#[nommy(prefix = Tag<"(">, suffix = Tag<")">)]
pub struct Bracketed(
    Tag<"foo">,
    Tag<"bar">,
);

let mut buffer = "(foobar)".chars().into_buf();
Bracketed::parse(&mut buffer).unwrap();
assert!(buffer.next().is_none());
```
