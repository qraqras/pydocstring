use super::*;

// =============================================================================
// Returns section
// =============================================================================

#[test]
fn test_returns_with_type() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        r.description.as_ref().unwrap().source_text(&result.source),
        "The result."
    );
}

#[test]
fn test_returns_multiple_lines() {
    let docstring = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        r.description.as_ref().unwrap().source_text(&result.source),
        "The count.\n    str: The message."
    );
}

#[test]
fn test_returns_without_type() {
    let docstring = "Summary.\n\nReturns:\n    The computed result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert!(r.return_type.is_none());
    assert_eq!(
        r.description.as_ref().unwrap().source_text(&result.source),
        "The computed result."
    );
}

#[test]
fn test_returns_multiline_description() {
    let docstring = "Summary.\n\nReturns:\n    int: The result\n        of the computation.";
    let result = parse_google(docstring);
    assert_eq!(
        returns(&result)
            .unwrap()
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The result\n        of the computation."
    );
}

#[test]
fn test_return_alias() {
    let docstring = "Summary.\n\nReturn:\n    int: The value.";
    let result = parse_google(docstring);
    assert!(returns(&result).is_some());
}

/// Returns entry with no space after colon.
#[test]
fn test_returns_no_space_after_colon() {
    let docstring = "Summary.\n\nReturns:\n    int:The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        r.description.as_ref().unwrap().source_text(&result.source),
        "The result."
    );
}

/// Returns entry with extra spaces after colon.
#[test]
fn test_returns_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nReturns:\n    int:   The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        r.description.as_ref().unwrap().source_text(&result.source),
        "The result."
    );
}

#[test]
fn test_docstring_like_returns() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

// =============================================================================
// Yields section
// =============================================================================

#[test]
fn test_yields() {
    let docstring = "Summary.\n\nYields:\n    int: The next value.";
    let result = parse_google(docstring);
    let y = yields(&result).unwrap();
    assert_eq!(
        y.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        y.description.as_ref().unwrap().source_text(&result.source),
        "The next value."
    );
}

#[test]
fn test_yield_alias() {
    let docstring = "Summary.\n\nYield:\n    str: Next string.";
    let result = parse_google(docstring);
    assert!(yields(&result).is_some());
}
