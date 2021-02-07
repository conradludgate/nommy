use std::{collections::VecDeque, marker::PhantomData};

/// Buffer is an extension to an [Iterator],
/// with the ability to create a cursor over the iterator,
/// which can infinitely read from the iterator, preserving the buffer's position
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
pub trait Buffer<T>: Iterator<Item = T> + Sized {
    /// Create a new cursor from this buffer
    /// any reads the cursor makes will not
    /// affect the next values the buffer will read
    fn cursor(&mut self) -> Cursor<T, Self> {
        Cursor::new(self)
    }

    /// Skip the iterator ahead by n steps
    fn fast_forward(&mut self, n: usize);

    /// Peek ahead by i spaces
    fn peek_ahead(&mut self, i: usize) -> Option<T>;
}

/// IntoBuf is the equivalent of [IntoIterator] for a basic implementation of [Buffer]
pub trait IntoBuf {
    /// The iterator that the Buf type will read from
    type Iter: Iterator;

    /// Convert the iterator into a [Buf]
    fn into_buf(self) -> Buf<Self::Iter>;
}

impl<I: IntoIterator> IntoBuf for I {
    type Iter = I::IntoIter;
    fn into_buf(self) -> Buf<Self::Iter> {
        Buf::new(self)
    }
}

/// Buf is the standard implementation of [Buffer]. It stores any peeked data into a [VecDeque].
/// Any values peeked will be stored into the [VecDeque], and next will either call [VecDeque::pop_front]
/// or [Iterator::next] on the inner iter
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
    /// Create a new Buf from the given [IntoIterator]
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Buf {
            iter: iter.into_iter(),
            buffer: VecDeque::new(),
        }
    }
}

impl<I: Iterator> Buffer<I::Item> for Buf<I>
where
    I::Item: Clone,
{
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

    fn peek_ahead(&mut self, i: usize) -> Option<I::Item> {
        if i < self.buffer.len() {
            Some(self.buffer[i].clone())
        } else {
            let diff = i - self.buffer.len();
            for _ in 0..diff {
                let cache = self.iter.next()?;
                self.buffer.push_back(cache);
            }
            let output = self.iter.next()?;
            self.buffer.push_back(output.clone());
            Some(output)
        }
    }
}

/// Cursors are a [Buffer] that non-destructively read from their parent buffers using [Buffer::peek_ahead]
/// See [Buffer] documentation for example usage
pub struct Cursor<'a, T, B: Buffer<T>> {
    buf: &'a mut B,
    index: usize,
    _t: PhantomData<T>,
}

impl<'a, T, B: Buffer<T>> Cursor<'a, T, B> {
    /// Create a new cursor over the buffer
    pub fn new(buf: &'a mut B) -> Self {
        Cursor {
            buf,
            index: 0,
            _t: PhantomData,
        }
    }

    /// Drops this cursor and calls [Buffer::fast_forward] on the parent buffer
    pub fn fast_forward_parent(self) {
        self.buf.fast_forward(self.index)
    }
}

impl<'a, T, B: Buffer<T>> Buffer<T> for Cursor<'a, T, B> {
    fn fast_forward(&mut self, n: usize) {
        self.index += n;
    }

    fn peek_ahead(&mut self, i: usize) -> Option<T> {
        self.buf.peek_ahead(self.index + i)
    }
}

impl<'a, T, B: Buffer<T>> Iterator for Cursor<'a, T, B> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let output = self.buf.peek_ahead(self.index);
        self.index += 1;
        output
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
