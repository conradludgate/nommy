# Parsers

[`Parse`] is a trait that defines how to go from a [`Buffer`] to a value. It is defined as the following

```rust
pub trait Parse<T>: Sized {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self>;
    #
    # // Covered in the next section
    #fn peek(input: &mut impl Buffer<T>) -> bool {
        #Self::parse(input).is_ok()
    #}
}
```

[`Parse`] isn't much on it's own, but it's the basis around the rest of this crate. We piggy-back off of [`eyre`] for error handling, as parsers may have several nested levels of errors and handling those with specific error types can get very complicated.

## Example

This example implementation of [`Parse`] reads from a `char` [`Buffer`], parsing a representation of a string.

```rust
/// StringParser parses a code representation of a string
struct StringParser(String);
impl Parse<char> for StringParser {
    fn parse(input: &mut impl Buffer<char>) -> eyre::Result<Self> {
        // ensure the first character is a quote mark
        if input.next() != Some('\"') {
            return Err(eyre::eyre!("starting quote not found"));
        }

        let mut output = String::new();
        let mut escaped = false;

        // read from the input until the ending quote is found
        for c in input {
            match (c, escaped) {
                ('\"', true) => output.push('\"'),
                ('n', true) => output.push('\n'),
                ('r', true) => output.push('\r'),
                ('t', true) => output.push('\t'),
                ('\\', true) => output.push('\\'),
                (c, true) => return Err(eyre::eyre!("unknown escaped character code \\{}", c)),

                ('\"', false) => return Ok(Self(output)),
                ('\\', false) => {
                    escaped = true;
                    continue;
                }
                (c, false) => output.push(c),
            }
            escaped = false;
        }

        Err(eyre::eyre!("ending quote not found"))
    }
}
```

[`Buffer`]: https://docs.rs/nommy/latest/nommy/trait.Buffer.html
[`Parse`]: https://docs.rs/nommy/latest/nommy/trait.Parse.html
[`eyre`]: https://crates.io/crates/eyre
