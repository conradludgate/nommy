# Peekers

Hidden in the [`Parse`] definition in the previous chapter is the `peek` method. It's definition is almost exactly the same as `parse`, but instead of returning `Result<Self>`, it returns `bool`. It's supposed to be a faster method of determining whether a given input could be parsed. A lot of the built in parsers utilise [`peek`] under the hood to resolve branches.

```rust
pub trait Parse<T>: Sized {
    #fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self>;
    #
    fn peek(input: &mut impl Buffer<T>) -> bool {
        // Default impl - override for better performance
        Self::parse(input).is_ok()
    }
}
```

## Example

This is the same example from the [`Parsers`] section, but instead implementing `peek`.
It follows a very similar implementation, but avoids a lot of the heavy work, such as dealing with errors
and saving the chars to the string buffer

```rust
/// StringParser parses a code representation of a string
struct StringParser(String);
impl Parse<char> for StringParser {
    #fn parse(input: &mut impl Buffer<char>) -> Result<Self> {
        #unimplemented!()
    #}
    #
    fn peek(input: &mut impl Buffer<char>) -> bool {
        // ensure the first character is a quote mark
        if input.next() != Some('\"') {
            return false;
        }

        let mut escaped = false;

        // read from the input until the ending quote is found
        for c in input {
            match (c, escaped) {
                ('\"', true) => { escaped = false }
                ('n', true) => { escaped = false }
                ('r', true) => { escaped = false }
                ('t', true) => { escaped = false }
                ('\\', true) => { escaped = false }
                (c, true) => return false,
                ('\"', false) => return true,
                ('\\', false) => { escaped = true; }
                _ => {},
            }
        }

        false
    }
}
```


[`Buffer`]: https://docs.rs/nommy/latest/nommy/trait.Buffer.html
[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
[`Parsers`]: parsers.html
