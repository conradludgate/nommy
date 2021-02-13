#![allow(incomplete_features)]
#![feature(array_map)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_generics)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Type based parsing library
//!
//! ```
//! use nommy::{parse, text::*, Parse};
//!
//! type Letters = AnyOf1<"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ">;
//!
//! #[derive(Debug, Parse, PartialEq)]
//! #[nommy(prefix = Tag<"struct">)]
//! #[nommy(ignore = WhiteSpace)]
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
//! #[nommy(ignore = WhiteSpace)]
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
pub mod bytes;
mod impls;
pub mod text;
pub mod vec;

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
/// #[nommy(ignore = WhiteSpace)]
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
/// #[nommy(ignore = WhiteSpace)]
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

/// `parse` takes the given iterator, putting it through [`P::parse`](Parse::parse)
///
/// ```
/// use nommy::{parse, text::Tag};
/// let dot: Tag<"."> = parse(".".chars()).unwrap();
/// ```
///
/// # Errors
/// If `P` failed to parse the input at any point, that error will
/// be propagated up the chain.
pub fn parse<P, I>(iter: I) -> eyre::Result<P>
where
    P: Parse<<I::Iter as Iterator>::Item>,
    I: IntoBuf,
    <I::Iter as Iterator>::Item: Clone,
{
    let mut buffer = iter.into_buf();
    P::parse(&mut buffer)
}

/// `parse_terminated` takes the given iterator, putting it through [`P::parse`](Parse::parse),
/// erroring if the full input was not consumed
///
/// ```
/// use nommy::{parse_terminated, text::Tag};
/// let res: Result<Tag<".">, _> = parse_terminated(".".chars());
/// res.unwrap();
/// let res: Result<Tag<".">, _> = parse_terminated("..".chars());
/// res.unwrap_err();
/// ```
///
/// # Errors
/// If `P` failed to parse the input at any point, that error will
/// be propagated up the chain.
///
/// Will also error if the input is not empty after parsing
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
/// Takes in a [`Buffer`] iterator and consumes a subset of it,
/// Returning Self if it managed to parse ok, otherwise returning a meaningful error
/// Parse can be derived for some types
///
/// ```
/// use nommy::{Parse, IntoBuf, text::Tag};
/// let mut buffer = ".".chars().into_buf();
/// Tag::<".">::parse(&mut buffer).unwrap();
/// ```
pub trait Parse<T>: Sized {
    /// Parse the input buffer, returning Ok if the value could be parsed,
    /// Otherwise, returns a meaningful error
    ///
    /// # Errors
    /// Will return an error if the parser fails to interpret the input at any point
    fn parse(input: &mut impl Buffer<T>) -> eyre::Result<Self>;

    /// Peek reads the input buffer, returning true if the value could be found,
    /// Otherwise, returns false.
    /// Not required, but usually provides better performance if implemented
    fn peek(input: &mut impl Buffer<T>) -> bool {
        Self::parse(input).is_ok()
    }
}
