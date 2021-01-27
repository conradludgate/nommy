pub mod impls;
pub mod surrounded;
pub mod token;
pub mod tuple;

use std::{collections::VecDeque, error::Error};

pub use impls::Vec1;

/// Derive Parse for structs
///
/// ```
/// use nommy::{parse, Parse, Tag};
///
/// Tag![Foo: "foo", Bar: "bar"];
///
/// #[derive(Parse)]
/// struct FooBar {
///     foo: Foo,
///     bar: Bar,
/// }
///
/// let _: FooBar = parse("foobar".chars()).unwrap();
/// ```
pub use nommy_derive::Parse;

/// parse takes the given iterator, putting it through `P::parse`
///
/// ```
/// use nommy::{parse, token::Dot};
/// let dot: Dot = parse(".".chars()).unwrap();
/// ```
pub fn parse<P: Parse<I::Item>, I: IntoIterator>(iter: I) -> Result<P, P::Error> {
    P::parse(&mut Buffer::new(iter))
}

/// An interface for creating and composing parsers
/// Takes in a [Buffer] iterator and consumes a subset of it,
/// Returning Self if it managed to parse ok, otherwise returning a meaningful error
/// Parse can be derived for some types
///
/// ```
/// use nommy::*;
/// let mut buffer = Buffer::new(".".chars());
/// assert_eq!(token::Dot::parse(&mut buffer), Ok(token::Dot));
/// ```
pub trait Parse<T>: Sized + Peek<T> {
    type Error: Error;

    fn parse(input: &mut Buffer<impl Iterator<Item = T>>) -> Result<Self, Self::Error>;
}

/// An interface with dealing with parser-peeking.
/// The required function [peek](Peek::peek) takes in a [Cursor] iterator
/// and will attempt to loosely parse the data provided,
/// asserting that if the equivalent [Buffer] is given to
/// the [Parse::parse] function, it should succeed.
///
/// ```
/// use nommy::*;
/// let mut buffer = Buffer::new(".".chars());
/// assert!(token::Dot::peek(&mut buffer.cursor()));
/// ```
pub trait Peek<T>: Sized {
    fn peek(input: &mut Cursor<impl Iterator<Item = T>>) -> bool;
}

/// Process is a standard interface to map a generated AST from the output of [Parse::parse].
/// All types that implement [Parse] should implement this trait.
pub trait Process {
    type Output;

    fn process(self) -> Self::Output;
}

/// Buffer is a wrapper around an [Iterator], highly linked to [Cursor]
///
/// ```
/// use nommy::Buffer;
/// let mut buffer = Buffer::new(0..);
/// let mut cursor1 = buffer.cursor();
///
/// // cursors act exactly like an iterator
/// assert_eq!(cursor1.next(), Some(0));
/// assert_eq!(cursor1.next(), Some(1));
///
/// // cursors can be made from other cursors
/// let mut cursor2 = cursor1.cursor();
/// assert_eq!(cursor2.next(), Some(2));
/// assert_eq!(cursor2.next(), Some(3));
///
/// // child cursors do not move the parent's iterator position
/// assert_eq!(cursor1.next(), Some(2));
///
/// // Same with the original buffer
/// assert_eq!(buffer.next(), Some(0));
/// ```
pub struct Buffer<I: Iterator> {
    iter: I,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for Buffer<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(output) = self.buffer.pop_front() {
            Some(output)
        } else {
            self.iter.next()
        }
    }
}

impl<I: Iterator> Buffer<I> {
    /// Create a new Buffer
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Buffer {
            iter: iter.into_iter(),
            buffer: VecDeque::new(),
        }
    }

    /// Create a [Cursor] over this buffer
    pub fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self,
            base: 0,
            index: 0,
        }
    }
}

/// Cursors are heavily related to [Buffer]s. Refer there for documentation
pub struct Cursor<'a, I: Iterator> {
    buf: &'a mut Buffer<I>,
    base: usize,
    index: usize,
}

impl<'a, I: Iterator> Cursor<'a, I> {
    /// Create a new cursor, starting from where this cursor left off
    pub fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self.buf,
            base: self.index,
            index: 0,
        }
    }

    /// Drop the Cursor, returning how many items have been read since it was created.
    pub fn close(self) -> usize {
        self.index
    }

    /// Skip forward `n` steps in the iterator
    /// Often paired with [Cursor::cursor] and [Cursor::close]
    pub fn fast_forward(&mut self, n: usize) {
        self.index += n;
    }
}

impl<'a, I: Iterator> Iterator for Cursor<'a, I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.base + self.index;
        let output = if i < self.buf.buffer.len() {
            self.buf.buffer[i].clone()
        } else {
            let diff = i - self.buf.buffer.len();
            for _ in 0..diff {
                let cache = self.buf.iter.next()?;
                self.buf.buffer.push_back(cache);
            }
            let output = self.buf.iter.next()?;
            self.buf.buffer.push_back(output.clone());
            output
        };

        self.index += 1;
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn cursor_isolation() {
        let mut buffer = Buffer::new("something".chars());
        {
            let mut cursor1 = buffer.cursor();
            assert_eq!(cursor1.next(), Some('s'));

            {
                let mut cursor2 = cursor1.cursor();
                assert_eq!(cursor2.next(), Some('o'));
                assert_eq!(cursor2.next(), Some('m'));
            }

            assert_eq!(cursor1.next(), Some('o'));
        }

        assert_eq!(buffer.next(), Some('s'));
        assert_eq!(buffer.next(), Some('o'));
        assert_eq!(buffer.next(), Some('m'));

        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn cursor_fast_forward() {
        let mut buffer = Buffer::new(0..);

        let mut cursor = buffer.cursor();
        cursor.fast_forward(2);

        assert_eq!(cursor.next(), Some(2));
        assert_eq!(cursor.next(), Some(3));

        assert_eq!(buffer.next(), Some(0));
        assert_eq!(buffer.next(), Some(1));
        assert_eq!(buffer.next(), Some(2));
        assert_eq!(buffer.next(), Some(3));

        assert!(buffer.buffer.is_empty());
    }
}
