# Enum

An `enum`s parser attempts to parse each `variant`. The first `variant` that succeeds to parse is the `variant` that is returned. If no `variant` could be parsed,
the parser returns an error

## Example

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub enum FooOrBar {
    Foo(Tag<"foo">),
    Bar(Tag<"bar">),
}
```

This can either parse `"foo"` or `"bar"`, but not both.

## First come first serve

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub enum OnlyFoo {
    Foo(Tag<"foo">),
    Foob(Tag<"foob">),
}
```

In this example, the `OnlyFoo` enum can never parse into a variant of `Foob`. To see why, let's put in the input `"foob"`.

Since `enum` parsers try to parse each variant in order, it will first try to parse the `Foo` variant. This will match the input `"foo"`, and that is indeed found in the input sequence, therefore the result is `OnlyFoo::Foo` and the input sequence will have `'b'` remaining.

One way to solve this is to swap the order, however that might not always be possible. It might be possible to configure greedy evaluation in the future, however that is currently not possible.

## Variant types

There are 3 types of variant in a rust `enum`. These are analagous to the [`struct`]s described in the previous chapter.

```rust
#use nommy::{Parse};
#[derive(Parse)]
pub enum ExampleEnum {
    NamedVariant{
        foo: Tag<"foo">,
        bar: Tag<"bar">,
    },

    UnnamedVariant(
        Tag<"foo">,
        Tag<"bar">,
    ),

    UnitVariant,
}
```


[`struct`]: struct.html
