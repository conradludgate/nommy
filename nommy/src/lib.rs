pub mod token;
pub mod tuple;
pub use thiserror;
pub use nommy_derive::Parse;

pub trait Parse: Sized {
    type Error;

    fn parse(input: &str) -> Result<(Self, &str), Self::Error>;
}

pub trait Process {
    type Output;

    fn process(self) -> Self::Output;
}
