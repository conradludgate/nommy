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
//! #[nommy(ignore = WhiteSpaces)]
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
//! #[nommy(ignore = WhiteSpaces)]
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

mod buffer;
pub use buffer::*;
mod impls;
pub mod bytes;
pub mod text;

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
/// #[nommy(ignore = WhiteSpaces)]
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
/// #[nommy(ignore = WhiteSpaces)]
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
pub fn parse<P, I>(iter: I) -> eyre::Result<P>
where
    P: Parse<<I::Iter as Iterator>::Item>,
    I: IntoBuf,
    <I::Iter as Iterator>::Item: Clone,
{
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
pub fn parse_terminated<P, I>(iter: I) -> eyre::Result<P>
where
    P: Parse<<I::Iter as Iterator>::Item>,
    I: IntoBuf,
    <I::Iter as Iterator>::Item: Clone,
{
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

// /// Process is a standard interface to map a generated AST from the output of [Parse::parse].
// /// All types that implement [Parse] should implement this trait.
// pub trait Process {
//     type Output;

//     fn process(self) -> Self::Output;
// }
