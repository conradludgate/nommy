use std::{collections::VecDeque, marker::PhantomData};

/// `Buffer` is an extension to an [`Iterator`],
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
    /// Base type for cursor
    type CursorBase: Buffer<T>;
    /// Create a new cursor from this buffer
    /// any reads the cursor makes will not
    /// affect the next values the buffer will read
    fn cursor(&mut self) -> Cursor<T, Self::CursorBase>;

    /// Skip the iterator ahead by n steps
    fn fast_forward(&mut self, n: usize);

    /// Peek ahead by i spaces
    fn peek_ahead(&mut self, i: usize) -> Option<T>;
}

/// `IntoBuf` is the equivalent of [`IntoIterator`] for a basic implementation of [`Buffer`]
pub trait IntoBuf {
    /// The Iterator type that the Buf type will read from
    type Iter: Iterator;

    /// Convert the iterator into a [`Buf`]
    fn into_buf(self) -> Buf<Self::Iter>;
}

impl<I: IntoIterator> IntoBuf for I {
    type Iter = <Self as IntoIterator>::IntoIter;
    fn into_buf(self) -> Buf<Self::Iter> {
        Buf::new(self)
    }
}

/// Buf is the standard implementation of [`Buffer`]. It stores any peeked data into a [`VecDeque`].
/// Any values peeked will be stored into the [`VecDeque`], and next will either call [`VecDeque::pop_front`]
/// or [`Iterator::next`] on the inner iter
pub struct Buf<I: Iterator> {
    iter: I,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> Iterator for Buf<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer
            .pop_front()
            .map_or_else(|| self.iter.next(), Some)
    }
}

impl<I: Iterator> Buf<I> {
    /// Create a new Buf from the given [`IntoIterator`]. Also see [`IntoBuf`]
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Self {
            iter: iter.into_iter(),
            buffer: VecDeque::new(),
        }
    }
}

impl<I: Iterator> Buffer<I::Item> for Buf<I>
where
    I::Item: Clone,
{
    type CursorBase = Self;
    fn cursor(&mut self) -> Cursor<I::Item, Self::CursorBase> {
        Cursor::new(self)
    }

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

/// `Cursor` is a [`Buffer`] that non-destructively reads from it's parent's buffer using [`Buffer::peek_ahead`]
/// See [`Buffer`] documentation for example usage
pub struct Cursor<'a, T, B: Buffer<T>> {
    buf: &'a mut B,
    base: usize,
    index: usize,
    _t: PhantomData<T>,
}

impl<'a, T, B: Buffer<T>> Cursor<'a, T, B> {
    fn new(buf: &'a mut B) -> Self {
        Self {
            buf,
            base: 0,
            index: 0,
            _t: PhantomData,
        }
    }

    // /// Drops this cursor and calls [`Buffer::fast_forward`] on the parent buffer
    // ///
    // /// ```
    // /// use nommy::{Buffer, IntoBuf};
    // /// let mut input = "foobar".chars().into_buf();
    // /// let mut cursor = input.cursor();
    // /// assert_eq!(cursor.next(), Some('f'));
    // /// assert_eq!(cursor.next(), Some('o'));
    // /// assert_eq!(cursor.next(), Some('o'));
    // ///
    // /// // Typically, the next three calls to `next` would repeat
    // /// // the first three calls because cursors read non-destructively.
    // /// // However, this method allows to drop the already-read contents
    // /// cursor.fast_forward_parent();
    // /// assert_eq!(input.next(), Some('b'));
    // /// assert_eq!(input.next(), Some('a'));
    // /// assert_eq!(input.next(), Some('r'));
    // /// ```
    // pub fn fast_forward_parent(self) {
    //     if cfg!(debug_assertions) && self.index == 0 {
    //         panic!("attempting to fast forward parent, but cursor has not be read from");
    //     }
    //     self.buf.fast_forward(self.index)
    // }

    /// Returns how far along the cursor has read beyond it's parent
    ///
    /// ```
    /// use nommy::{Buffer, IntoBuf};
    /// let mut input = "foobar".chars().into_buf();
    /// let mut cursor = input.cursor();
    /// assert_eq!(cursor.next(), Some('f'));
    /// assert_eq!(cursor.position(), 1);
    ///
    /// let mut cursor2 = cursor.cursor();
    /// assert_eq!(cursor2.next(), Some('o'));
    /// assert_eq!(cursor2.next(), Some('o'));
    /// assert_eq!(cursor2.position(), 2);
    ///
    /// cursor2.fast_forward_parent();
    /// assert_eq!(cursor.position(), 3);
    /// ```
    pub fn position(&self) -> usize {
        self.index
    }

    /// Resets the cursor back to where it started
    pub fn reset(&mut self) -> bool {
        self.index = 0;
        true
    }
}

impl<'a, T, B: Buffer<T>> Buffer<T> for Cursor<'a, T, B> {
    fn fast_forward(&mut self, n: usize) {
        self.index += n;
    }

    fn peek_ahead(&mut self, i: usize) -> Option<T> {
        self.buf.peek_ahead(self.base + self.index + i)
    }

    type CursorBase = B;
    fn cursor(&mut self) -> Cursor<T, Self::CursorBase> {
        Cursor {
            buf: self.buf,
            base: self.base + self.index,
            index: 0,
            _t: PhantomData,
        }
    }
}

impl<'a, T, B: Buffer<T>> Iterator for Cursor<'a, T, B> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let output = self.buf.peek_ahead(self.base + self.index);
        self.index += 1;
        output
    }
}

// use std::io::Read;

// /// Implements [`Buffer`] for types that implement [`Read`]
// pub struct BufRead<R: Read> {
//     read: R,
//     buf: Vec<u8>,
//     head: usize,
//     len: usize,
// }

// impl<R: Read> BufRead<R> {
//     const DEFAULT_CAPACITY: usize = 4 * 1024;
//     pub fn new(read: R) -> Self {
//         Self::with_capacity(read, Self::DEFAULT_CAPACITY)
//     }

//     pub fn with_capacity(read: R, capacity: usize) -> Self {
//         let mut buf = Vec::with_capacity(capacity);

//         // Safety: buf makes use of `head` and `len` to determine whether the
//         // value is initialised.
//         unsafe {
//             buf.set_len(capacity);
//         }
//         Self {
//             read,
//             buf,
//             head: 0,
//             len: 0,
//         }
//     }

//     pub fn set_capacity(&mut self, capacity: usize) {
//         let capacity = capacity.max(self.len);

//         let head = self.head;
//         let len = self.len;
//         let cc = self.buf.len();

//         if head + len > cc {
//             // Data is currently fragmented
//             // solve by creating a new vec with the desired capacity to defrag

//             let mut new_buf = Vec::with_capacity(capacity);

//             // Safety: buf makes use of `head` and `len` to determine whether the
//             // value is initialised.
//             unsafe {
//                 new_buf.set_len(capacity);
//             }

//             new_buf[..(cc - head)].copy_from_slice(&self.buf[head..]);
//             new_buf[(cc - head)..(cc - head + len)].copy_from_slice(&self.buf[..len]);

//             self.buf = new_buf;
//         } else {
//             self.buf.reserve(capacity - cc);
//         }
//     }

//     pub fn fill_buffer(&mut self) -> std::io::Result<()> {
//         if self.head + self.len < self.buf.len() {
//             self.len += self.read.read(&mut self.buf[self.head + self.len..])?;

//             if self.len < self.buf.len() {
//                 // didn't completely fill up
//                 return Ok(());
//             }
//         }

//         let tail = (self.head + self.len) % self.buf.len();
//         self.len += self.read.read(&mut self.buf[tail..self.head])?;
//         Ok(())
//     }
// }

// impl<R: Read> Iterator for BufRead<R> {
//     type Item = u8;
//     fn next(&mut self) -> Option<Self::Item> {
//         println!("buf: {:?} head: {} len: {}", self.buf, self.head, self.len);
//         let capacity = self.buf.len();

//         if self.len == 0 {
//             self.fill_buffer().map_or(None, Some)?;

//             if self.len == 0 {
//                 // attempted to fill but length still 0, implies that the reader is most likely now empty
//                 return None;
//             }
//         }

//         let byte = self.buf[self.head];
//         self.head += 1;
//         self.head %= capacity;
//         self.len -= 1;
//         Some(byte)
//     }
// }

// impl<R: Read> Buffer<u8> for BufRead<R> {
//     fn fast_forward(&mut self, n: usize) {
//         if n > self.len {
//         } else {
//             self.head += n;
//             self.head %= self.buf.len();
//             self.len -= n;
//         }
//     }

//     fn peek_ahead(&mut self, i: usize) -> Option<u8> {
//         if i > self.buf.len() {
//             self.set_capacity(i.next_power_of_two());
//         }
//         if i > self.len {}

//         let byte = self.buf[self.head];
//         self.head += 1;
//         self.head %= self.buf.len();
//         Some(byte)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{IntoBuf};

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

    // #[test]
    // fn bufread() {
    //     let read: &[u8] = b"Hello World!";
    //     let buffer = BufRead::with_capacity(read, 5);

    //     let output: Vec<u8> = buffer.collect();
    //     assert_eq!(&output, b"Hello World!");
    // }
}
