//! Minimal traits for synchronous, fallible, pull-based item sources.
//!
//! This module defines three related traits:
//!
//! - [`TryNext<S = ()>`] — a context-free, fallible producer of items,
//! - [`TryNextWithContext<C, S = ()>`] — a context-aware variant that allows the caller
//!   to supply mutable external state on each iteration step.
//! - [`IterInput<I>`] — an input adapter that wraps any iterator
//!   and provides `TryNext` and `TryNextWithContext<C>` interface, automatically
//!   fusing the iterator
//!
//! Traits [`TryNext<S = ()>`] and [`TryNextWithContext<C, S = ()>`] follow the same basic pattern:
//! they represent a source that can **attempt to produce the next item**, which may
//! succeed, fail, or signal the end of the sequence.
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
//! ### Optional stats type `S`
//!
//! Both [`TryNext`] and [`TryNextWithContext<C>`] accept an **optional generic**
//! `S` that represents a *lightweight stats snapshot* for an implementation.
//! By default, `S = ()` and [`stats`](TryNext::stats)/[`stats`](TryNextWithContext::stats)
//! simply return `()`. Implementors may choose a custom `S` to expose metrics
//! (counters, flags, etc.) and override `stats()` to return them. The only
//! requirement is that `S: Default + Copy`.
//!
//! ```rust
//! use try_next::TryNext;
//!
//! #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
//! struct MyStats { calls: u32 }
//!
//! struct Demo { calls: u32, left: u32 }
//!
//! impl TryNext<MyStats> for Demo {
//!     type Item = u32;
//!     type Error = core::convert::Infallible;
//!
//!     fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
//!         self.calls += 1;
//!         if self.left == 0 { return Ok(None); }
//!         let out = self.left - 1;
//!         self.left -= 1;
//!         Ok(Some(out))
//!     }
//!
//!     fn stats(&self) -> MyStats { MyStats { calls: self.calls } }
//! }
//! ```
//!
//! ## [`TryNext<S = ()>`]
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
//! ## [`TryNextWithContext<C, S = ()>`]
//!
//! A generalization of [`TryNext<S = ()>`] that allows each call to [`try_next_with_context`]
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
//! impl TryNextWithContext<Ctx> for Producer {
//!     type Item = u8;
//!     type Error = MyError;
//!
//!     fn try_next_with_context(
//!         &mut self,
//!         ctx: &mut Ctx,
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
//! ## [`IterInput<I>`]
//!
//! A simple [`TryNextWithContext<C, S = ()>`] adapter for ordinary Rust iterators.
//!
//! `IterInput` wraps any [`Iterator`] and exposes it as a
//! [`TryNextWithContext<C, S = ()>`] producer that never fails and ignores the provided context.
//! Internally, the iterator is automatically *fused* — once it yields `None`,
//! all subsequent calls also return `None`.
//!
//! This is useful for integrating plain iterators into APIs or components that
//! expect a context-aware, fallible producer, without changing their semantics.
//!
//! ### Example
//!
//! ```rust
//! use try_next::TryNextWithContext;
//! use try_next::IterInput;
//!
//! struct DummyCtx;
//!
//! let data = [10, 20, 30];
//! let mut input = IterInput::from(data.into_iter());
//! let mut ctx = DummyCtx;
//!
//! assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(10));
//! assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(20));
//! assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(30));
//! assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
//! assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None); // fused
//! ```
//!
//! ### Notes
//!
//! - The `C` type parameter exists for trait compatibility but is not used.
//! - The error type is always [`Infallible`], as the wrapped iterator cannot fail.
//! - Ideal for testing or bridging APIs that use [`TryNextWithContext<C>`] but only
//!   need to pull from a fixed iterator.
//! - The optional stats type parameter `S` on the traits is independent of the
//!   iterator and commonly left as the default `()`.
//!
//! ## Design notes
//!
//! - All traits are deliberately **minimal**: they define no combinators or adapters.
//!   Their purpose is to provide a simple, low-level interface for fallible, stepwise
//!   data production.
//! - `TryNextWithContext<C, S = ()>` can often serve as a building block for adapters that
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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Context-aware, fallible producer.
///
/// A trait for types that can produce items one at a time with the help of
/// an external context, where fetching the next item may fail.
///
/// This trait is **synchronous** — each call to [`try_next`](Self::try_next)
/// blocks until an item is produced or an error occurs.
///
/// The context type `C` allows the caller to provide additional
/// state or resources used during iteration. It can hold shared
/// mutable state, configuration data, or external resources such as file
/// handles, buffers, or network clients. Each call to [`try_next`](Self::try_next)
/// receives a mutable reference to this context.
///
/// # Type Parameters
///
/// - `C` - The type of the external context passed to each call of
///   [`try_next_with_context`](Self::try_next_with_context). It represents the
///   environment or state that the producer can use or mutate while producing
///   items. For example, this might be a reader, a buffer pool, or a user-defined
///   structure containing shared resources.
/// - `S` - Optional stats type.
pub trait TryNextWithContext<C, S = ()>
where
    S: Default + Copy,
{
    /// The type of items yielded by this source.
    type Item;

    /// The error type that may be returned when producing the next item fails.
    type Error;

    /// Attempts to produce the next item, using the provided mutable context.
    ///
    /// Returns:
    /// - `Ok(Some(item))` — a new item was produced,
    /// - `Ok(None)` — the source is exhausted,
    /// - `Err(e)` — iteration failed with an error.
    fn try_next_with_context(&mut self, context: &mut C)
    -> Result<Option<Self::Item>, Self::Error>;

    /// Collects all remaining items into a [`Vec`], using the given context.
    ///
    /// The method repeatedly calls [`try_next_with_context`](Self::try_next_with_context)
    /// until `None` or an error is returned, collecting all successful items into a vector.
    /// If any call returns `Err(e)`, iteration stops immediately and that error is returned.
    ///
    /// # Feature
    /// This method is only available when the `alloc` feature is enabled.
    #[cfg(feature = "alloc")]
    #[inline]
    fn try_collect_with_context(
        &mut self,
        context: &mut C,
    ) -> Result<Vec<Self::Item>, Self::Error> {
        let mut vs = Vec::new();
        while let Some(v) = self.try_next_with_context(context)? {
            vs.push(v);
        }
        Ok(vs)
    }

    fn stats(&self) -> S {
        S::default()
    }
}

/// Context-free, fallible producer.
///
/// A trait for types that can produce items one at a time, where fetching
/// the next item may fail.
///
/// This trait is **synchronous** — each call to [`try_next`](Self::try_next)
/// blocks until an item is produced or an error occurs. See the
/// [module-level documentation](self) for details and examples.
///
/// - `S` - Optional stats type.
pub trait TryNext<S = ()>
where
    S: Default + Copy,
{
    /// The type of items yielded by this source.
    type Item;

    /// The error type that may be returned when producing the next item fails.
    type Error;

    /// Attempts to produce the next item from the source.
    ///
    /// Returns:
    /// - `Ok(Some(item))` — a new item was produced,
    /// - `Ok(None)` — the source is exhausted,
    /// - `Err(e)` — iteration failed with an error.
    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error>;

    /// Collects all remaining items into a [`Vec`].
    ///
    /// The method repeatedly calls [`try_next`](Self::try_next) until `None` or an error
    /// is returned, collecting all successful items into a vector.  
    /// If any call returns `Err(e)`, iteration stops immediately and that error is returned.
    ///
    /// # Feature
    /// This method is only available when the `alloc` feature is enabled.
    #[cfg(feature = "alloc")]
    #[inline]
    fn try_collect(&mut self) -> Result<Vec<Self::Item>, Self::Error> {
        let mut vs = Vec::new();
        while let Some(v) = self.try_next()? {
            vs.push(v);
        }
        Ok(vs)
    }

    fn stats(&self) -> S {
        S::default()
    }
}

/// An input adapter that wraps any iterator and provides `TryNext` and
/// `TryNextWithContext<C>` interface, automatically fusing the iterator
/// so it never yields items after returning `None` once.
///
/// # Type Parameters
///
/// - `I`: The underlying iterator type. It can be any `Iterator`.
/// - `C`: The *context* type, which is passed by mutable reference to each
///   `try_next_with_context` call.
pub struct IterInput<I>
where
    I: Iterator,
{
    /// The underlying fused iterator.
    iter: core::iter::Fuse<I>,
}

impl<I> IterInput<I>
where
    I: Iterator,
{
    /// Creates a new `IterInput` from any iterator.
    ///
    /// The iterator is automatically fused internally, so that once it returns
    /// `None`, all further `next()` calls will also return `None`.
    pub fn from(iter: I) -> Self {
        Self { iter: iter.fuse() }
    }
}

impl<I> TryNext for IterInput<I>
where
    I: Iterator,
{
    type Item = I::Item;
    type Error = core::convert::Infallible;

    #[inline]
    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.iter.next())
    }
}

impl<I, C> TryNextWithContext<C> for IterInput<I>
where
    I: Iterator,
{
    type Item = I::Item;
    type Error = core::convert::Infallible;

    #[inline]
    fn try_next_with_context(
        &mut self,
        _context: &mut C,
    ) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.iter.next())
    }
}

#[cfg(feature = "std")]
pub mod io;

#[cfg(test)]
mod tests {
    use super::{IterInput, TryNext, TryNextWithContext};
    use core::convert::Infallible;

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

    #[cfg(feature = "alloc")]
    fn drain<S: TryNext>(mut src: S) -> Result<Vec<S::Item>, S::Error> {
        src.try_collect()
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
    #[cfg(feature = "alloc")]
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

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Ctx {
        calls: usize,
    }

    impl TryNextWithContext<Ctx> for Counter {
        type Item = usize;
        type Error = Infallible;

        fn try_next_with_context(
            &mut self,
            ctx: &mut Ctx,
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
    #[cfg(feature = "alloc")]
    fn drain_with_ctx<C, S: TryNextWithContext<C>>(
        mut src: S,
        mut ctx: C,
    ) -> Result<(Vec<S::Item>, C), S::Error> {
        let mut out = Vec::new();
        while let Some(item) = src.try_next_with_context(&mut ctx)? {
            out.push(item);
        }
        Ok((out, ctx))
    }

    #[test]
    #[cfg(feature = "alloc")]
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

    #[derive(Default)]
    struct DummyCtx;

    #[test]
    fn iter_input_yields_all_items_from_array() {
        let mut input = IterInput::from([1, 2, 3].into_iter());
        let mut ctx = DummyCtx;

        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(1));
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(2));
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(3));
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
    }

    #[test]
    fn iter_input_is_fused_after_exhaustion() {
        let mut input = IterInput::from(0..1);
        let mut ctx = DummyCtx;

        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), Some(0));
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
    }

    #[test]
    fn iter_input_with_empty_range_returns_none() {
        let mut input = IterInput::from(0..0);
        let mut ctx = DummyCtx;

        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
    }

    #[test]
    fn iter_input_over_string_bytes() {
        let mut input = IterInput::from("hello, world!".bytes());
        let mut ctx = DummyCtx;

        // Collect bytes manually via try_next_with_context
        let mut collected = [0u8; 13];
        let mut count = 0;

        while let Some(byte) = input.try_next_with_context(&mut ctx).unwrap() {
            collected[count] = byte;
            count += 1;
        }

        assert_eq!(&collected[..count], b"hello, world!");
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
        assert_eq!(input.try_next_with_context(&mut ctx).unwrap(), None);
    }

    #[test]
    fn stats_default_is_unit() {
        // With the default S=() the blanket stats() should return ().
        let c = Counter {
            current: 0,
            limit: 1,
        };
        let unit_stats: () = TryNext::stats(&c);
        assert_eq!(unit_stats, ());
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct MyStats {
        calls: usize,
    }

    /// A tiny source that tracks the number of `try_next` calls in `stats()`.
    struct StatsSource {
        remaining: usize,
        emitted: usize,
        call_count: usize,
    }

    impl TryNext<MyStats> for StatsSource {
        type Item = usize;
        type Error = Infallible;

        fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
            self.call_count += 1;
            if self.remaining == 0 {
                return Ok(None);
            }
            let v = self.emitted;
            self.emitted += 1;
            self.remaining -= 1;
            Ok(Some(v))
        }

        fn stats(&self) -> MyStats {
            MyStats {
                calls: self.call_count,
            }
        }
    }

    #[test]
    fn custom_stats_are_reported() {
        let mut s = StatsSource {
            remaining: 3,
            emitted: 0,
            call_count: 0,
        };
        assert_eq!(s.stats(), MyStats { calls: 0 });

        // 3 items followed by None => 4 total calls
        assert_eq!(s.try_next().unwrap(), Some(0));
        assert_eq!(s.try_next().unwrap(), Some(1));
        assert_eq!(s.try_next().unwrap(), Some(2));
        assert_eq!(s.try_next().unwrap(), None);

        assert_eq!(s.stats(), MyStats { calls: 4 });
    }

    #[test]
    fn iter_input_try_next_works_without_context() {
        // Ensure the context-free TryNext impl behaves as expected.
        let mut input = IterInput::from([10, 20, 30].into_iter());
        assert_eq!(TryNext::try_next(&mut input).unwrap(), Some(10));
        assert_eq!(TryNext::try_next(&mut input).unwrap(), Some(20));
        assert_eq!(TryNext::try_next(&mut input).unwrap(), Some(30));
        assert_eq!(TryNext::try_next(&mut input).unwrap(), None);
        assert_eq!(TryNext::try_next(&mut input).unwrap(), None); // fused
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn iter_input_try_collect_collects_all() {
        let mut input = IterInput::from([7, 8, 9].into_iter());
        let items = input.try_collect().unwrap();
        assert_eq!(items, vec![7, 8, 9]);
        // And it stays exhausted.
        assert_eq!(TryNext::try_next(&mut input).unwrap(), None);
    }
}
