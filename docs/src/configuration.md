# Configuration

The [`derive(Parse)`] macro makes use of the `nommy` attribute to configure how the [`Parse`] implementation is created. There are two different types of attribute. `type attributes` and `field attributes`.

If the attribute is on a `struct`, `enum` or an `enum variant` definition, then it will be a `type attribute`.
Otherwise, if the attribute is on a field definition, it will be a `field attribute`.

You can repeat many attribute blocks, or repeat attribute rules within the same attribute block, eg:

```rust,ignore
#[nommy(prefix = Tag<"(">, suffix = Tag<")">)]

// is the same as

#[nommy(prefix = Tag<"(">)]
#[nommy(suffix = Tag<")">)]
```

## Example

This code example indicates which attributes are understood as `type attributes`, and which are `field attributes`

```rust,ignore
#use nommy::{Parse, text::Tag};

/// Named struct FooBar
#[derive(Parse)]
#[nommy("TYPE", "TYPE")]
pub struct FooBar {
    #[nommy("FIELD")]
    foo: Tag<"foo">,

    #[nommy("FIELD")]
    #[nommy("FIELD")]
    bar: Tag<"bar">,
}

/// Tuple struct Baz123
#[derive(Parse)]
#[nommy("TYPE")]
#[nommy("TYPE")]
pub struct Baz123 (
    #[nommy("FIELD", "FIELD")]
    Tag<"baz">,

    #[nommy("FIELD")]
    #[nommy("FIELD")]
    Tag<"123">,
);

/// Enum struct FooBarBaz123
#[derive(Parse)]
#[nommy("TYPE")]
#[nommy("TYPE")]
pub struct FooBarBaz123 (
    #[nommy("TYPE")]
    FooBar{
        #[nommy("FIELD")]
        foo: Tag<"foo">,

        #[nommy("FIELD")]
        #[nommy("FIELD")]
        bar: Tag<"bar">,
    },

    #[nommy("TYPE")]
    Baz123(
        #[nommy("FIELD")]
        Tag<"baz">,

        #[nommy("FIELD")]
        #[nommy("FIELD")]
        Tag<"123">,
    ),

    #[nommy("TYPE")]
    None,
);
```

[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
[`derive(Parse)`]: https://docs.rs/nommy/latest/nommy/derive.Parse.html
