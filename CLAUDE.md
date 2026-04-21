# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
cargo build              # build the crate
cargo test               # run all tests (unit tests are inline in src/)
cargo test <test_name>   # run a single test by name
cargo doc --open         # build and view documentation
```

## Architecture

`try-next` is a minimal, zero-dependency Rust crate providing synchronous, fallible, pull-based iteration traits. It targets `no_std` environments with optional `alloc` and `std` features.

### Feature flags

- **`std`** (default): enables `alloc` + the `io` module (`BufReader` impls)
- **`alloc`**: enables `try_collect()` / `try_collect_with_context()` methods (Vec collection)
- No features: `core`-only, bare trait definitions

### Core types (src/lib.rs)

- **`TryNext<S>`** trait — context-free fallible producer: `fn try_next(&mut self) -> Result<Option<Item>, Error>`. The generic `S` (default `()`) is an optional stats type returned by `stats()`.
- **`TryNextWithContext<C, S>`** trait — same contract but receives `&mut C` context on each call.
- **`IterInput<I>`** — adapter wrapping any `Iterator` into both traits using `Infallible` error. Auto-fuses (returns `None` forever after first `None`).

### I/O module (src/io.rs, behind `std` feature)

Implements `TryNext` and `TryNextWithContext` for `BufReader<R: Read>`, yielding one byte at a time.

### Conventions

- All tests are inline `#[cfg(test)]` modules within source files.
- The crate is `no_std`-compatible; guard `std`-only code with `#[cfg(feature = "std")]` and `alloc`-only code with `#[cfg(feature = "alloc")]`.
- Edition 2024. License: MIT.
