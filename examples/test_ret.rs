//! Example: Parsing a Returns-only Google-style docstring
//!
//! Demonstrates how pydocstring handles a docstring that starts
//! directly with a section header (no summary line).

use pydocstring::google::parse_google;

fn main() {
    let docstring = "\
Returns:
  A dict mapping keys to the corresponding table row data
  fetched. Each row is represented as a tuple of strings. For
  example:

  {b'Serak': ('Rigel VII', 'Preparer'),
        b'Zim': ('Irk', 'Invader'),
        b'Lrrr': ('Omicron Persei 8', 'Emperor')}

  Returned keys are always bytes.  If a key from the keys argument is
  missing from the dictionary, then that row was not found in the
  table (and require_all_keys must have been False).
";

    let parsed = parse_google(docstring);

    // Display: raw source text
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     Returns-only Google Docstring Example        ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("── Display (raw text) ─────────────────────────────");
    println!("{}", parsed.source());

    // pretty_print: structured AST
    println!("── pretty_print (parsed AST) ──────────────────────");
    print!("{}", parsed.pretty_print());
}
