//! Example: Sphinx-style docstrings (not supported in v1)
//!
//! This example demonstrates that Sphinx-style docstrings are detected but
//! not fully parsed. The parser returns an error diagnostic.

use pydocstring::sphinx::parse_sphinx;

fn main() {
    let docstring = r#"
Calculate the area of a rectangle.

:param width: The width of the rectangle.
:type width: float
:param height: The height of the rectangle.
:type height: float
:return: The area of the rectangle.
:rtype: float
"#;

    let result = parse_sphinx(docstring);
    let doc = &result.value;
    println!("Summary: {}", doc.summary.value);
    println!();

    // Sphinx style is not supported in v1 — diagnostics will indicate this.
    for diag in &result.diagnostics {
        println!("[{:?}] {}", diag.severity, diag.message);
    }
}
