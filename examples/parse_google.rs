//! Example: Parsing Google-style docstrings
//!
//! Shows the raw docstring text, then the detailed parsed AST.

use pydocstring::google::parse_google;

fn main() {
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

    let parsed = parse_google(docstring);

    // Display: raw source text
    println!("╔══════════════════════════════════════════════════╗");
    println!("║          Google-style Docstring Example          ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("── Display (raw text) ─────────────────────────────");
    println!("{}", parsed.source());

    // pretty_print: structured AST
    println!("── pretty_print (parsed AST) ──────────────────────");
    print!("{}", parsed.pretty_print());
}
