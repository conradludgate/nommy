#![feature(array_map)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]
#![allow(incomplete_features)]
#![feature(const_generics)]

//! Type based parsing library
//!
//! ```
//! use nommy::{parse, text::*, Parse};
//!
//! type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;
//!
//! #[derive(Debug, Parse, PartialEq)]
//! #[nommy(prefix = Tag<"struct">)]
//! #[nommy(ignore_whitespace)]
//! struct StructNamed {
//!     #[nommy(parser = Letters)]
//!     name: String,
//!
//!     #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
//!     fields: Vec<NamedField>,
//! }
//!
//! #[derive(Debug, Parse, PartialEq)]
//! #[nommy(suffix = Tag<",">)]
//! #[nommy(ignore_whitespace = "all")]
//! struct NamedField {
//!     #[nommy(parser = Letters)]
//!     name: String,
//!
//!     #[nommy(prefix = Tag<":">, parser = Letters)]
//!     ty: String,
//! }
//! let input = "struct Foo {
//!     bar: Abc,
//!     baz: Xyz,
//! }";
//!
//! let struct_: StructNamed = parse(input.chars()).unwrap();
//! assert_eq!(
//!     struct_,
//!     StructNamed {
//!         name: "Foo".to_string(),
//!         fields: vec![
//!             NamedField {
//!                 name: "bar".to_string(),
//!                 ty: "Abc".to_string(),
//!             },
//!             NamedField {
//!                 name: "baz".to_string(),
//!                 ty: "Xyz".to_string(),
//!             },
//!         ]
//!     }
//! );
//! ```

pub mod impls;
pub mod surrounded;
// pub mod tuple;
pub mod bytes;
pub mod text;

use std::collections::VecDeque;

use eyre::Context;
pub use impls::Vec1;

/// Derive Parse for structs or enums
///
/// ```
/// use nommy::{parse, text::*, Parse};
///
/// type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;
///
/// #[derive(Debug, Parse, PartialEq)]
/// #[nommy(prefix = Tag<"struct">)]
/// #[nommy(ignore_whitespace)]
/// struct StructNamed {
///     #[nommy(parser = Letters)]
///     name: String,
///
///     #[nommy(prefix = Tag<"{">, suffix = Tag<"}">)]
///     fields: Vec<NamedField>,
/// }
///
/// #[derive(Debug, Parse, PartialEq)]
/// #[nommy(suffix = Tag<",">)]
/// #[nommy(ignore_whitespace = "all")]
/// struct NamedField {
///     #[nommy(parser = Letters)]
///     name: String,
///
///     #[nommy(prefix = Tag<":">, parser = Letters)]
///     ty: String,
/// }
/// let input = "struct Foo {
///     bar: Abc,
///     baz: Xyz,
/// }";
///
/// let struct_: StructNamed = parse(input.chars()).unwrap();
/// assert_eq!(
///     struct_,
///     StructNamed {
///         name: "Foo".to_string(),
///         fields: vec![
///             NamedField {
///                 name: "bar".to_string(),
///                 ty: "Abc".to_string(),
///             },
///             NamedField {
///                 name: "baz".to_string(),
///                 ty: "Xyz".to_string(),
///             },
///         ]
///     }
/// );
/// ```
pub use nommy_derive::Parse;

pub use eyre;

/// parse takes the given iterator, putting it through `P::parse`
///
/// ```
/// use nommy::{parse, text::Tag};
/// let dot: Tag<"."> = parse(".".chars()).unwrap();
/// ```
pub fn parse<P: Parse<<I::Iter as Iterator>::Item>, I: IntoBuf>(iter: I) -> eyre::Result<P> {
    P::parse(&mut iter.into_buf())
}

/// parse_terminated takes the given iterator, putting it through `P::parse`,
/// erroring if the full input was not consumed
///
/// ```
/// use nommy::{parse_terminated, text::Tag};
/// let res: Result<Tag<".">, _> = parse_terminated(".".chars());
/// res.unwrap();
/// let res: Result<Tag<".">, _> = parse_terminated("..".chars());
/// res.unwrap_err();
/// ```
pub fn parse_terminated<P: Parse<<I::Iter as Iterator>::Item>, I: IntoBuf>(
    iter: I,
) -> eyre::Result<P> {
    let mut buffer = iter.into_buf();
    let output = P::parse(&mut buffer)?;
    if buffer.next().is_some() {
        Err(eyre::eyre!("input was not parsed completely"))
    } else {
        Ok(output)
    }
}

/// An interface for creating and composing parsers
/// Takes in a [Buffer] iterator and consumes a subset of it,
/// Returning Self if it managed to parse ok, otherwise returning a meaningful error
/// Parse can be derived for some types
///
/// ```
/// use nommy::{Parse, IntoBuf, text::Tag};
/// let mut buffer = ".".chars().into_buf();
/// Tag::<".">::parse(&mut buffer).unwrap();
/// ```
pub trait Parse<T>: Sized + Peek<T> {
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self>;
}

/// An interface with dealing with parser-peeking.
/// The required function [peek](Peek::peek) takes in a [Cursor] iterator
/// and will attempt to loosely parse the data provided,
/// asserting that if the equivalent [Buffer] is given to
/// the [Parse::parse] function, it should succeed.
///
/// ```
/// use nommy::{Peek, Buffer, IntoBuf, text::Tag};
/// let mut buffer = ".".chars().into_buf();
/// assert!(Tag::<".">::peek(&mut buffer.cursor()));
/// ```
pub trait Peek<T>: Sized {
    fn peek(input: &mut impl Buffer<T>) -> bool;
}

/// Process is a standard interface to map a generated AST from the output of [Parse::parse].
/// All types that implement [Parse] should implement this trait.
pub trait Process {
    type Output;

    fn process(self) -> Self::Output;
}

pub trait Buffer<T>: Iterator<Item = T> {
    type Iter: Iterator<Item = T>;
    fn cursor(&mut self) -> Cursor<Self::Iter>;
    fn fast_forward(&mut self, n: usize);
}
pub trait IntoBuf {
    type Iter: Iterator;
    fn into_buf(self) -> Buf<Self::Iter>;
}
impl<I: IntoIterator> IntoBuf for I {
    type Iter = I::IntoIter;
    fn into_buf(self) -> Buf<Self::Iter> {
        Buf::new(self)
    }
}

/// Buffer is a wrapper around an [Iterator], highly linked to [Cursor]
///
/// ```
/// use nommy::{Buffer, IntoBuf};
/// let mut buffer = (0..).into_buf();
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
pub struct Buf<I: Iterator> {
    iter: I,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for Buf<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(output) = self.buffer.pop_front() {
            Some(output)
        } else {
            self.iter.next()
        }
    }
}

impl<I: Iterator> Buf<I> {
    /// Create a new Buf
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Buf {
            iter: iter.into_iter(),
            buffer: VecDeque::new(),
        }
    }
}

impl<I: Iterator> Buffer<I::Item> for Buf<I> {
    type Iter = I;
    /// Create a [Cursor] over this buffer
    fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self,
            base: 0,
            index: 0,
        }
    }

    /// Skip forward `n` steps in the iterator
    /// Often paired with [Cursor::cursor] and [Cursor::close]
    fn fast_forward(&mut self, n: usize) {
        let len = self.buffer.len();
        if len <= n {
            self.buffer.clear();
            for _ in 0..(n - len) {
                if self.iter.next().is_none() {
                    break;
                }
            }
        } else {
            self.buffer.rotate_left(n);
            self.buffer.truncate(len - n);
        }
    }
}

/// Cursors are heavily related to [Buffer]s. Refer there for documentation
pub struct Cursor<'a, I: Iterator> {
    buf: &'a mut Buf<I>,
    base: usize,
    index: usize,
}

impl<'a, I: Iterator> Buffer<I::Item> for Cursor<'a, I>
where
    I::Item: Clone,
{
    type Iter = I;
    fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self.buf,
            base: self.index + self.base,
            index: 0,
        }
    }

    /// Skip forward `n` steps in the iterator
    /// Often paired with [Cursor::cursor] and [Cursor::close]
    fn fast_forward(&mut self, n: usize) {
        self.index += n;
    }
}

impl<'a, I: Iterator> Cursor<'a, I> {
    /// Drop the Cursor, returning how many items have been read since it was created.
    pub fn close(self) -> usize {
        self.index
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
    use crate::IntoBuf;

    use super::Buffer;

    #[test]
    fn cursor_isolation() {
        let mut buffer = "something".chars().into_buf();
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
        let mut buffer = (0..).into_buf();

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
