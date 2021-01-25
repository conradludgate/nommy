use std::iter::Peekable;

pub mod token;
pub mod tuple;

pub trait Parse: Sized {
    type Error;

    fn parse(input: &mut Peekable<impl Iterator<Item=char>>) -> Result<Self, Self::Error>;
}

pub trait Process {
    type Output;

    fn process(self) -> Self::Output;
}

