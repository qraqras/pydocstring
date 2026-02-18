//! Integration tests for Sphinx-style docstring parser.

use pydocstring::parser::sphinx::parse_sphinx;

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_sphinx(docstring).unwrap();

    assert_eq!(result.summary.value, "This is a brief summary.");
}

// More tests will be added as Sphinx-style parser is implemented
