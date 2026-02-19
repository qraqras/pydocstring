//! Example: Parsing Google-style docstrings

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

    let result = parse_google(docstring);
    let doc = &result.value;

    println!("Summary: {}", doc.summary.value);
    if let Some(desc) = &doc.description {
        println!("Description: {}", desc.value);
    }
    println!("\nArgs ({}):", doc.args().len());
    for arg in doc.args() {
        let type_str = arg
            .arg_type
            .as_ref()
            .map(|t| t.value.as_str())
            .unwrap_or("?");
        println!(
            "  {} ({}): {}",
            arg.name.value, type_str, arg.description.value
        );
    }
    println!("\nReturns ({}):", doc.returns().len());
    for ret in doc.returns() {
        let type_str = ret
            .return_type
            .as_ref()
            .map(|t| t.value.as_str())
            .unwrap_or("?");
        println!("  {}: {}", type_str, ret.description.value);
    }
    println!("\nRaises ({}):", doc.raises().len());
    for exc in doc.raises() {
        println!("  {}: {}", exc.exception_type.value, exc.description.value);
    }
    println!("\nSections ({}):", doc.sections.len());
    for section in &doc.sections {
        println!(
            "  {} (header: {:?})",
            section.header.name.value, section.header.range
        );
    }

    if !result.diagnostics.is_empty() {
        println!("\nDiagnostics:");
        for d in &result.diagnostics {
            println!("  {}", d);
        }
    }
}
