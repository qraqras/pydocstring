//! Example: Parsing NumPy-style docstrings

use pydocstring::NumPySectionBody;
use pydocstring::numpy::parse_numpy;

fn main() {
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

Examples
--------
>>> calculate_area(5.0, 3.0)
15.0
"#;

    let result = parse_numpy(docstring);
    let doc = &result;

    println!(
        "Summary: {}",
        doc.summary
            .as_ref()
            .map_or("", |s| s.source_text(&doc.source))
    );
    println!(
        "\nExtended Summary: {}",
        doc.extended_summary
            .as_ref()
            .map(|s| s.source_text(&doc.source))
            .unwrap_or_default()
    );

    for item in &doc.items {
        let section = match item {
            pydocstring::NumPyDocstringItem::Section(s) => s,
            pydocstring::NumPyDocstringItem::StrayLine(line) => {
                println!("\nStray line: {}", line.source_text(&doc.source));
                continue;
            }
        };
        match &section.body {
            NumPySectionBody::Parameters(params) => {
                println!("\nParameters:");
                for param in params {
                    let names: Vec<&str> = param
                        .names
                        .iter()
                        .map(|n| n.source_text(&doc.source))
                        .collect();
                    println!(
                        "  - {:?}: {:?}",
                        names,
                        param.r#type.as_ref().map(|t| t.source_text(&doc.source))
                    );
                    println!("    {}", param.description.source_text(&doc.source));
                }
            }
            NumPySectionBody::Returns(rets) => {
                println!("\nReturns:");
                for ret in rets {
                    println!(
                        "  Type: {:?}",
                        ret.return_type.as_ref().map(|t| t.source_text(&doc.source))
                    );
                    println!("  {}", ret.description.source_text(&doc.source));
                }
            }
            NumPySectionBody::Raises(excs) => {
                println!("\nRaises:");
                for exc in excs {
                    println!(
                        "  - {}: {}",
                        exc.r#type.source_text(&doc.source),
                        exc.description.source_text(&doc.source)
                    );
                }
            }
            NumPySectionBody::Notes(text) => {
                println!("\nNotes:");
                println!("  {}", text.source_text(&doc.source));
            }
            NumPySectionBody::Examples(text) => {
                println!("\nExamples:");
                println!("{}", text.source_text(&doc.source));
            }
            _ => {}
        }
    }
}
