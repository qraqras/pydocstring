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
    let doc = &result.value;
    {
        println!("Summary: {}", doc.summary.value);
        println!(
            "\nExtended Summary: {}",
            doc.extended_summary
                .as_ref()
                .map(|s| s.value.as_str())
                .unwrap_or_default()
        );

        for section in &doc.sections {
            match &section.body {
                NumPySectionBody::Parameters(params) => {
                    println!("\nParameters:");
                    for param in params {
                        let names: Vec<&str> =
                            param.names.iter().map(|n| n.value.as_str()).collect();
                        println!(
                            "  - {:?}: {:?}",
                            names,
                            param.r#type.as_ref().map(|t| t.value.as_str())
                        );
                        println!("    {}", param.description.value);
                    }
                }
                NumPySectionBody::Returns(rets) => {
                    println!("\nReturns:");
                    for ret in rets {
                        println!(
                            "  Type: {:?}",
                            ret.return_type.as_ref().map(|t| t.value.as_str())
                        );
                        println!("  {}", ret.description.value);
                    }
                }
                NumPySectionBody::Raises(excs) => {
                    println!("\nRaises:");
                    for exc in excs {
                        println!("  - {}: {}", exc.r#type.value, exc.description.value);
                    }
                }
                NumPySectionBody::Notes(text) => {
                    println!("\nNotes:");
                    println!("  {}", text.value);
                }
                NumPySectionBody::Examples(text) => {
                    println!("\nExamples:");
                    println!("{}", text.value);
                }
                _ => {}
            }
        }

        if !result.diagnostics.is_empty() {
            println!("\nDiagnostics:");
            for d in &result.diagnostics {
                println!("  {}", d);
            }
        }
    }
}
