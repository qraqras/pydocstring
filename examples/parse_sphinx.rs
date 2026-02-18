//! Example: Parsing Sphinx-style docstrings (placeholder)

use pydocstring::parser::sphinx::parse_sphinx;

fn main() {
    let docstring = r#"
Calculate the area of a rectangle.

This function takes the width and height of a rectangle
and returns its area.

:param width: The width of the rectangle.
:type width: float
:param height: The height of the rectangle.
:type height: float
:return: The area of the rectangle.
:rtype: float
"#;

    match parse_sphinx(docstring) {
        Ok(doc) => {
            println!("Summary: {}", doc.summary.value);
            println!("\n(Sphinx-style parser is not yet fully implemented)");
            println!("Parameters: {}", doc.parameters.len());
            println!("Has Returns: {}", doc.returns.is_some());
        }
        Err(e) => {
            eprintln!("Failed to parse docstring: {}", e);
        }
    }
}
