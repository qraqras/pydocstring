# pydocstring

A fast, zero-dependency Rust parser for Python docstrings.
Parses Google and NumPy style docstrings into a **unified syntax tree** with **byte-precise source locations** on every token.

## Why pydocstring?

Existing Python docstring parsers (docstring_parser, griffe, etc.) return flat lists of extracted values with no positional information. pydocstring is designed as **infrastructure for linters and formatters**:

- **Byte-precise source locations** — every `SyntaxToken` carries a `TextRange` (byte offset pair), so tools can emit diagnostics pointing to exact positions in the original text
- **Uniform syntax tree** — Google and NumPy styles produce the same `SyntaxNode` / `SyntaxToken` tree structure (inspired by [Biome](https://biomejs.dev/)), enabling style-agnostic tree traversal via `Visitor` + `walk`
- **Zero dependencies, never panics** — pure Rust with no external crates; always returns a best-effort tree for any input
- **Native performance** — suitable for embedding in Rust-based toolchains like Ruff

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pydocstring = "0.0.1"
```

## Quick Start

### NumPy Style

```rust
use pydocstring::numpy::{parse_numpy, nodes::NumPyDocstring};
use pydocstring::NumPySectionKind;

let docstring = "\
Calculate the area of a rectangle.

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
";

let result = parse_numpy(docstring);
let doc = NumPyDocstring::cast(result.root()).unwrap();

// Summary
println!("{}", doc.summary().unwrap().text(result.source()));

// Iterate sections
for section in doc.sections() {
    match section.section_kind(result.source()) {
        NumPySectionKind::Parameters => {
            for param in section.parameters() {
                let names: Vec<&str> = param.names()
                    .map(|n| n.text(result.source()))
                    .collect();
                let ty = param.r#type().map(|t| t.text(result.source()));
                println!("  {:?}: {:?}", names, ty);
            }
        }
        NumPySectionKind::Returns => {
            for ret in section.returns() {
                let ty = ret.return_type().map(|t| t.text(result.source()));
                println!("  -> {:?}", ty);
            }
        }
        _ => {}
    }
}
```

### Google Style

```rust
use pydocstring::google::{parse_google, nodes::GoogleDocstring};
use pydocstring::GoogleSectionKind;

let docstring = "\
Calculate the area of a rectangle.

Args:
    width (float): The width of the rectangle.
    height (float): The height of the rectangle.

Returns:
    float: The area of the rectangle.

Raises:
    ValueError: If width or height is negative.
";

let result = parse_google(docstring);
let doc = GoogleDocstring::cast(result.root()).unwrap();

// Summary
println!("{}", doc.summary().unwrap().text(result.source()));

// Iterate sections
for section in doc.sections() {
    match section.section_kind(result.source()) {
        GoogleSectionKind::Args => {
            for arg in section.args() {
                println!("  {} ({:?}): {:?}",
                    arg.name().text(result.source()),
                    arg.r#type().map(|t| t.text(result.source())),
                    arg.description().map(|d| d.text(result.source())));
            }
        }
        GoogleSectionKind::Raises => {
            for exc in section.exceptions() {
                println!("  {}: {:?}",
                    exc.r#type().text(result.source()),
                    exc.description().map(|d| d.text(result.source())));
            }
        }
        _ => {}
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

Every token carries a `TextRange` (byte offsets), so linters can report precise positions:

```rust
use pydocstring::numpy::{parse_numpy, nodes::NumPyDocstring};
use pydocstring::NumPySectionKind;

let docstring = "Summary.\n\nParameters\n----------\nx : int\n    The value.";
let result = parse_numpy(docstring);
let doc = NumPyDocstring::cast(result.root()).unwrap();

for section in doc.sections() {
    if section.section_kind(result.source()) == NumPySectionKind::Parameters {
        for param in section.parameters() {
            let name = param.names().next().unwrap();
            println!("Parameter '{}' at byte {}..{}",
                name.text(result.source()),
                name.range().start(),
                name.range().end());
            // => Parameter 'x' at byte 31..32
        }
    }
}
```

## Syntax Tree

The parse result is a tree of `SyntaxNode` (branches) and `SyntaxToken` (leaves), each tagged with a `SyntaxKind`. Use `pretty_print()` to visualize:

```rust
use pydocstring::google::parse_google;

let result = parse_google("Summary.\n\nArgs:\n    x (int): The value.");
println!("{}", result.pretty_print());
```

Output:
```
GOOGLE_DOCSTRING@0..42 {
    SUMMARY: "Summary."@0..8
    GOOGLE_SECTION@10..42 {
        GOOGLE_SECTION_HEADER@10..15 {
            NAME: "Args"@10..14
            COLON: ":"@14..15
        }
        GOOGLE_ARG@20..42 {
            NAME: "x"@20..21
            OPEN_BRACKET: "("@22..23
            TYPE: "int"@23..26
            CLOSE_BRACKET: ")"@26..27
            COLON: ":"@27..28
            DESCRIPTION: "The value."@29..39
        }
    }
}
```

### Visitor Pattern

Walk the tree with the `Visitor` trait for style-agnostic analysis:

```rust
use pydocstring::{Visitor, walk, SyntaxNode, SyntaxToken, SyntaxKind};
use pydocstring::google::parse_google;

struct NameCollector<'a> {
    source: &'a str,
    names: Vec<String>,
}

impl Visitor for NameCollector<'_> {
    fn visit_token(&mut self, token: &SyntaxToken) {
        if token.kind() == SyntaxKind::NAME {
            self.names.push(token.text(self.source).to_string());
        }
    }
}

let result = parse_google("Summary.\n\nArgs:\n    x: Desc.\n    y: Desc.");
let mut collector = NameCollector { source: result.source(), names: vec![] };
walk(result.root(), &mut collector);
assert_eq!(collector.names, vec!["Args", "x", "y"]);
```

## Supported Sections

### NumPy Style

| Section | Typed accessor | Entry type |
|---------|---------------|------------|
| Parameters / Other Parameters / Receives | `parameters()` | `NumPyParameter` |
| Returns / Yields | `returns()` | `NumPyReturns` |
| Raises | `exceptions()` | `NumPyException` |
| Warns | `warnings()` | `NumPyWarning` |
| See Also | `see_also_items()` | `NumPySeeAlsoItem` |
| References | `references()` | `NumPyReference` |
| Attributes | `attributes()` | `NumPyAttribute` |
| Methods | `methods()` | `NumPyMethod` |
| Notes / Examples / Warnings | `body_text()` | Free text |

Additional root-level elements: `summary()`, `extended_summary()`, `deprecation()`.

### Google Style

| Section | Typed accessor | Entry type |
|---------|---------------|------------|
| Args / Keyword Args / Other Parameters / Receives | `args()` | `GoogleArg` |
| Returns / Yields | `returns()` | `GoogleReturns` |
| Raises | `exceptions()` | `GoogleException` |
| Warns | `warnings()` | `GoogleWarning` |
| See Also | `see_also_items()` | `GoogleSeeAlsoItem` |
| Attributes | `attributes()` | `GoogleAttribute` |
| Methods | `methods()` | `GoogleMethod` |
| Notes / Examples / Todo / References / Warnings | `body_text()` | Free text |
| Admonitions (Attention, Caution, Danger, ...) | `body_text()` | Free text |

Additional root-level elements: `summary()`, `extended_summary()`.

## Development

```bash
cargo build          # Build
cargo test           # Run all 270+ tests
cargo run --example parse_numpy
cargo run --example parse_google
```

## License

MIT
