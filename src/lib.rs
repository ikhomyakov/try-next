//! A minimal trait for **synchronous**, fallible, pull-based item sources.
//!
//! The [`TryNext`] trait defines a simple interface for producing items
//! one at a time, where advancing to the next item may fail.
//!
//! Each call to [`TryNext::try_next`] attempts to yield the next value and
//! returns a [`Result`] distinguishing between three cases:
//!
//! * `Ok(Some(item))` — a successfully produced item,
//! * `Ok(None)` — there are no more items available (the source is exhausted),
//! * `Err(error)` — an error occurred while trying to produce the next item.
//!
//! This trait is intentionally **synchronous** and does not use `async` or
//! `Poll`. It is designed for ordinary blocking or CPU-bound sources such as
//! parsers, readers, or generators that advance in discrete steps. For
//! asynchronous, non-blocking use cases, see
//! [`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html).
//!
//! Unlike [`Iterator`] or [`Stream`], `TryNext` is a minimal abstraction: it
//! defines no combinators or adapters, and is best suited for simple cases where
//! you need to fetch or parse items fallibly in a loop.
//!
//! ## Example
//!
//! ```
//! use try_next::TryNext;
//!
//! struct Counter {
//!     current: usize,
//!     limit: usize,
//! }
//!
//! impl TryNext for Counter {
//!     type Item = usize;
//!     type Error = std::convert::Infallible;
//!
//!     fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
//!         if self.current < self.limit {
//!             let v = self.current;
//!             self.current += 1;
//!             Ok(Some(v))
//!         } else {
//!             Ok(None)
//!         }
//!     }
//! }
//!
//! let mut c = Counter { current: 0, limit: 3 };
//! assert_eq!(c.try_next().unwrap(), Some(0));
//! assert_eq!(c.try_next().unwrap(), Some(1));
//! assert_eq!(c.try_next().unwrap(), Some(2));
//! assert_eq!(c.try_next().unwrap(), None);
//! ```
//!
//! ## See also
//!
//! * [`Iterator`] — the infallible standard trait for synchronous iteration.
//! * [`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html) — asynchronous equivalent.
//!
//! The [`TryNext`] trait serves as a lightweight foundation for synchronous,
//! fallible data sources that don’t require the full machinery of iterators
//! or streams.

/// A trait for types that can produce items one at a time, where fetching
/// the next item may fail.
///
/// This trait is **synchronous** — each call to [`try_next`](Self::try_next)
/// blocks until an item is produced or an error occurs. See the
/// [module-level documentation](self) for details and examples.
pub trait TryNext {
    /// The type of items yielded by this source.
    type Item;

    /// The error type that may be returned when producing the next item fails.
    type Error;

    /// Attempts to produce the next item in the sequence.
    ///
    /// Returns:
    /// * `Ok(Some(item))` if a new item was successfully produced,
    /// * `Ok(None)` if the source is exhausted,
    /// * `Err(error)` if an error occurred.
    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::TryNext;
    use std::convert::Infallible;

    /// A simple source that yields 0..limit, then `Ok(None)`.
    struct Counter {
        current: usize,
        limit: usize,
    }

    impl TryNext for Counter {
        type Item = usize;
        type Error = Infallible;

        fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
            if self.current < self.limit {
                let v = self.current;
                self.current += 1;
                Ok(Some(v))
            } else {
                Ok(None)
            }
        }
    }

    /// A source that yields 0..fail_at, then returns `Err(())`.
    struct FailableCounter {
        current: usize,
        fail_at: usize,
        failed: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct UnitErr;

    impl TryNext for FailableCounter {
        type Item = usize;
        type Error = UnitErr;

        fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
            if self.failed {
                // Once failed, keep failing to make behavior explicit for tests
                return Err(UnitErr);
            }
            if self.current == self.fail_at {
                self.failed = true;
                return Err(UnitErr);
            }
            let v = self.current;
            self.current += 1;
            Ok(Some(v))
        }
    }

    fn drain<S: TryNext>(mut src: S) -> Result<Vec<S::Item>, S::Error> {
        let mut out = Vec::new();
        while let Some(item) = src.try_next()? {
            out.push(item);
        }
        Ok(out)
    }

    #[test]
    fn counter_yields_then_none() {
        let mut c = Counter { current: 0, limit: 3 };

        assert_eq!(c.try_next().unwrap(), Some(0));
        assert_eq!(c.try_next().unwrap(), Some(1));
        assert_eq!(c.try_next().unwrap(), Some(2));
        assert_eq!(c.try_next().unwrap(), None);

        // Stay exhausted
        assert_eq!(c.try_next().unwrap(), None);
    }

    #[test]
    fn drain_collects_all_items() {
        let c = Counter { current: 0, limit: 5 };
        let items = drain(c).unwrap();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn error_propagates() {
        let mut s = FailableCounter { current: 0, fail_at: 2, failed: false };

        // First two items OK
        assert_eq!(s.try_next(), Ok(Some(0)));
        assert_eq!(s.try_next(), Ok(Some(1)));

        // Then an error
        assert_eq!(s.try_next(), Err(UnitErr));

        // Subsequent calls keep erroring in this test source
        assert_eq!(s.try_next(), Err(UnitErr));
    }

    #[test]
    fn works_through_trait_object() {
        let mut src: Box<dyn TryNext<Item = usize, Error = Infallible>> =
            Box::new(Counter { current: 0, limit: 2 });

        assert_eq!(src.try_next().unwrap(), Some(0));
        assert_eq!(src.try_next().unwrap(), Some(1));
        assert_eq!(src.try_next().unwrap(), None);
    }
}

