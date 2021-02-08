# Struct

There are 3 different types of `struct` in rust, all of them are supported by [`derive(Parse)`] in a very similar way

## Named Struct

This is the standard `struct` that people think about,

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub struct FooBar {
    foo: Tag<"foo">,
    bar: Tag<"bar">,
}
```

This will parse the text `"foo"`, then the text `"bar"`. Order matters. If any single field returns an error when parsing, then the struct returns an error too.

## Unnamed/Tuple Struct

Rust also provides `unnamed struct`s that are essentially the same, but have unnamed fields

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub struct FooBar (
    Tag<"foo">,
    Tag<"bar">,
);
```

This parses exactly the same as the named variety

## Unit Struct

Lastly, rust provides unit structs. While these may seem useless in parsing, they do have uses when you configure how the macro should implement [`Parse`].

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub struct Unit;
```

This currently parses nothing, the [configuration] section lets you expand the functionality.

[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
[`derive(Parse)`]: https://docs.rs/nommy/latest/nommy/derive.Parse.html
[configuration]: configuration.html
