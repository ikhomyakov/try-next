# try-next &nbsp; [![Crates.io](https://img.shields.io/crates/v/try-next.svg)](https://crates.io/crates/try-next) [![Documentation](https://docs.rs/parlex/badge.svg)](https://docs.rs/try-next) [![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL%203.0--or--later-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)

Minimal traits for synchronous, fallible, pull-based item sources.


## Overview

This module defines two related traits:

- [`TryNext`] — a context-free, fallible producer of items,
- [`TryNextWithContext`] — a context-aware variant that allows the caller
  to supply mutable external state on each iteration step.

Both traits follow the same basic pattern: they represent a source that can
**attempt to produce the next item**, which may succeed, fail, or signal the
end of the sequence.


## Core idea

Each `try_next*` method call returns a [`Result`] with three possible outcomes:

* `Ok(Some(item))` — a successfully produced item,
* `Ok(None)` — no more items are available (the source is exhausted),
* `Err(error)` — an error occurred while trying to produce the next item.

These traits are **synchronous** — each call blocks until a result is ready.
They are suitable for ordinary blocking or CPU-bound producers such as parsers,
generators, or readers. For asynchronous, non-blocking sources, use
[`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html).


## [`TryNext`]

The simplest case: a self-contained, fallible producer that doesn’t depend on
any external context.

```rust
use try_next::TryNext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyError { Broken }

struct Demo { state: u8 }

impl TryNext for Demo {
    type Item = u8;
    type Error = MyError;

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            0..=2 => {
                let v = self.state;
                self.state += 1;
                Ok(Some(v))
            }
            3 => {
                self.state += 1;
                Ok(None)
            }
            _ => Err(MyError::Broken),
        }
    }
}

let mut src = Demo { state: 0 };
assert_eq!(src.try_next(), Ok(Some(0)));
assert_eq!(src.try_next(), Ok(Some(1)));
assert_eq!(src.try_next(), Ok(Some(2)));
assert_eq!(src.try_next(), Ok(None));
assert_eq!(src.try_next(), Err(MyError::Broken));
```


## [`TryNextWithContext`]

A generalization of [`TryNext`] that allows each call to [`try_next_with_context`]
to receive a mutable reference to a caller-supplied **context**.

The context can hold shared mutable state, configuration data, or external
resources such as file handles, buffers, or clients. This pattern is useful when
the producer needs external help or coordination to produce the next item, while
keeping the trait itself simple and generic.

```rust
use try_next::TryNextWithContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyError { Fail }

struct Producer;

struct Ctx { next_val: u8 }

impl TryNextWithContext for Producer {
    type Item = u8;
    type Error = MyError;
    type Context = Ctx;

    fn try_next_with_context(
        &mut self,
        ctx: &mut Self::Context,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if ctx.next_val < 3 {
            let v = ctx.next_val;
            ctx.next_val += 1;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
}
let mut producer = Producer;
let mut ctx = Ctx { next_val: 0 };

assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(0)));
assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(1)));
assert_eq!(producer.try_next_with_context(&mut ctx), Ok(Some(2)));
assert_eq!(producer.try_next_with_context(&mut ctx), Ok(None));
```


## Why not just `Iterator<Item = Result<T, E>>`?

You *can* use an `Iterator` of `Result`s — and for many cases you should.
`TryNext` exists for scenarios where:

* You don’t need or want the entire iterator API surface,
* You want a **blocking**, stepwise producer that can fail (e.g., parser, file reader),
* You’d like an API closer to I/O traits like `Read` or `BufRead` with fallible semantics.

It’s deliberately small and easy to wrap or adapt into an iterator when needed.


## Features

* 🦀 Zero dependencies
* ⚙️  Simple and explicit `Result<Option<T>, E>` semantics
* 🧩 Works in `no_std` environments (optional, if you don’t depend on `std::error::Error`)
* 📚 Documented and unit-tested


## Installation

Add this line to your `Cargo.toml`:

```toml
[dependencies]
try-next = "0.2"
```

Then import the trait:

```rust
use try_next::{TryNext, TryNextWithContext};
```


## Design notes

- Both traits are deliberately **minimal**: they define no combinators or adapters.
  Their purpose is to provide a simple, low-level interface for fallible, stepwise
  data production.
- `TryNextWithContext` can often serve as a building block for adapters that
  integrate external state or resources.
- These traits are a good fit for *incremental* or *stateful* producers such as
  **parsers**, **lexers**, **tokenizers**, and other components that advance in
  discrete steps while potentially failing mid-stream.


## Related Work

- [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) —  
  The standard *synchronous, fallible, pull-based* trait for reading **bytes**.  
  `TryNext` is conceptually similar but works with **generic items** instead of raw byte buffers.

- [`Iterator<Item = Result<T, E>>`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) —  
  The idiomatic pattern for representing fallible iteration in the standard library.  
  Works well for most use cases but couples error handling with the iterator interface.

- [`fallible-iterator`](https://crates.io/crates/fallible-iterator) —  
  A rich abstraction for fallible iteration, including combinators and adapters.  
  Heavier than `TryNext`, but feature-complete if you need iterator-like utilities.

- [`fallible-streaming-iterator`](https://crates.io/crates/fallible-streaming-iterator) —  
  Similar to `fallible-iterator` but optimized for *borrowing* streams, avoiding allocations.

- [`futures::TryStream`](https://docs.rs/futures/latest/futures/stream/trait.TryStream.html) —  
  The *asynchronous* equivalent of this pattern.  
  Defines `try_poll_next` returning `Poll<Option<Result<T, E>>>` for non-blocking sources.


## License

Released under the terms of the GNU Lesser General Public License, version 3.0 or (at your option) any later version (LGPL-3.0-or-later).


## Contribution

Contributions are welcome!
Unless explicitly stated otherwise, any contribution intentionally submitted for inclusion in `try-next` by you shall be licensed as above, without any additional terms or conditions.


### Author

Copyright (c) 2005–2025 IKH Software, Inc.


[crates.io]: https://crates.io/crates/try-next
[docs.rs]: https://docs.rs/try-next
