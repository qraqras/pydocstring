//! Example: Parsing Google-style docstrings

use pydocstring::google::parse_google;
use pydocstring::{GoogleDocstringItem, GoogleSectionBody};

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

    println!("Summary: {}", doc.summary.source_text(&doc.source));
    if let Some(desc) = &doc.extended_summary {
        println!("Description: {}", desc.source_text(&doc.source));
    }

    let args: Vec<_> = doc
        .items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Args(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect();
    println!("\nArgs ({}):", args.len());
    for arg in &args {
        let type_str = arg
            .r#type
            .as_ref()
            .map(|t| t.source_text(&doc.source))
            .unwrap_or("?");
        println!(
            "  {} ({}): {}",
            arg.name.source_text(&doc.source),
            type_str,
            arg.description.source_text(&doc.source)
        );
    }

    let ret = doc.items.iter().find_map(|item| match item {
        GoogleDocstringItem::Section(s) => match &s.body {
            GoogleSectionBody::Returns(r) => Some(r),
            _ => None,
        },
        _ => None,
    });
    if let Some(ret) = ret {
        let type_str = ret
            .return_type
            .as_ref()
            .map(|t| t.source_text(&doc.source))
            .unwrap_or("?");
        println!(
            "\nReturns: {}: {}",
            type_str,
            ret.description.source_text(&doc.source)
        );
    }

    let raises: Vec<_> = doc
        .items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect();
    println!("\nRaises ({}):", raises.len());
    for exc in &raises {
        println!(
            "  {}: {}",
            exc.r#type.source_text(&doc.source),
            exc.description.source_text(&doc.source)
        );
    }

    let all_sections: Vec<_> = doc
        .items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .collect();
    println!("\nSections ({}):", all_sections.len());
    for section in &all_sections {
        println!(
            "  {} (header: {:?})",
            section.header.name.source_text(&doc.source),
            section.header.range
        );
    }
}
