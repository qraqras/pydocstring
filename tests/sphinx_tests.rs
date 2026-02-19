//! Integration tests for Sphinx-style docstring parser.
//!
//! Sphinx style is not supported in v1. These tests verify that the parser
//! returns a diagnostic and extracts a best-effort summary.

use pydocstring::sphinx::parse_sphinx;
use pydocstring::Severity;

#[test]
fn test_sphinx_unsupported_diagnostic() {
    let docstring = "This is a brief summary.";
    let result = parse_sphinx(docstring);

    assert_eq!(result.value.summary.value, "This is a brief summary.");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert!(result.diagnostics[0].message.contains("not supported"));
}

#[test]
fn test_sphinx_with_fields_still_unsupported() {
    let docstring = "Summary.\n\n:param x: The value.\n:type x: int\n:returns: Result.";
    let result = parse_sphinx(docstring);

    // Summary is extracted as best-effort
    assert_eq!(result.value.summary.value, "Summary.");
    // But fields are not parsed
    assert!(result.value.parameters.is_empty());
    assert!(result.value.returns.is_none());
    // Diagnostic indicates unsupported
    assert_eq!(result.diagnostics.len(), 1);
}
