//! Example: Parsing NumPy-style docstrings

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
            if let Some(sig) = &doc.signature {
                println!("Signature: {}", sig.value);
            }
            println!("Summary: {}", doc.summary.value);
            println!(
                "\nExtended Summary: {}",
                doc.extended_summary
                    .as_ref()
                    .map(|s| s.value.as_str())
                    .unwrap_or_default()
            );

            println!("\nParameters:");
            for param in doc.parameters() {
                let names: Vec<&str> = param.names.iter().map(|n| n.value.as_str()).collect();
                println!(
                    "  - {:?}: {:?}",
                    names,
                    param.param_type.as_ref().map(|t| t.value.as_str())
                );
                println!("    {}", param.description.value);
            }

            if !doc.returns().is_empty() {
                println!("\nReturns:");
                for ret in doc.returns() {
                    println!(
                        "  Type: {:?}",
                        ret.return_type.as_ref().map(|t| t.value.as_str())
                    );
                    println!("  {}", ret.description.value);
                }
            }

            if !doc.raises().is_empty() {
                println!("\nRaises:");
                for exc in doc.raises() {
                    println!(
                        "  - {}: {}",
                        exc.exception_type.value, exc.description.value
                    );
                }
            }

            if let Some(notes) = doc.notes() {
                println!("\nNotes:");
                println!("  {}", notes.value);
            }

            if let Some(examples) = doc.examples() {
                println!("\nExamples:");
                println!("{}", examples.value);
            }

            if !result.diagnostics.is_empty() {
                println!("\nDiagnostics:");
                for d in &result.diagnostics {
                    println!("  {}", d);
                }
            }
    }
}
