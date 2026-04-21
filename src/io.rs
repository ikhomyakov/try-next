//! I/O utility implementations for `TryNext` and `TryNextWithContext`.
//!
//! This module provides implementations of the [`TryNext`] and
//! [`TryNextWithContext`] traits for [`BufReader`], enabling incremental,
//! fallible reading of bytes from any [`Read`] stream.
//!
//! These traits abstract over iterating through potentially fallible
//! sources that yield elements sequentially. By
//! implementing them for `BufReader`, this module allows buffered reading
//! of single bytes in a way that works well for streaming parsers or
//! byte-oriented protocols.
//!
//! # Features
//!
//! This module is only available when the **`std`** feature is enabled.
//!
//! # Example
//!
//! ```rust
//! use std::io::BufReader;
//! use try_next::{TryNext, TryNextWithContext};
//!
//! let data = b"abc";
//! let mut reader = BufReader::new(&data[..]);
//!
//! assert_eq!(reader.try_next().unwrap(), Some(b'a'));
//! assert_eq!(reader.try_next().unwrap(), Some(b'b'));
//! assert_eq!(reader.try_next().unwrap(), Some(b'c'));
//! assert_eq!(reader.try_next().unwrap(), None);
//! ```

#![cfg(feature = "std")]

use crate::{TryNext, TryNextWithContext};
use std::io::{self, BufRead, BufReader, Read};

/// Implements [`TryNextWithContext`] for [`BufReader`].
///
/// This implementation enables reading the next byte from a buffered
/// reader while optionally using a mutable external context `C`.
///
/// The context parameter allows stateful processing during iteration,
/// even though this implementation does not directly use it.
///
/// # Type Parameters
///
/// * `R` — The underlying reader type implementing [`Read`].
/// * `C` — The context type passed into [`try_next_with_context`].
///
/// # Errors
///
/// Returns any I/O error encountered during reading. Returns `Ok(None)`
/// when the reader reaches the end of the input.
impl<R, C> TryNextWithContext<C> for BufReader<R>
where
    R: Read,
{
    /// The type of item yielded — a single `u8` byte.
    type Item = u8;

    /// The error type produced by this iterator — [`io::Error`].
    type Error = io::Error;

    /// Attempts to read the next byte from the buffer using an optional context.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(byte))` if a byte was read successfully.
    /// * `Ok(None)` if EOF was reached.
    /// * `Err(e)` if an I/O error occurred.
    fn try_next_with_context(&mut self, _context: &mut C) -> Result<Option<Self::Item>, Self::Error> {
        let buf = self.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }
        let b = buf[0];
        self.consume(1);
        Ok(Some(b))
    }
}

/// Implements [`TryNext`] for [`BufReader`].
///
/// This version does not use a context, and simply reads the next byte
/// sequentially from the buffer.
///
/// # Example
///
/// ```rust
/// use std::io::BufReader;
/// use try_next::TryNext;
///
/// let data = b"hi";
/// let mut reader = BufReader::new(&data[..]);
///
/// assert_eq!(reader.try_next().unwrap(), Some(b'h'));
/// assert_eq!(reader.try_next().unwrap(), Some(b'i'));
/// assert_eq!(reader.try_next().unwrap(), None);
/// ```
impl<R> TryNext for BufReader<R>
where
    R: Read,
{
    /// The type of item yielded — a single `u8` byte.
    type Item = u8;

    /// The error type produced — [`io::Error`].
    type Error = io::Error;

    /// Attempts to read the next byte from the buffer.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(byte))` if a byte was successfully read.
    /// * `Ok(None)` if EOF was reached.
    /// * `Err(e)` if an I/O error occurred.
    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let buf = self.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }
        let b = buf[0];
        self.consume(1);
        Ok(Some(b))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read};

    #[test]
    fn try_next_reads_each_byte_then_none() {
        let data = b"abc";
        let mut rdr = BufReader::new(&data[..]);

        assert_eq!(rdr.try_next().unwrap(), Some(b'a'));
        assert_eq!(rdr.try_next().unwrap(), Some(b'b'));
        assert_eq!(rdr.try_next().unwrap(), Some(b'c'));
        assert_eq!(rdr.try_next().unwrap(), None); // EOF
        assert_eq!(rdr.try_next().unwrap(), None); // Stays at EOF
    }

    #[test]
    fn try_next_with_context_reads_bytes_and_reaches_eof() {
        let data = b"xy";
        let mut rdr = BufReader::new(&data[..]);
        let mut ctx = (); // context is unused; just verify it compiles/works

        assert_eq!(rdr.try_next_with_context(&mut ctx).unwrap(), Some(b'x'));
        assert_eq!(rdr.try_next_with_context(&mut ctx).unwrap(), Some(b'y'));
        assert_eq!(rdr.try_next_with_context(&mut ctx).unwrap(), None);
    }

    #[test]
    fn try_next_on_empty_input_is_none() {
        let data: &[u8] = b"";
        let mut rdr = BufReader::new(&data[..]);

        assert_eq!(rdr.try_next().unwrap(), None);
    }

    #[test]
    fn try_next_with_context_on_empty_input_is_none() {
        let data: &[u8] = b"";
        let mut rdr = BufReader::new(&data[..]);
        let mut ctx = ();

        assert_eq!(rdr.try_next_with_context(&mut ctx).unwrap(), None);
    }

    /// A reader that returns an error on first read to exercise error paths.
    struct Boom;
    impl Read for Boom {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "boom"))
        }
    }

    #[test]
    fn try_next_propagates_errors() {
        let mut rdr = BufReader::new(Boom);
        let err = rdr.try_next().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.to_string(), "boom");
    }

    #[test]
    fn try_next_with_context_propagates_errors() {
        let mut rdr = BufReader::new(Boom);
        let mut ctx = ();
        let err = rdr.try_next_with_context(&mut ctx).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.to_string(), "boom");
    }
}

