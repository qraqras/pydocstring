# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-03-09

### Added

- Google style docstring parsing (`parse_google`)
- NumPy style docstring parsing (`parse_numpy`)
- Automatic style detection (`detect_style`)
- Unified model IR (`Docstring`, `Section`, `Parameter`, `Return`, etc.)
- Emit back to Google style (`emit_google`)
- Emit back to NumPy style (`emit_numpy`)
- Full syntax tree (AST) with byte-precise source locations (`TextRange`)
- Tree traversal via `walk` and visitor pattern
- Pretty-print for AST debugging (`pretty_print`)
- Conversion from AST to unified model (`to_model`)
- Support for all standard sections:
  - Parameters / Args / Keyword Args / Other Parameters
  - Returns / Yields
  - Raises / Warns
  - Attributes / Methods
  - See Also / References
  - Deprecation
  - Free-text sections (Notes, Examples, Warnings, Todo, etc.)
- Error-resilient parsing — never panics on malformed input
- Zero external crate dependencies
- Python bindings via PyO3 (`pydocstring-rs`)

[0.1.0]: https://github.com/qraqras/pydocstring/releases/tag/v0.1.0
