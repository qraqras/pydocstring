# pydocstring

A fast Rust parser for Python docstrings (NumPy, Google, and Sphinx styles)

## Features

- 🚀 Fast parsing with zero external dependencies
- 📝 Support for multiple docstring styles:
  - NumPy style (fully implemented)
  - Google style (in progress)
  - Sphinx style (planned)
- 🦀 Pure Rust implementation
- ✅ Comprehensive test coverage

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

- [x] Project structure
- [x] Common types and error handling
- [x] NumPy-style parser (basic implementation)
- [ ] NumPy-style parser (advanced features)
- [ ] Google-style parser
- [ ] Sphinx-style parser
- [ ] Performance benchmarks

## License

MIT
