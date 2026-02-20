//! Example: Parsing Google-style docstrings

use pydocstring::google::parse_google;
use pydocstring::GoogleSectionBody;

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
    let doc = &result;

    println!("Summary: {}", doc.summary.value);
    if let Some(desc) = &doc.extended_summary {
        println!("Description: {}", desc.value);
    }

    let args: Vec<_> = doc.sections.iter().filter_map(|s| match &s.body {
        GoogleSectionBody::Args(v) => Some(v.iter()),
        _ => None,
    }).flatten().collect();
    println!("\nArgs ({}):", args.len());
    for arg in &args {
        let type_str = arg.r#type.as_ref().map(|t| t.value.as_str()).unwrap_or("?");
        println!(
            "  {} ({}): {}",
            arg.name.value, type_str, arg.description.value
        );
    }

    let returns: Vec<_> = doc.sections.iter().filter_map(|s| match &s.body {
        GoogleSectionBody::Returns(v) => Some(v.iter()),
        _ => None,
    }).flatten().collect();
    println!("\nReturns ({}):", returns.len());
    for ret in &returns {
        let type_str = ret
            .return_type
            .as_ref()
            .map(|t| t.value.as_str())
            .unwrap_or("?");
        println!("  {}: {}", type_str, ret.description.value);
    }

    let raises: Vec<_> = doc.sections.iter().filter_map(|s| match &s.body {
        GoogleSectionBody::Raises(v) => Some(v.iter()),
        _ => None,
    }).flatten().collect();
    println!("\nRaises ({}):", raises.len());
    for exc in &raises {
        println!("  {}: {}", exc.r#type.value, exc.description.value);
    }

    println!("\nSections ({}):", doc.sections.len());
    for section in &doc.sections {
        println!(
            "  {} (header: {:?})",
            section.header.name.value, section.header.range
        );
    }
}
