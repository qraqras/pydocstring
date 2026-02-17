//! Example: Parsing NumPy-style docstrings

use pydocstring::parser::numpy::parse_numpy;

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

    match parse_numpy(docstring) {
        Ok(doc) => {
            if let Some(sig) = &doc.signature {
                println!("Signature: {}", sig);
            }
            println!("Summary: {}", doc.summary);
            println!("\nExtended Summary: {}", doc.extended_summary.unwrap_or_default());

            println!("\nParameters:");
            for param in &doc.parameters {
                println!("  - {:?}: {:?}", param.names, param.param_type);
                println!("    {}", param.description);
            }

            if !doc.returns.is_empty() {
                println!("\nReturns:");
                for ret in &doc.returns {
                    println!("  Type: {:?}", ret.return_type);
                    println!("  {}", ret.description);
                }
            }

            if !doc.raises.is_empty() {
                println!("\nRaises:");
                for exc in &doc.raises {
                    println!("  - {}: {}", exc.exception_type, exc.description);
                }
            }

            if let Some(notes) = &doc.notes {
                println!("\nNotes:");
                println!("  {}", notes);
            }

            if let Some(examples) = &doc.examples {
                println!("\nExamples:");
                println!("{}", examples);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse docstring: {}", e);
        }
    }
}
