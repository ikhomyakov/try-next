//! Minimal traits for synchronous, fallible, pull-based item sources.
//!
//! This module defines two related traits:
//!
//! - [`TryNext`] — a context-free, fallible producer of items,
//! - [`TryNextWithContext`] — a context-aware variant that allows the caller
//!   to supply mutable external state on each iteration step.
//!
//! Both traits follow the same basic pattern: they represent a source that can
//! **attempt to produce the next item**, which may succeed, fail, or signal the
//! end of the sequence.
//!
//! ## Core idea
//!
//! Each `try_next*` method call returns a [`Result`] with three possible outcomes:
//!
//! * `Ok(Some(item))` — a successfully produced item,
//! * `Ok(None)` — no more items are available (the source is exhausted),
//! * `Err(error)` — an error occurred while trying to produce the next item.
//!
//! These traits are **synchronous** — each call blocks until a result is ready.
//! They are suitable for ordinary blocking or CPU-bound producers such as parsers,
//! generators, or readers. For asynchronous, non-blocking sources, use
//! [`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html).
//!
//! ## [`TryNext`]
//!
//! The simplest case: a self-contained, fallible producer that doesn’t depend on
//! any external context.
//!
//! ```rust
//! use try_next::TryNext;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum MyError { Broken }
//!
//! struct Demo { state: u8 }
//!
//! impl TryNext for Demo {
//!     type Item = u8;
//!     type Error = MyError;
//!
//!     fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
//!         match self.state {
//!             0..=2 => {
//!                 let v = self.state;
//!                 self.state += 1;
//!                 Ok(Some(v))
//!             }
//!             3 => {
//!                 self.state += 1;
//!                 Ok(None)
//!             }
//!             _ => Err(MyError::Broken),
//!         }
//!     }
//! }
//!
//! let mut src = Demo { state: 0 };
//! assert_eq!(src.try_next(), Ok(Some(0)));
//! assert_eq!(src.try_next(), Ok(Some(1)));
//! assert_eq!(src.try_next(), Ok(Some(2)));
//! assert_eq!(src.try_next(), Ok(None));
//! assert_eq!(src.try_next(), Err(MyError::Broken));
//! ```
//!
//! ## [`TryNextWithContext`]
//!
//! A generalization of [`TryNext`] that allows each call to [`try_next_with_context`]
//! to receive a mutable reference to a caller-supplied **context**.
//!
//! The context can hold shared mutable state, configuration data, or external
//! resources such as file handles, buffers, or clients. This pattern is useful when
//! the producer needs external help or coordination to produce the next item, while
//! keeping the trait itself simple and generic.
//!
//! ```rust
//! use try_next::TryNextWithContext;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum MyError { Fail }
//!
//! struct Producer;
//!
//! struct Ctx { next_val: u8 }
//!
//! impl TryNextWithContext for Producer {
//!     type Item = u8;
//!     type Error = MyError;
//!     type Context = Ctx;
//!
//!     fn try_next_with_context(
//!         &mut self,
//!         ctx: &mut Self::Context,
//!     ) -> Result<Option<Self::Item>, Self::Error> {
//!         if ctx.next_val < 3 {
//!             let v = ctx.next_val;
//!             ctx.next_val += 1;
//!             Ok(Some(v))
//!         } else {
//!             Ok(None)
//!         }
//!     }
//! }
//!
//! let mut producer = Producer;
//! let mut ctx = Ctx { next_val: 0 };
//!
//! assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(0)));
//! assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(1)));
//! assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(2)));
//! assert_eq!(producer.try_next_with_context(&mut ctx), Ok(None));
//! ```
//!
//! ## Design notes
//!
//! - Both traits are deliberately **minimal**: they define no combinators or adapters.
//!   Their purpose is to provide a simple, low-level interface for fallible, stepwise
//!   data production.
//! - `TryNextWithContext` can often serve as a building block for adapters that
//!   integrate external state or resources.
//! - These traits are a good fit for *incremental* or *stateful* producers such as
//!   **parsers**, **lexers**, **tokenizers**, and other components that advance in
//!   discrete steps while potentially failing mid-stream.
//! - For richer iterator-like abstractions, consider crates like
//!   [`fallible-iterator`](https://crates.io/crates/fallible-iterator) or
//!   [`fallible-streaming-iterator`](https://crates.io/crates/fallible-streaming-iterator).
//!
//! ## See also
//!
//! - [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) —
//!   The standard *synchronous, fallible, pull-based* trait for reading **bytes**.
//!   These traits generalize that idea to arbitrary item types.
//! - [`Iterator<Item = Result<T, E>>`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) —
//!   The idiomatic pattern for representing fallible iteration in the standard library.
//! - [`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html) —
//!   The *asynchronous* equivalent of this pattern.

/// Context-aware, fallible producer.
///
/// A trait for types that can produce items one at a time with the help of
/// an external context, where fetching the next item may fail.
///
/// This trait is **synchronous** — each call to [`try_next`](Self::try_next)
/// blocks until an item is produced or an error occurs.
///
/// The [`Context`](Self::Context) type allows the caller to provide
/// additional state or resources used during iteration. It can hold shared
/// mutable state, configuration data, or external resources such as file
/// handles, buffers, or network clients. Each call to [`try_next`](Self::try_next)
/// receives a mutable reference to this context.
///
/// See the [module-level documentation](self) for details and examples.
pub trait TryNextWithContext {
    /// The type of items yielded by this source.
    type Item;

    /// The error type that may be returned when producing the next item fails.
    type Error;

    /// The type of context passed to each call to [`try_next`](Self::try_next).
    type Context;

    fn try_next_with_context(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<Option<Self::Item>, Self::Error>;
}

/// Context-free, fallible producer.
///
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

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::{TryNext, TryNextWithContext};
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
        let mut c = Counter {
            current: 0,
            limit: 3,
        };

        assert_eq!(c.try_next().unwrap(), Some(0));
        assert_eq!(c.try_next().unwrap(), Some(1));
        assert_eq!(c.try_next().unwrap(), Some(2));
        assert_eq!(c.try_next().unwrap(), None);

        // Stay exhausted
        assert_eq!(c.try_next().unwrap(), None);
    }

    #[test]
    fn drain_collects_all_items() {
        let c = Counter {
            current: 0,
            limit: 5,
        };
        let items = drain(c).unwrap();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn error_propagates() {
        let mut s = FailableCounter {
            current: 0,
            fail_at: 2,
            failed: false,
        };

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
        let mut src: Box<dyn TryNext<Item = usize, Error = Infallible>> = Box::new(Counter {
            current: 0,
            limit: 2,
        });

        assert_eq!(src.try_next().unwrap(), Some(0));
        assert_eq!(src.try_next().unwrap(), Some(1));
        assert_eq!(src.try_next().unwrap(), None);
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Ctx {
        calls: usize,
    }

    impl TryNextWithContext for Counter {
        type Item = usize;
        type Error = Infallible;
        type Context = Ctx;

        fn try_next_with_context(
            &mut self,
            ctx: &mut Self::Context,
        ) -> Result<Option<Self::Item>, Self::Error> {
            ctx.calls += 1;
            if self.current < self.limit {
                let v = self.current;
                self.current += 1;
                Ok(Some(v))
            } else {
                Ok(None)
            }
        }
    }

    /// Drain helper for context-aware sources; returns both the items and the
    /// final context so the caller can assert on context changes.
    fn drain_with_ctx<S: TryNextWithContext>(
        mut src: S,
        mut ctx: S::Context,
    ) -> Result<(Vec<S::Item>, S::Context), S::Error> {
        let mut out = Vec::new();
        while let Some(item) = src.try_next_with_context(&mut ctx)? {
            out.push(item);
        }
        Ok((out, ctx))
    }

    #[test]
    fn context_counter_yields_and_updates_context() {
        let src = Counter {
            current: 0,
            limit: 3,
        };
        let (items, ctx) = drain_with_ctx(src, Ctx::default()).unwrap();

        // Produced the expected sequence 0, 1, 2.
        assert_eq!(items, vec![0, 1, 2]);

        // Called once per yielded item plus one final call returning None.
        assert_eq!(ctx.calls, 4);
    }

    #[test]
    fn context_works_through_trait_object() {
        let mut src: Box<dyn TryNextWithContext<Item = usize, Error = Infallible, Context = Ctx>> =
            Box::new(Counter {
                current: 0,
                limit: 2,
            });

        let mut ctx = Ctx::default();

        assert_eq!(src.try_next_with_context(&mut ctx).unwrap(), Some(0));
        assert_eq!(src.try_next_with_context(&mut ctx).unwrap(), Some(1));
        assert_eq!(src.try_next_with_context(&mut ctx).unwrap(), None);

        // Two items + final None
        assert_eq!(ctx.calls, 3);
    }
}
