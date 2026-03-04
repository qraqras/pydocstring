# pydocstring

A fast, zero-dependency Rust parser for Python docstrings with full AST and source location tracking.

## Features

- **Zero external dependencies** — pure Rust implementation
- **Docstring styles:**
  - Google style — fully supported
  - NumPy style — fully supported
- **Accurate source spans** (byte offsets) on every AST node
- **Always succeeds** — returns a best-effort AST for any input, never panics
- **Style auto-detection** — automatically identifies NumPy or Google style
- **Comprehensive test coverage** (260+ tests)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pydocstring = "0.1.0"
```

## Quick Start

### NumPy Style

```rust
use pydocstring::numpy::parse_numpy;

let docstring = r#"
Calculate the area of a rectangle.

This function takes the width and height of a rectangle
and returns its area.

Parameters
----------
width : float
    The width of the rectangle.
height : float
    The height of the rectangle.

Returns
-------
float
    The area of the rectangle.

Raises
------
ValueError
    If width or height is negative.
"#;

let result = parse_numpy(docstring);

println!("Summary: {}", result.summary.as_ref().map_or("", |s| s.source_text(&result.source)));
for item in &result.items {
    if let pydocstring::NumPyDocstringItem::Section(s) = item {
        if let pydocstring::NumPySectionBody::Parameters(params) = &s.body {
            for param in params {
                let names: Vec<&str> = param.names.iter()
                    .map(|n| n.source_text(&result.source)).collect();
                println!("  {:?}: {:?}", names,
                    param.r#type.as_ref().map(|t| t.source_text(&result.source)));
            }
        }
    }
}
```

### Google Style

```rust
use pydocstring::google::parse_google;

let docstring = r#"
Calculate the area of a rectangle.

This function takes the width and height of a rectangle
and returns its area.

Args:
    width (float): The width of the rectangle.
    height (float): The height of the rectangle.

Returns:
    float: The area of the rectangle.

Raises:
    ValueError: If width or height is negative.
"#;

let result = parse_google(docstring);

println!("Summary: {}", result.summary.as_ref().map_or("", |s| s.source_text(&result.source)));
for item in &result.items {
    if let pydocstring::GoogleDocstringItem::Section(s) = item {
        if let pydocstring::GoogleSectionBody::Args(args) = &s.body {
            for arg in args {
                println!("  {} ({:?}): {}", arg.name.source_text(&result.source),
                    arg.r#type.as_ref().map(|t| t.source_text(&result.source)),
                    arg.description.source_text(&result.source));
            }
        }
    }
}
```

### Style Auto-Detection

```rust
use pydocstring::{detect_style, Style};

let numpy_doc = "Summary.\n\nParameters\n----------\nx : int\n    Desc.";
assert_eq!(detect_style(numpy_doc), Style::NumPy);

let google_doc = "Summary.\n\nArgs:\n    x: Desc.";
assert_eq!(detect_style(google_doc), Style::Google);
```

## Source Locations

Every AST node carries a `TextRange` (byte offsets) so linters can report precise positions:

```rust
use pydocstring::numpy::parse_numpy;

let docstring = "Summary.\n\nParameters\n----------\nx : int\n    Desc.";
let result = parse_numpy(docstring);

for item in &result.items {
    if let pydocstring::NumPyDocstringItem::Section(s) = item {
        if let pydocstring::NumPySectionBody::Parameters(params) = &s.body {
            if let Some(param) = params.first() {
                let name_range = &param.names[0];
                println!("Parameter '{}' at byte {}..{}",
                    name_range.source_text(&result.source),
                    name_range.start(), name_range.end());
            }
        }
    }
}
```

## Supported Sections

### NumPy Style

| Section | Body variant | Body type |
|---------|-------------|-----------|
| Parameters | `Parameters(...)` | `Vec<NumPyParameter>` |
| Other Parameters | `OtherParameters(...)` | `Vec<NumPyParameter>` |
| Receives | `Receives(...)` | `Vec<NumPyParameter>` |
| Returns | `Returns(...)` | `Vec<NumPyReturns>` |
| Yields | `Yields(...)` | `Vec<NumPyReturns>` |
| Raises | `Raises(...)` | `Vec<NumPyException>` |
| Warns | `Warns(...)` | `Vec<NumPyWarning>` |
| See Also | `SeeAlso(...)` | `Vec<SeeAlsoItem>` |
| Attributes | `Attributes(...)` | `Vec<NumPyAttribute>` |
| Methods | `Methods(...)` | `Vec<NumPyMethod>` |
| References | `References(...)` | `Vec<NumPyReference>` |
| Warnings | `Warnings(...)` | `Option<TextRange>` |
| Notes | `Notes(...)` | `Option<TextRange>` |
| Examples | `Examples(...)` | `Option<TextRange>` |

Additionally, `NumPyDocstring` has fields: `summary`, `deprecation`, `extended_summary`.

### Google Style

| Section | Body variant | Body type |
|---------|-------------|-----------|
| Args | `Args(...)` | `Vec<GoogleArg>` |
| Keyword Args | `KeywordArgs(...)` | `Vec<GoogleArg>` |
| Other Parameters | `OtherParameters(...)` | `Vec<GoogleArg>` |
| Receives | `Receives(...)` | `Vec<GoogleArg>` |
| Returns | `Returns(...)` | `GoogleReturns` |
| Yields | `Yields(...)` | `GoogleReturns` |
| Raises | `Raises(...)` | `Vec<GoogleException>` |
| Warns | `Warns(...)` | `Vec<GoogleWarning>` |
| See Also | `SeeAlso(...)` | `Vec<GoogleSeeAlsoItem>` |
| Attributes | `Attributes(...)` | `Vec<GoogleAttribute>` |
| Methods | `Methods(...)` | `Vec<GoogleMethod>` |
| Notes | `Notes(...)` | `TextRange` |
| Examples | `Examples(...)` | `TextRange` |
| Todo | `Todo(...)` | `TextRange` |
| References | `References(...)` | `TextRange` |
| Warnings | `Warnings(...)` | `TextRange` |

Admonition sections (Attention, Caution, Danger, Error, Hint, Important, Tip) are also supported as `TextRange` bodies.

Additionally, `GoogleDocstring` has fields: `summary`, `extended_summary`.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example parse_numpy
cargo run --example parse_google
```

## License

MIT
