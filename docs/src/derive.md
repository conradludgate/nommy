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

## Options

The following sections will cover the derive macro in more detail, but before then, let's cover the structure of how to configure the macro.

The [`derive(Parse)`] macro makes use of the `nommy` attribute to configure how the [`Parse`] implementation is created. There are two different types of attribute. `type attributes` and `field attributes`.

Refer to the examples below to understand the difference

```rust
#use nommy::{Parse, text::Tag};

/// Named struct FooBar
#[derive(Parse)]
#[nommy("i am a type attribute")]
#[nommy("i am another type attribute")]
pub struct FooBar {
    #[nommy("i am a field attribute")]
    foo: Tag<"foo">,

    #[nommy("i am also a field attribute")]
    #[nommy("i am another field attribute")]
    bar: Tag<"bar">,
}

/// Tuple struct Baz123
#[derive(Parse)]
#[nommy("i am a type attribute")]
#[nommy("i am another type attribute")]
pub struct Baz123 (
    #[nommy("i am a field attribute")]
    Tag<"baz">,

    #[nommy("i am also a field attribute")]
    #[nommy("i am another field attribute")]
    Tag<"123">,
);

/// Enum struct FooBarBaz123
#[derive(Parse)]
#[nommy("i am a type attribute")]
#[nommy("i am another type attribute")]
pub struct FooBarBaz123 (
    /// named variants are supported
    #[nommy("for now, variants also are type attributes")]
    FooBar{
        #[nommy("i am a field attribute")]
        foo: Tag<"foo">,

        #[nommy("i am also a field attribute")]
        #[nommy("i am another field attribute")]
        bar: Tag<"bar">,
    },

    /// tuple variants are supported
    #[nommy("for now, variants also are type attributes")]
    Baz123(
        #[nommy("i am a field attribute")]
        Tag<"baz">,

        #[nommy("i am also a field attribute")]
        #[nommy("i am another field attribute")]
        Tag<"123">,
    ),

    /// unit variants are also supported
    #[nommy("for now, variants also are type attributes")]
    None,
);
```

[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
[`derive(Parse)`]: https://docs.rs/nommy/latest/nommy/derive.Parse.html
