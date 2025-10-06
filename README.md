# try-next &nbsp; [![Crates.io](https://img.shields.io/crates/v/try-next.svg)](https://crates.io/crates/try-next) [![Documentation](https://docs.rs/parlex/badge.svg)](https://docs.rs/try-next) [![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL%203.0--or--later-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)

A minimal **synchronous** trait for fallible, pull-based item sources.

## Overview

`try-next` provides a single trait, [`TryNext`], that defines a lightweight interface for producing items one at a time, where advancing to the next item may fail.

Each call to [`try_next`](https://docs.rs/try-next/latest/try_next/trait.TryNext.html#tymethod.try_next) returns:

- `Ok(Some(item))` — a successfully produced item,  
- `Ok(None)` — when the source is exhausted, and  
- `Err(error)` — if an error occurred while fetching the next item.

This trait is intentionally **synchronous** and does not depend on `async`, `Poll`, or the `futures` crate. It’s ideal for parsers, readers, or generators that yield data step by step and may fail.

## Example

A small source that yields a few numbers, then ends normally, then errors:

```rust
use try_next::TryNext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyError {
    Broken,
}

struct Demo {
    state: u8,
}

impl TryNext for Demo {
    type Item = u8;
    type Error = MyError;

    fn try_next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            0..=2 => {
                let v = self.state;
                self.state += 1;
                Ok(Some(v)) // produce some items
            }
            3 => {
                self.state += 1;
                Ok(None) // normal end
            }
            _ => Err(MyError::Broken), // error afterwards
        }
    }
}

fn main() {
    let mut src = Demo { state: 0 };

    println!("{:?}", src.try_next()); // Ok(Some(0))
    println!("{:?}", src.try_next()); // Ok(Some(1))
    println!("{:?}", src.try_next()); // Ok(Some(2))
    println!("{:?}", src.try_next()); // Ok(None)
    println!("{:?}", src.try_next()); // Err(Broken)
}
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
try-next = "0.1"
```

Then import the trait:

```rust
use try_next::TryNext;
```

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
