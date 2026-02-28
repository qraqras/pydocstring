# pydocstring

A fast, zero-dependency Rust parser for Python docstrings with full AST and source location tracking.

## Features

- **Zero external dependencies** — pure Rust implementation
- **Docstring styles:**
  - Google style — fully supported
  - NumPy style — fully supported
- **Accurate source spans** (byte offsets) on every AST node
- **Diagnostic-based error reporting** — partial results + diagnostics, never panics
- **Style auto-detection** — automatically identifies NumPy or Google style
- **Comprehensive test coverage** (140+ tests)

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

## Diagnostic-Based Error Handling

Parsers always return a result — even for malformed input. Diagnostics are collected alongside the best-effort AST:

```rust
use pydocstring::google::parse_google;

let result = parse_google("Summary.\n\nArgs:\n    : missing name");

if result.has_errors() {
    for diag in result.errors() {
        eprintln!("{}", diag); // e.g. "error at 14..28: ..."
    }
}

// The AST is still available
println!("Summary: {}", result.summary.as_ref().map_or("", |s| s.source_text(&result.source)));
```

## Source Locations

Every AST node carries a `TextRange` (byte offsets) so linters can report precise positions:

```rust
use pydocstring::numpy::parse_numpy;
use pydocstring::LineIndex;

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

                // Convert to line/column if needed
                let index = LineIndex::from_source(docstring);
                let (line, col) = index.line_col(name_range.start());
                println!("  line {}, col {}", line, col);
            }
        }
    }
}
```

## Supported Sections

### NumPy Style

| Section | Method | Return Type |
|---------|--------|-------------|
| Parameters | `parameters()` | `Vec<&NumPyParameter>` |
| Other Parameters | `other_parameters()` | `Vec<&NumPyParameter>` |
| Returns | `returns()` | `Vec<&NumPyReturns>` |
| Yields | `yields()` | `Vec<&NumPyReturns>` |
| Receives | `receives()` | `Vec<&NumPyParameter>` |
| Raises | `raises()` | `Vec<&NumPyException>` |
| Warns | `warns()` | `Vec<&NumPyWarning>` |
| Warnings | `warnings()` | `Option<&TextRange>` |
| See Also | `see_also()` | `Vec<&SeeAlsoItem>` |
| Notes | `notes()` | `Option<&TextRange>` |
| References | `references()` | `Vec<&NumPyReference>` |
| Examples | `examples()` | `Option<&TextRange>` |
| Attributes | `attributes()` | `Vec<&NumPyAttribute>` |
| Methods | `methods()` | `Vec<&NumPyMethod>` |

Additionally, `NumPyDocstring` has fields: `summary`, `deprecation`, `extended_summary`.

### Google Style

| Section | Method | Return Type |
|---------|--------|-------------|
| Args | `args()` | `Vec<&GoogleArgument>` |
| Keyword Args | `keyword_args()` | `Vec<&GoogleArgument>` |
| Other Parameters | `other_parameters()` | `Vec<&GoogleArgument>` |
| Returns | `returns()` | `Vec<&GoogleReturns>` |
| Yields | `yields()` | `Vec<&GoogleReturns>` |
| Receives | `receives()` | `Vec<&GoogleArgument>` |
| Raises | `raises()` | `Vec<&GoogleException>` |
| Warns | `warns()` | `Vec<&GoogleWarning>` |
| Warnings | `warnings()` | `Option<&TextRange>` |
| See Also | `see_also()` | `Vec<&GoogleSeeAlsoItem>` |
| Notes | `notes()` | `Option<&TextRange>` |
| References | `references()` | `Option<&TextRange>` |
| Examples | `examples()` | `Option<&TextRange>` |
| Attributes | `attributes()` | `Vec<&GoogleAttribute>` |
| Methods | `methods()` | `Vec<&GoogleMethod>` |
| Todo | `todo()` | `Option<&TextRange>` |

Additionally, `GoogleDocstring` has fields: `summary`, `description`, and admonition sections (Attention, Caution, Danger, etc.).

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
