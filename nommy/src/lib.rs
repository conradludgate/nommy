pub mod impls;
pub mod token;
pub mod tuple;

use std::{collections::VecDeque, error::Error};

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
    pub fn new(iter: I) -> Self {
        Buffer {
            iter,
            buffer: VecDeque::new(),
        }
    }

    pub fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self,
            index: 0,
        }
    }
}

pub struct Cursor<'a, I: Iterator>{
    buf: &'a mut Buffer<I>,
    index: usize,
}

impl<'a, I: Iterator> Cursor<'a, I> {
    pub fn cursor(&mut self) -> Cursor<I> {
        Cursor {
            buf: self.buf,
            index: self.index,
        }
    }
}

impl<'a, I: Iterator> Iterator for Cursor<'a, I> where I::Item: Clone {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let output = if self.index >= self.buf.buffer.len() {
            let output = self.buf.iter.next()?;
            self.buf.buffer.push_back(output.clone());
            output
        } else {
            self.buf.buffer[self.index].clone()
        };

        self.index += 1;
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn sequence_at_least_one_but_none() {
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
}
