use super::*;

// =============================================================================
// Summary / Extended Summary
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "This is a brief summary."
    );
}

#[test]
fn test_summary_span() {
    let docstring = "Brief description.";
    let result = parse_google(docstring);
    let s = doc(&result).summary().unwrap();
    assert_eq!(s.range().start(), TextSize::new(0));
    assert_eq!(s.range().end(), TextSize::new(18));
    assert_eq!(s.text(result.source()), "Brief description.");
}

#[test]
fn test_empty_docstring() {
    let result = parse_google("");
    assert!(doc(&result).summary().is_none());
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_google("   \n   \n");
    assert!(doc(&result).summary().is_none());
}

#[test]
fn test_summary_with_description() {
    let docstring = "Brief summary.\n\nExtended description that provides\nmore details about the function.";
    let result = parse_google(docstring);

    assert_eq!(doc(&result).summary().unwrap().text(result.source()), "Brief summary.");
    let desc = doc(&result).extended_summary().unwrap();
    assert_eq!(
        desc.text(result.source()),
        "Extended description that provides\nmore details about the function."
    );
}

#[test]
fn test_summary_with_multiline_description() {
    let docstring = r#"Brief summary.

First paragraph of description.

Second paragraph of description."#;
    let result = parse_google(docstring);
    assert_eq!(doc(&result).summary().unwrap().text(result.source()), "Brief summary.");
    let desc = doc(&result).extended_summary().unwrap();
    assert!(desc.text(result.source()).contains("First paragraph"));
    assert!(desc.text(result.source()).contains("Second paragraph"));
}

#[test]
fn test_multiline_summary() {
    let docstring = "This is a long summary\nthat spans two lines.\n\nExtended description.";
    let result = parse_google(docstring);
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
    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary line one\ncontinues here."
    );
    assert!(doc(&result).extended_summary().is_none());
}

#[test]
fn test_multiline_summary_then_section() {
    let docstring = "Summary line one\ncontinues here.\nArgs:\n    x (int): val";
    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary line one\ncontinues here."
    );
    assert!(doc(&result).extended_summary().is_none());
    assert_eq!(doc(&result).sections().count(), 1);
}

#[test]
fn test_section_only_no_summary() {
    let docstring = "Args:\n    x (int): Value.";
    let result = parse_google(docstring);
    assert_eq!(args(&result).len(), 1);
}

#[test]
fn test_leading_blank_lines() {
    let docstring = "\n\n\nSummary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    assert_eq!(doc(&result).summary().unwrap().text(result.source()), "Summary.");
    assert_eq!(args(&result).len(), 1);
}

#[test]
fn test_docstring_like_summary() {
    let docstring = "Summary.";
    let result = parse_google(docstring);
    assert_eq!(doc(&result).summary().unwrap().text(result.source()), "Summary.");
}
