# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-10-14

### Added

* **`IterInput<I>` adapter**

  * Wraps any standard [`Iterator`] and exposes it as a [`TryNext`] and [`TryNextWithContext<C>`] source.
  * Automatically **fuses** the iterator internally, ensuring that once it returns `None`, all subsequent calls also return `None`.
  * Always uses the [`Infallible`] error type and ignores the context parameter.
  * Useful for bridging ordinary iterators into APIs that expect fallible or context-aware producers.

### Breaking Change

* **`TryNextWithContext` type parameterization**

  * The associated type `Context` has been moved to a type parameter:
    from `trait TryNextWithContext { type Context; ... }`
    to `trait TryNextWithContext<C> { ... }`.
  * Implementations must now specify the context type as a parameter, e.g.
    `impl TryNextWithContext<MyContext> for MyProducer { ... }`.
  * This change also removes the need for PhantomData markers in generic
    implementations like [IterInput<I>], since the context type is no longer
    part of the trait’s associated types.


## [0.3.0] - 2025-10-13

### Added

* **`alloc` and `std` feature**

  * Added feature gating for heap-allocated types (`Vec` etc.).
  * `std` is enabled by default.
  * Enables `no_std` builds without allocation.

* **`try_collect` and `try_collect_with_context` methods:**

  * Collect all yielded items into a `Vec<T>`.
  * Available only when the `std` or `alloc` feature is enabled.

## [0.2.0] - 2025-10-07

### Added

- **`TryNextWithContext`** trait — a context-aware variant of `TryNext` for producing items
  with the help of an external, mutable context.
  - Each call to `try_next_with_context` receives a mutable reference to user-supplied state.
  - Designed for use cases such as parsers, lexers, or tokenizers that require external buffers,
    configuration, or shared mutable state.

### Notes

- `TryNextWithContext` is fully backward-compatible with `TryNext` and introduces no breaking
  changes to existing implementations.


## [0.1.0] - 2025-10-06

### Added

- Initial release defining the **`TryNext`** trait — a synchronous, fallible, pull-based
  interface for producing items one at a time.
- Documentation, examples, and tests for basic fallible iteration patterns.

