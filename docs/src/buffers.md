
# Buffers

[`Buffer`] is trait that wraps an `Iterator`. It extends upon this by requiring two extra methods

```rust
/// eagerly drops the first `n` elements in the buffer
fn fast_forward(&mut self, n: usize);

/// finds the `i`th element in the iterator, storing any read elements into a buffer for later access
fn peek_ahead(&mut self, i: usize) -> Option<T>
```

With these two method, the trait can implement a third method, `cursor` which returns a new [`Buffer`] type [`Cursor`].

[`Cursor`] reads from a buffer only using `peek_ahead`.
It ensures that any data read through the buffer can be read again in future.

```rust
use nommy::{Buffer, IntoBuf};
let mut buffer = (0..).into_buf();
let mut cursor1 = buffer.cursor();

// cursors act exactly like an iterator
assert_eq!(cursor1.next(), Some(0));
assert_eq!(cursor1.next(), Some(1));

// cursors can be made from other cursors
let mut cursor2 = cursor1.cursor();
assert_eq!(cursor2.next(), Some(2));
assert_eq!(cursor2.next(), Some(3));

// child cursors do not move the parent's iterator position
assert_eq!(cursor1.next(), Some(2));

assert_eq!(buffer.next(), Some(0));
```

If you read from a cursor and decide that you won't need to re-read that contents again,
you can call `fast_forward_parent`. This takes how many elements ahead the [`Cursor`] has read,
and calls the parent buffer's `fast_forward` method with it.

```rust
use nommy::{Buffer, IntoBuf};
let mut input = "foobar".chars().into_buf();
let mut cursor = input.cursor();
assert_eq!(cursor.next(), Some('f'));
assert_eq!(cursor.next(), Some('o'));
assert_eq!(cursor.next(), Some('o'));

// Typically, the next three calls to `next` would repeat
// the first three calls because cursors read non-destructively.
// However, this method allows to drop the already-read contents
cursor.fast_forward_parent();
assert_eq!(input.next(), Some('b'));
assert_eq!(input.next(), Some('a'));
assert_eq!(input.next(), Some('r'));
```

The standard implementation of [`Buffer`](Buffer) is [`Buf`], and can be created from any type that implements `IntoIterator`.

[`Buffer`]: https://docs.rs/nommy/latest/nommy/trait.Buffer.html
[`Cursor`]: https://docs.rs/nommy/latest/nommy/struct.Cursor.html
[`Buf`]: https://docs.rs/nommy/latest/nommy/struct.Buf.html
