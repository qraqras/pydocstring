use super::*;

// =============================================================================
// Multiple sections
// =============================================================================

#[test]
fn test_all_sections() {
    let docstring = r#"Calculate the sum.

This function adds two numbers.

Args:
    a (int): The first number.
    b (int): The second number.

Returns:
    int: The sum of a and b.

Raises:
    TypeError: If inputs are not numbers.

Example:
    >>> add(1, 2)
    3

Note:
    This is a simple function."#;

    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Calculate the sum."
    );
    assert!(doc(&result).extended_summary().is_some());
    assert_eq!(args(&result).len(), 2);
    assert!(returns(&result).is_some());
    assert_eq!(raises(&result).len(), 1);
    assert!(examples(&result).is_some());
    assert!(notes(&result).is_some());
}

#[test]
fn test_sections_with_blank_lines() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\n    y (str): Name.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring);
    assert_eq!(args(&result).len(), 2);
    assert!(returns(&result).is_some());
}

// =============================================================================
// Section order preservation
// =============================================================================

#[test]
fn test_section_order() {
    let docstring = "Summary.\n\nReturns:\n    int: Value.\n\nArgs:\n    x: Input.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header().name().text(result.source()), "Returns");
    assert_eq!(sections[1].header().name().text(result.source()), "Args");
}

// =============================================================================
// Section header / section spans
// =============================================================================

#[test]
fn test_section_header_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    let header = all_sections(&result)[0].header();
    assert_eq!(header.name().text(result.source()), "Args");
    assert_eq!(header.syntax().range().source_text(result.source()), "Args:");
}

#[test]
fn test_section_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    let section = &all_sections(&result)[0];
    assert_eq!(
        section.syntax().range().source_text(result.source()),
        "Args:\n    x: Value."
    );
}

// =============================================================================
// Unknown sections
// =============================================================================

#[test]
fn test_unknown_section_preserved() {
    let docstring = "Summary.\n\nCustom:\n    Some custom content.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header().name().text(result.source()), "Custom");
    assert_eq!(sections[0].section_kind(result.source()), GoogleSectionKind::Unknown);
    assert_eq!(
        sections[0].body_text().unwrap().text(result.source()),
        "Some custom content."
    );
}

#[test]
fn test_unknown_section_with_known() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.\n\nCustom:\n    Content.\n\nReturns:\n    int: Result.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].header().name().text(result.source()), "Args");
    assert_eq!(sections[1].header().name().text(result.source()), "Custom");
    assert_eq!(sections[2].header().name().text(result.source()), "Returns");
    assert_eq!(args(&result).len(), 1);
    assert!(returns(&result).is_some());
}

#[test]
fn test_multiple_unknown_sections() {
    let docstring = "Summary.\n\nCustom One:\n    First.\n\nCustom Two:\n    Second.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header().name().text(result.source()), "Custom One");
    assert_eq!(sections[1].header().name().text(result.source()), "Custom Two");
}

// =============================================================================
// Case-insensitive section headers
// =============================================================================

#[test]
fn test_napoleon_case_insensitive() {
    let docstring = "Summary.\n\nkeyword args:\n    x (int): Value.";
    let result = parse_google(docstring);
    assert_eq!(keyword_args(&result).len(), 1);
}

#[test]
fn test_see_also_case_insensitive() {
    let docstring = "Summary.\n\nsee also:\n    func_a: Description.";
    let result = parse_google(docstring);
    assert_eq!(see_also(&result).len(), 1);
}

// =============================================================================
// Full docstring with all Napoleon sections
// =============================================================================

#[test]
fn test_napoleon_full_docstring() {
    let docstring = r#"Calculate something.

Extended description.

Args:
    x (int): First argument.

Keyword Args:
    timeout (float): Timeout value.

Returns:
    int: The result.

Raises:
    ValueError: If x is negative.

Warns:
    DeprecationWarning: If old API is used.

See Also:
    other_func: Related function.

Note:
    Implementation detail.

Example:
    >>> calculate(1)
    1"#;

    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Calculate something."
    );
    assert!(doc(&result).extended_summary().is_some());
    assert_eq!(args(&result).len(), 1);
    assert_eq!(keyword_args(&result).len(), 1);
    assert!(returns(&result).is_some());
    assert_eq!(raises(&result).len(), 1);
    assert_eq!(warns(&result).len(), 1);
    assert_eq!(see_also(&result).len(), 1);
    assert!(notes(&result).is_some());
    assert!(examples(&result).is_some());
}

// =============================================================================
// Span round-trip
// =============================================================================

#[test]
fn test_span_source_text_round_trip() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring);

    assert_eq!(doc(&result).summary().unwrap().text(result.source()), "Summary.");
    assert_eq!(args(&result)[0].name().text(result.source()), "x");
    assert_eq!(args(&result)[0].r#type().unwrap().text(result.source()), "int");
    assert_eq!(
        returns(&result).unwrap().return_type().unwrap().text(result.source()),
        "bool"
    );
}
