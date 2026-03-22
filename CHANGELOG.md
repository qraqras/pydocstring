# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-03-22

### Added

- `SyntaxKind::GOOGLE_YIELDS` — dedicated node kind for entries inside a Google
  `Yields:` section. Previously these were emitted as `GOOGLE_RETURNS`.
- `SyntaxKind::NUMPY_YIELDS` — dedicated node kind for entries inside a NumPy
  `Yields` section. Previously these were emitted as `NUMPY_RETURNS`.
- `GoogleYields` typed wrapper with `return_type()`, `colon()`, and
  `description()` accessors (analogous to `GoogleReturns`).
- `NumPyYields` typed wrapper with `name()`, `colon()`, `return_type()`, and
  `description()` accessors (analogous to `NumPyReturns`).
- `GoogleSection::yields()` — accessor returning the `GoogleYields` node for
  a Yields section, distinct from `returns()`.
- `NumPySection::yields()` — accessor returning an iterator of `NumPyYields`
  nodes for a Yields section, distinct from `returns()`.
- Python bindings: `SyntaxKind.GOOGLE_YIELDS`, `SyntaxKind.NUMPY_YIELDS`,
  `GoogleYields` class, `NumPyYields` class, `GoogleSection.yields` property,
  and `NumPySection.yields` property.

### Changed

- Google parser: `Yields:` sections now produce `GOOGLE_YIELDS` child nodes
  instead of `GOOGLE_RETURNS`.
- NumPy parser: `Yields` sections now produce `NUMPY_YIELDS` child nodes
  instead of `NUMPY_RETURNS`.
- `to_model` (Google & NumPy): `Yields` sections now use the `yields()`
  accessor on the typed section wrapper rather than sharing the `returns()`
  code path.

## [0.1.4] - 2026-03-20

### Added

- `Style::Plain` — new style variant returned by `detect_style` for docstrings
  that contain no NumPy section underlines or Google section headers (e.g.
  summary-only docstrings, Sphinx-style docstrings).
- `SyntaxKind::PLAIN_DOCSTRING` — root node kind for plain-style parse trees.
- `parse_plain(input)` — lightweight parser that extracts only a `SUMMARY` and
  an optional `EXTENDED_SUMMARY` token from the input, without attempting
  section detection.
- `parse(input)` — unified entry point that calls `detect_style` and dispatches
  to `parse_google`, `parse_numpy`, or `parse_plain` automatically.
- `PlainDocstring` typed wrapper with `summary()` and `extended_summary()`
  accessors (mirrors the existing `GoogleDocstring` / `NumPyDocstring` API).
- Python bindings: `Style.PLAIN`, `SyntaxKind.PLAIN_DOCSTRING`, `PlainDocstring`
  class, and `parse_plain(input)` function.
- Google parser: zero-length `DESCRIPTION` token emitted when a colon is
  present but no description text follows (e.g. `a (int):`, `a:`), and
  zero-length `TYPE` token emitted for empty brackets `()`.
- NumPy parser: zero-length `TYPE` token emitted when a colon is present but
  type text is absent (e.g. `a :`); zero-length `DEFAULT_VALUE` token emitted
  when a default separator is present but no value follows (e.g. `default =`).
  Callers can use `find_missing(KIND)` to detect these absent-but-declared
  slots without inspecting surrounding tokens.
- `examples/parse_auto.rs` — demonstrates the unified `parse()` entry point
  with Google, NumPy, and plain-style inputs.

### Changed

- `detect_style` rewritten as a single O(n) pass; returns `Style::Plain` as the
  fallback instead of `Style::Google`.

## [0.1.3] - 2026-03-19

### Added

- Section name matching now accepts additional singular and alias forms for both
  Google and NumPy styles:
  - `"arg"`, `"param"`, `"keyword arg"`, `"keyword param"`, `"other arg"`,
    `"other param"`, `"method"`, `"reference"` (Google)
  - `"arguments"`, `"argument"`, `"args"`, `"arg"`, `"other arguments"`,
    `"other argument"`, `"other args"`, `"other arg"`, `"attribute"`,
    `"method"`, `"reference"` (NumPy)
  - Common typos tolerated: `"argment"`, `"paramter"` (Google)

### Fixed

- Google parser: arg entries with no description (e.g. `b :`) inside a section
  body were incorrectly classified as new section headers. Fixed by comparing
  the indentation of each line against the current section header's indentation
  and skipping header detection for more-indented lines.

### Changed

- Refactored Google entry header parsing to use a left-to-right confirmation
  algorithm. Handles missing close brackets, missing colons, and text after
  brackets without a colon more robustly. `close_bracket` in `TypeInfo` is now
  `Option<TextRange>` to represent the missing-bracket case.
- Added `rustfmt.toml` (`max_width = 120`) and reformatted all source files.

## [0.1.2] - 2026-03-16

### Added

- `LineColumn` struct (`lineno`, `col`) in `text.rs` for representing
  line/column positions; `lineno` is 1-based, `col` is a 0-based byte offset
  within the line.
- `LineIndex` in `text.rs` — a newline-offset lookup table built from source
  text; converts any `TextSize` byte offset to `LineColumn` in O(log n).
- `Parsed::line_col(offset: TextSize) -> LineColumn` method for resolving
  byte offsets in the syntax tree to line/column positions.
- Python bindings: `LineColumn` class with `lineno` and `col` properties.
  `col` is expressed in **Unicode codepoints** (compatible with Python's
  `ast` module convention) rather than raw bytes.
- Python bindings: `GoogleDocstring.line_col(offset)` and
  `NumPyDocstring.line_col(offset)` methods; `offset` is typically obtained
  from `Token.range.start` or `Token.range.end`.

## [0.1.1] - 2026-03-10

### Added

- Python bindings: `SyntaxKind` enum exposed as `enum.IntEnum`, usable for
  pattern matching on `Token.kind` and `Node.kind` instead of raw strings.

### Fixed

- `emit_google` / `emit_numpy` now correctly apply `base_indent` to all lines
  of the emitted docstring, not just the first line.

### Changed

- Python bindings: `Token.kind` and `Node.kind` now return `SyntaxKind` instead
  of `str`.

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

[0.1.4]: https://github.com/qraqras/pydocstring/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/qraqras/pydocstring/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/qraqras/pydocstring/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/qraqras/pydocstring/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/qraqras/pydocstring/releases/tag/v0.1.0
