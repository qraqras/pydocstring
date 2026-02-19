# pydocstring

A fast Rust parser for Python docstrings with full AST and source location tracking.

## Features

- 🚀 Fast parsing with zero external dependencies
- 📝 Docstring styles:
  - Google style — fully supported
  - NumPy style — fully supported
  - Sphinx style — not supported in v1 (planned for v2)
- 🎯 Accurate source spans (byte offsets) on every AST element
- 🩺 Diagnostic-based error reporting (partial results + diagnostics)
- 🦀 Pure Rust, no external crates
- ✅ Comprehensive test coverage (140+ tests)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pydocstring = "0.1.0"
```

## Usage

### NumPy Style

```rust
use pydocstring::numpy::parse_numpy;

let docstring = r#"
Calculate the sum of two numbers.

Parameters
----------
x : int
    The first number.
y : int
    The second number.

Returns
-------
int
    The sum of x and y.
"#;

let result = parse_numpy(docstring).unwrap();
println!("Summary: {}", result.summary);
println!("Parameters: {}", result.parameters.len());
```

## Project Structure

```
pydocstring/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── types.rs        # Common type definitions
│   ├── error.rs        # Error types
│   └── parser/
│       ├── mod.rs      # Parser module
│       ├── numpy.rs    # NumPy-style parser
│       ├── google.rs   # Google-style parser
│       └── sphinx.rs   # Sphinx-style parser
├── tests/              # Integration tests
├── examples/           # Usage examples
└── Cargo.toml
```

## Development

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run examples:
```bash
cargo run --example parse_numpy
```

## Implementation Progress

- [x] Project structure and common AST types
- [x] Source span tracking on all elements
- [x] Diagnostic-based error reporting
- [x] Google-style parser (fully implemented)
- [x] NumPy-style parser (fully implemented)
- [x] Style auto-detection
- [x] Unified `DocstringLike` trait
- [ ] Sphinx-style parser (planned for v2)
- [ ] Performance benchmarks

## License

MIT
