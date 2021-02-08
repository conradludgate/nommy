# Derive Parse

If writing [`Parse`] `impl`s is getting a bit tiresome for some basic situations,
you can make use of the derive macro provided to implement a lot of standard situations

```rust
#use nommy::{IntoBuf, Parse, text::Tag};
#[derive(Parse)]
pub struct FooBar {
    foo: Tag<"foo">,
    bar: Tag<"bar">,
}

let mut buffer = "foobar".chars().into_buf();
assert!(FooBar::peek(&mut buffer));
assert!(buffer.next().is_none());
```

[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
