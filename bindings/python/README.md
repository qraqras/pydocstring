# pydocstring-rs

[![PyPI - Version](https://img.shields.io/pypi/v/pydocstring-rs?color=0062A8)](https://pypi.org/project/pydocstring-rs/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/pydocstring-rs?color=0062A8)](https://devguide.python.org/versions/)
[![Crates.io Version](https://img.shields.io/crates/v/pydocstring?color=FFC12d)](https://crates.io/crates/pydocstring)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/pydocstring?color=FFC12d)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0)

Python bindings for [pydocstring](https://crates.io/crates/pydocstring) — a zero-dependency Rust parser for Python docstrings (Google and NumPy styles).

Produces a **unified syntax tree** with **byte-precise source locations** on every token — designed as infrastructure for linters and formatters.

## Features

- **Full syntax tree** — builds a complete AST, not just extracted fields; traverse it with `walk()`
- **Typed objects per style** — style-specific classes like `GoogleArg`, `NumPyParameter`
- **Byte-precise source locations** — every token carries its exact byte range for pinpoint diagnostics
- **Powered by Rust** — native extension with no Python runtime overhead
- **Error-resilient** — never raises exceptions; malformed input still yields a best-effort tree
- **Style auto-detection** — hand it a docstring, get back `Style.GOOGLE`, `Style.NUMPY`, or `Style.PLAIN`

## Installation

```bash
pip install pydocstring-rs
```

## Usage

### Unified Parse (auto-detect)

Use `parse()` when you don't know the style in advance.
The returned object has a `.style` property so you can dispatch without `isinstance` checks:

```python
from pydocstring import parse, Style

doc = parse(source)

match doc.style:
    case Style.GOOGLE:
        for arg in doc.sections[0].args:
            print(arg.name.text, arg.description.text)
    case Style.NUMPY:
        for param in doc.sections[0].parameters:
            print([n.text for n in param.names], param.description.text)
    case Style.PLAIN:
        print(doc.summary.text)
```

When you only need the style-independent model, no dispatch is necessary:

```python
model = parse(source).to_model()  # works for all three styles
```

If you already know the style, prefer the explicit functions `parse_google()`,
`parse_numpy()`, or `parse_plain()` — they return a concrete type and are
slightly more efficient.

### Style Detection

```python
from pydocstring import detect_style, Style

detect_style("Summary.\n\nArgs:\n    x: Desc.")       # Style.GOOGLE
detect_style("Summary.\n\nParameters\n----------\n")  # Style.NUMPY
detect_style("Just a summary.")                       # Style.PLAIN
```

`Style.PLAIN` covers docstrings with no recognised section markers:
summary-only, summary + extended, and unrecognised styles such as Sphinx.

### Plain Style

Docstrings with no NumPy or Google section markers are parsed as plain:

```python
from pydocstring import parse_plain

doc = parse_plain("""Brief summary.

More detail here.
Spanning multiple lines.
""")

print(doc.summary.text)            # "Brief summary."
print(doc.extended_summary.text)   # "More detail here.\nSpanning multiple lines."
```

Unrecognised styles such as Sphinx are also treated as plain: the `:param:`
lines are preserved verbatim in `extended_summary`.

### Google Style

```python
from pydocstring import parse_google

doc = parse_google("""Summary line.

Args:
    x (int): The first value.
    y (str): The second value.

Returns:
    bool: True if successful.

Raises:
    ValueError: If x is negative.
""")

# Summary
print(doc.summary.text)  # "Summary line."

# Sections
for section in doc.sections:
    print(section.kind)  # "Args", "Returns", "Raises"

    for arg in section.args:
        print(f"  {arg.name.text}: {arg.type.text} — {arg.description.text}")

    if section.returns:
        r = section.returns
        print(f"  -> {r.return_type.text}: {r.description.text}")

    for exc in section.exceptions:
        print(f"  raises {exc.type.text}: {exc.description.text}")
```

### NumPy Style

```python
from pydocstring import parse_numpy

doc = parse_numpy("""Summary line.

Parameters
----------
x : int
    The first value.
y : str
    The second value.

Returns
-------
bool
    True if successful.
""")

print(doc.summary.text)  # "Summary line."

for section in doc.sections:
    print(section.kind)  # "Parameters", "Returns"

    for param in section.parameters:
        names = [n.text for n in param.names]
        print(f"  {names}: {param.type.text} — {param.description.text}")

    for ret in section.returns:
        print(f"  -> {ret.return_type.text}: {ret.description.text}")
```

### AST Access

Every parsed result exposes the full syntax tree via the `node` property:

```python
doc = parse_google("Summary.\n\nArgs:\n    x (int): Value.")

# Raw tree node
print(doc.node.kind)      # "GOOGLE_DOCSTRING"
print(doc.node.children)  # list of Node and Token objects

# Pretty-printed tree
print(doc.pretty_print())
```

Output:

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
      DESCRIPTION: "Value."@29..35
    }
  }
}
```

### Tree Traversal

Use `walk()` for depth-first traversal of the syntax tree:

```python
from pydocstring import parse_google, walk, Token

doc = parse_google("Summary.\n\nArgs:\n    x (int): Value.")

for item in walk(doc.node):
    if isinstance(item, Token) and item.kind == "NAME":
        print(item.text)  # "Args", "x"
```

### Source Locations

All tokens carry byte-precise source ranges:

```python
doc = parse_google("Summary.\n\nArgs:\n    x (int): Value.")
token = doc.summary
print(token.range.start, token.range.end)  # 0 8
```

### Style-Independent Model (IR)

Convert any parsed docstring into a style-independent intermediate representation for analysis or transformation:

```python
from pydocstring import parse_google

parsed = parse_google("Summary.\n\nArgs:\n    x (int): The value.\n")
doc = parsed.to_model()

print(doc.summary)  # "Summary."

for section in doc.sections:
    if section.kind == "Parameters":
        for param in section.parameters:
            print(param.names)            # ["x"]
            print(param.type_annotation)  # "int"
            print(param.description)      # "The value."
```

### Emitting (Code Generation)

Re-emit a `Docstring` model in any style — useful for style conversion or formatting:

```python
from pydocstring import Docstring, Section, Parameter, emit_google, emit_numpy

doc = Docstring(
    summary="Brief summary.",
    sections=[
        Section(
            "Parameters",
            parameters=[
                Parameter(
                    ["x"],
                    type_annotation="int",
                    description="The value.",
                ),
            ],
        ),
    ],
)

google = emit_google(doc)
print(google)  # Contains "Args:"

numpy = emit_numpy(doc)
print(numpy)  # Contains "Parameters\n----------"
```

Combine parsing and emitting to convert between styles:

```python
from pydocstring import parse_google, emit_numpy

parsed = parse_google("Summary.\n\nArgs:\n    x (int): The value.\n")
doc = parsed.to_model()
numpy_text = emit_numpy(doc)
print(numpy_text)  # Contains "Parameters\n----------"
```

## API Reference

### Functions

| Function             | Returns                                         | Description                                                   |
|----------------------|-------------------------------------------------|---------------------------------------------------------------|
| `parse(text)`        | `GoogleDocstring \| NumPyDocstring \| PlainDocstring` | Auto-detect style and parse                             |
| `parse_google(text)` | `GoogleDocstring`                               | Parse a Google-style docstring                                |
| `parse_numpy(text)`  | `NumPyDocstring`                                | Parse a NumPy-style docstring                                 |
| `parse_plain(text)`  | `PlainDocstring`                                | Parse a plain docstring (no section markers)                  |
| `detect_style(text)` | `Style`                                         | Detect style: `Style.GOOGLE`, `Style.NUMPY`, or `Style.PLAIN` |
| `emit_google(doc)`   | `str`                                           | Emit a `Docstring` model as Google-style text                 |
| `emit_numpy(doc)`    | `str`                                           | Emit a `Docstring` model as NumPy-style text                  |

### Objects

| Class             | Key Properties                                                                                                   |
|-------------------|------------------------------------------------------------------------------------------------------------------|
| `Style`           | `GOOGLE`, `NUMPY`, `PLAIN` (enum)                                                                                |
| `GoogleDocstring` | `style`, `summary`, `extended_summary`, `sections`, `node`, `source`, `pretty_print()`, `to_model()`             |
| `GoogleSection`   | `kind`, `args`, `returns`, `exceptions`, `body_text`, `node`                                                     |
| `GoogleArg`       | `name`, `type`, `description`, `optional`                                                                        |
| `GoogleReturns`   | `return_type`, `description`                                                                                     |
| `GoogleException` | `type`, `description`                                                                                            |
| `PlainDocstring`  | `style`, `summary`, `extended_summary`, `node`, `source`, `pretty_print()`, `to_model()`                         |
| `NumPyDocstring`  | `style`, `summary`, `extended_summary`, `sections`, `node`, `source`, `pretty_print()`, `to_model()`             |
| `NumPySection`    | `kind`, `parameters`, `returns`, `exceptions`, `body_text`, `node`                                               |
| `NumPyParameter`  | `names`, `type`, `description`, `optional`, `default_value`                                                      |
| `NumPyReturns`    | `name`, `return_type`, `description`                                                                             |
| `NumPyException`  | `type`, `description`                                                                                            |
| `Token`           | `kind`, `text`, `range`                                                                                          |
| `Node`            | `kind`, `range`, `children`                                                                                      |
| `TextRange`       | `start`, `end`                                                                                                   |
| `Docstring`       | `summary`, `extended_summary`, `deprecation`, `sections`                                                         |
| `Section` (model) | `kind`, `parameters`, `returns`, `exceptions`, `attributes`, `methods`, `see_also_entries`, `references`, `body` |
| `Parameter`       | `names`, `type_annotation`, `description`, `is_optional`, `default_value`                                        |
| `Return`          | `name`, `type_annotation`, `description`                                                                         |
| `ExceptionEntry`  | `type_name`, `description`                                                                                       |
| `Attribute`       | `name`, `type_annotation`, `description`                                                                         |
| `Method`          | `name`, `type_annotation`, `description`                                                                         |
| `SeeAlsoEntry`    | `names`, `description`                                                                                           |
| `Reference`       | `number`, `content`                                                                                              |
| `Deprecation`     | `version`, `description`                                                                                         |

## Development

### Prerequisites

- Rust (stable)
- Python 3.10+
- maturin

### Build

```bash
cd bindings/python

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Verify
python -c "import pydocstring; print(pydocstring.detect_style('Args:\n    x: y'))"
```

### Build a wheel

```bash
maturin build --release
# Output: target/wheels/pydocstring-*.whl
```

### Publish to PyPI

```bash
maturin publish
```
