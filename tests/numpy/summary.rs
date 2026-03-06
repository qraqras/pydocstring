use super::*;

// =============================================================================
// Basic parsing / Summary / Extended Summary
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "This is a brief summary."
    );
    assert!(doc(&result).extended_summary().is_none());
    assert!(parameters(&result).is_empty());
}

#[test]
fn test_parse_simple_span() {
    let docstring = "Brief description.";
    let result = parse_numpy(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Brief description."
    );
    assert_eq!(
        doc(&result).summary().unwrap().range().start(),
        TextSize::new(0)
    );
    assert_eq!(
        doc(&result).summary().unwrap().range().end(),
        TextSize::new(18)
    );
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Brief description."
    );
}

#[test]
fn test_summary_with_description() {
    let docstring = r#"Brief summary.

This is a longer description that provides
more details about the function.
"#;
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Brief summary."
    );
    assert!(doc(&result).extended_summary().is_some());
}

#[test]
fn test_multiline_summary() {
    let docstring = "This is a long summary\nthat spans two lines.\n\nExtended description.";
    let result = parse_numpy(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "This is a long summary\nthat spans two lines."
    );
    let desc = doc(&result).extended_summary().unwrap();
    assert_eq!(desc.text(result.source()), "Extended description.");
}

#[test]
fn test_multiline_summary_no_extended() {
    let docstring = "Summary line one\ncontinues here.";
    let result = parse_numpy(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary line one\ncontinues here."
    );
    assert!(doc(&result).extended_summary().is_none());
}

#[test]
fn test_empty_docstring() {
    let result = parse_numpy("");
    assert!(doc(&result).summary().is_none());
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_numpy("   \n\n   ");
    assert!(doc(&result).summary().is_none());
}

#[test]
fn test_docstring_span_covers_entire_input() {
    let docstring = "First line.\n\nSecond line.";
    let result = parse_numpy(docstring);
    assert_eq!(doc(&result).syntax().range().start(), TextSize::new(0));
    assert_eq!(
        doc(&result).syntax().range().end().raw() as usize,
        docstring.len()
    );
}

// =============================================================================
// Signature-like line is treated as summary
// =============================================================================

#[test]
fn test_parse_with_signature_line() {
    let docstring = r#"add(a, b)

The sum of two numbers.

Parameters
----------
a : int
    First number.
b : int
    Second number.
"#;
    let result = parse_numpy(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "add(a, b)"
    );
    assert_eq!(parameters(&result).len(), 2);
}

// =============================================================================
// Extended summary
// =============================================================================

#[test]
fn test_extended_summary_preserves_paragraphs() {
    let docstring = r#"Summary.

First paragraph of extended.

Second paragraph of extended.

Parameters
----------
x : int
    Desc.
"#;
    let result = parse_numpy(docstring);
    let ext = doc(&result).extended_summary().unwrap();
    assert!(ext.text(result.source()).contains("First paragraph"));
    assert!(ext.text(result.source()).contains("Second paragraph"));
    assert!(ext.text(result.source()).contains('\n'));
}
