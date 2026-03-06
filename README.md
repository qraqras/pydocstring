# pydocstring

![Crates.io Version](https://img.shields.io/crates/v/pydocstring?color=CC6688)
![Crates.io MSRV](https://img.shields.io/crates/msrv/pydocstring?color=CC6688)
![Crates.io License](https://img.shields.io/crates/l/pydocstring?color=CC6688)

A zero-dependency Rust parser for Python docstrings (Google / NumPy style).

Produces a **unified syntax tree** with **byte-precise source locations** on every token — designed as infrastructure for linters and formatters.


## Features

- **Byte-precise source locations** — every token carries a `TextRange` (byte offset pair) for exact diagnostic positions
- **Unified syntax tree** — both styles produce the same `SyntaxNode` / `SyntaxToken` tree, enabling style-agnostic traversal via `Visitor` + `walk`
- **Zero dependencies** — pure Rust, no external crates
- **Never panics** — always returns a best-effort tree for any input
- **Style auto-detection** — `detect_style()` identifies the docstring convention automatically

## Installation

```toml
[dependencies]
pydocstring = "0.0.2"
```

## Usage

### Parsing

```rust
use pydocstring::google::{parse_google, nodes::GoogleDocstring};
use pydocstring::GoogleSectionKind;

let input = "Summary.\n\nArgs:\n    x (int): The value.\n    y (int): Another value.";
let result = parse_google(input);
let doc = GoogleDocstring::cast(result.root()).unwrap();

println!("{}", doc.summary().unwrap().text(result.source()));

for section in doc.sections() {
    if section.section_kind(result.source()) == GoogleSectionKind::Args {
        for arg in section.args() {
            println!("{}: {}",
                arg.name().text(result.source()),
                arg.r#type().map(|t| t.text(result.source())).unwrap_or(""));
        }
    }
}
```

NumPy style works the same way — use `parse_numpy` / `NumPyDocstring` instead.

### Style Auto-Detection

```rust
use pydocstring::{detect_style, Style};

assert_eq!(detect_style("Summary.\n\nArgs:\n    x: Desc."), Style::Google);
assert_eq!(detect_style("Summary.\n\nParameters\n----------\nx : int"), Style::NumPy);
```

### Source Locations

Every token carries byte offsets for precise diagnostics:

```rust
use pydocstring::google::{parse_google, nodes::GoogleDocstring};
use pydocstring::GoogleSectionKind;

let result = parse_google("Summary.\n\nArgs:\n    x (int): The value.");
let doc = GoogleDocstring::cast(result.root()).unwrap();

for section in doc.sections() {
    if section.section_kind(result.source()) == GoogleSectionKind::Args {
        for arg in section.args() {
            let name = arg.name();
            println!("'{}' at byte {}..{}",
                name.text(result.source()), name.range().start(), name.range().end());
        }
    }
}
```

### Syntax Tree

The parse result is a tree of `SyntaxNode` (branches) and `SyntaxToken` (leaves), each tagged with a `SyntaxKind`. Use `pretty_print()` to visualize:

```rust
use pydocstring::google::parse_google;

let result = parse_google("Summary.\n\nArgs:\n    x (int): The value.");
println!("{}", result.pretty_print());
```

```text
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
use pydocstring::{Visitor, walk, SyntaxToken, SyntaxKind};
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

Both styles support the following section categories. Typed accessor methods are available on each style's section node.

| Category                          | Google                                   | NumPy                                   |
|-----------------------------------|------------------------------------------|-----------------------------------------|
| Parameters                        | `args()` → `GoogleArg`                   | `parameters()` → `NumPyParameter`       |
| Returns / Yields                  | `returns()` → `GoogleReturns`            | `returns()` → `NumPyReturns`            |
| Raises                            | `exceptions()` → `GoogleException`       | `exceptions()` → `NumPyException`       |
| Warns                             | `warnings()` → `GoogleWarning`           | `warnings()` → `NumPyWarning`           |
| See Also                          | `see_also_items()` → `GoogleSeeAlsoItem` | `see_also_items()` → `NumPySeeAlsoItem` |
| Attributes                        | `attributes()` → `GoogleAttribute`       | `attributes()` → `NumPyAttribute`       |
| Methods                           | `methods()` → `GoogleMethod`             | `methods()` → `NumPyMethod`             |
| Free text (Notes, Examples, etc.) | `body_text()`                            | `body_text()`                           |

Root-level accessors: `summary()`, `extended_summary()` (NumPy also has `deprecation()`).

## Development

```bash
cargo build
cargo test
cargo run --example parse_google
cargo run --example parse_numpy
```
