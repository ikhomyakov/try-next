# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


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

