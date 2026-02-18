//! Integration tests for Google-style docstring parser.

use pydocstring::google::parse_google;

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_google(docstring).unwrap();

    assert_eq!(result.summary.value, "This is a brief summary.");
}

// More tests will be added as Google-style parser is implemented
