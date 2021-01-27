pub mod impls;
pub mod token;
pub mod tuple;

use std::error::Error;

pub use impls::Vec1;

pub use nommy_derive::Parse;

pub trait Parse: Sized {
    type Error: Error;

    fn parse(input: &str) -> Result<(Self, &str), Self::Error>;
}

pub trait Process {
    type Output;

    fn process(self) -> Self::Output;
}
