use super::*;

// =============================================================================
// Indented docstrings
// =============================================================================

#[test]
fn test_indented_docstring() {
    let docstring = "    Summary.\n\n    Args:\n        x (int): Value.";
    let result = parse_google(docstring);
    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary."
    );
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name().text(result.source()), "x");
    assert_eq!(a[0].r#type().unwrap().text(result.source()), "int");
}

#[test]
fn test_indented_summary_span() {
    let docstring = "    Summary.";
    let result = parse_google(docstring);
    let s = doc(&result).summary().unwrap();
    assert_eq!(s.range().start(), TextSize::new(4));
    assert_eq!(s.range().end(), TextSize::new(12));
    assert_eq!(s.text(result.source()), "Summary.");
}

// =============================================================================
// Space-before-colon and colonless header tests
// =============================================================================

/// `Args :` (space before colon) should be dispatched as Args, not Unknown.
#[test]
fn test_section_header_space_before_colon() {
    let input = "Summary.\n\nArgs :\n    x (int): The value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 1, "expected 1 arg from 'Args :'");
    assert_eq!(a[0].name().text(result.source()), "x");

    assert_eq!(
        all_sections(&result)[0]
            .header()
            .name()
            .text(result.source()),
        "Args"
    );
    assert!(all_sections(&result)[0].header().colon().is_some());
}

/// `Returns :` with space before colon.
#[test]
fn test_returns_space_before_colon() {
    let input = "Summary.\n\nReturns :\n    int: The result.";
    let result = parse_google(input);
    let r = returns(&result).unwrap();
    assert_eq!(r.return_type().unwrap().text(result.source()), "int");
}

/// Colonless `Args` should be parsed as Args section.
#[test]
fn test_section_header_no_colon() {
    let input = "Summary.\n\nArgs\n    x (int): The value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 1, "expected 1 arg from colonless 'Args'");
    assert_eq!(a[0].name().text(result.source()), "x");

    assert_eq!(
        all_sections(&result)[0]
            .header()
            .name()
            .text(result.source()),
        "Args"
    );
    assert!(all_sections(&result)[0].header().colon().is_none());
}

/// Colonless `Returns` should be parsed as Returns section.
#[test]
fn test_returns_no_colon() {
    let input = "Summary.\n\nReturns\n    int: The result.";
    let result = parse_google(input);
    let r = returns(&result).unwrap();
    assert_eq!(r.return_type().unwrap().text(result.source()), "int");
}

/// Colonless `Raises` should be parsed as Raises section.
#[test]
fn test_raises_no_colon() {
    let input = "Summary.\n\nRaises\n    ValueError: If invalid.";
    let result = parse_google(input);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type().text(result.source()), "ValueError");
}

/// Unknown names without colon should NOT be treated as headers.
#[test]
fn test_unknown_name_without_colon_not_header() {
    let input = "Summary.\n\nSomeWord\n    x (int): value.";
    let result = parse_google(input);
    assert!(
        all_sections(&result).is_empty(),
        "unknown colonless name should not become a section"
    );
}

/// Multiple sections with mixed colon styles.
#[test]
fn test_mixed_colon_styles() {
    let input = "Summary.\n\nArgs:\n    x: value.\n\nReturns\n    int: result.\n\nRaises :\n    ValueError: If bad.";
    let result = parse_google(input);
    assert_eq!(args(&result).len(), 1);
    assert!(returns(&result).is_some());
    assert_eq!(raises(&result).len(), 1);
}

// =============================================================================
// Tab indentation tests
// =============================================================================

/// Args section with tab-indented entries.
#[test]
fn test_tab_indented_args() {
    let input = "Summary.\n\nArgs:\n\tx: The value.\n\ty: Another value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name().text(result.source()), "x");
    assert_eq!(
        a[0].description().unwrap().text(result.source()),
        "The value."
    );
    assert_eq!(a[1].name().text(result.source()), "y");
    assert_eq!(
        a[1].description().unwrap().text(result.source()),
        "Another value."
    );
}

/// Args entries with tab indent and descriptions with deeper tab+space indent.
#[test]
fn test_tab_args_with_continuation() {
    let input = "Summary.\n\nArgs:\n\tx: First line.\n\t    Continuation.\n\ty: Second.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name().text(result.source()), "x");
    let desc = a[0].description().unwrap().text(result.source());
    assert!(desc.contains("First line."), "desc = {:?}", desc);
    assert!(desc.contains("Continuation."), "desc = {:?}", desc);
}

/// Returns section with tab-indented entry.
#[test]
fn test_tab_indented_returns() {
    let input = "Summary.\n\nReturns:\n\tint: The result.";
    let result = parse_google(input);
    let r = returns(&result);
    assert!(r.is_some());
    let r = r.unwrap();
    assert_eq!(r.return_type().unwrap().text(result.source()), "int");
    assert_eq!(
        r.description().unwrap().text(result.source()),
        "The result."
    );
}

/// Raises section with tab-indented entries.
#[test]
fn test_tab_indented_raises() {
    let input = "Summary.\n\nRaises:\n\tValueError: If bad.\n\tTypeError: If wrong type.";
    let result = parse_google(input);
    let r = raises(&result);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].r#type().text(result.source()), "ValueError");
    assert_eq!(r[1].r#type().text(result.source()), "TypeError");
}

/// Section header detection with tab indentation matches.
#[test]
fn test_tab_indented_section_header() {
    let input = "\tSummary.\n\n\tArgs:\n\t\tx: The value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name().text(result.source()), "x");
}
