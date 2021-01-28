# nommy
A type based parsing library with convenient macros

```rust
use nommy::{parse, Parse, TextTag};

TextTag![Foo: "foo", Bar: "bar"];

#[derive(Parse)]
struct FooBar {
    foo: Foo,
    bar: Bar,
}

let _: FooBar = parse("foobar".chars()).unwrap();

#[derive(Parse, PartialEq, Debug)]
enum FooOrBar {
    Foo(Foo),
    Bar(Bar),
}

let output: FooOrBar = parse("foo".chars()).unwrap();
assert_eq!(output, FooOrBar::Foo(Foo));

let output: FooOrBar = parse("bar".chars()).unwrap();
assert_eq!(output, FooOrBar::Bar(Bar));
```
