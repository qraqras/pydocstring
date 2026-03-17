use super::*;

// =============================================================================
// Returns section
// =============================================================================

#[test]
fn test_parse_named_returns() {
    let docstring = r#"Compute values.

Returns
-------
x : int
    The first value.
y : float
    The second value.
"#;
    let result = parse_numpy(docstring);
    assert_eq!(returns(&result).len(), 2);
    assert_eq!(returns(&result)[0].name().map(|n| n.text(result.source())), Some("x"));
    assert_eq!(
        returns(&result)[0].return_type().map(|t| t.text(result.source())),
        Some("int")
    );
    assert_eq!(
        returns(&result)[0].description().unwrap().text(result.source()),
        "The first value."
    );
    assert_eq!(returns(&result)[1].name().map(|n| n.text(result.source())), Some("y"));
}

/// Returns with no spaces around colon (named): `result:int`
#[test]
fn test_returns_no_spaces_around_colon() {
    let docstring = "Summary.\n\nReturns\n-------\nresult:int\n    The result.\n";
    let result = parse_numpy(docstring);
    let r = returns(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].name().unwrap().text(result.source()), "result");
    assert_eq!(r[0].return_type().unwrap().text(result.source()), "int");
}

/// Returns with type only (no name).
#[test]
fn test_returns_type_only() {
    let docstring = "Summary.\n\nReturns\n-------\nint\n    The result.\n";
    let result = parse_numpy(docstring);
    let r = returns(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].return_type().unwrap().text(result.source()), "int");
    assert_eq!(r[0].description().unwrap().text(result.source()), "The result.");
}

/// Returns — `Return` alias.
#[test]
fn test_return_alias() {
    let docstring = "Summary.\n\nReturn\n------\nint\n    The value.\n";
    let result = parse_numpy(docstring);
    let r = returns(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Return");
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Returns
    );
}

/// Returns with multiline description.
#[test]
fn test_returns_multiline_description() {
    let docstring = "Summary.\n\nReturns\n-------\nresult : int\n    First line.\n\n    Second paragraph.\n";
    let result = parse_numpy(docstring);
    let r = returns(&result);
    assert_eq!(r.len(), 1);
    let desc = r[0].description().unwrap().text(result.source());
    assert!(desc.contains("First line."));
    assert!(desc.contains("Second paragraph."));
}

// =============================================================================
// Yields section
// =============================================================================

#[test]
fn test_yields_basic() {
    let docstring = "Summary.\n\nYields\n------\nint\n    The next value.\n";
    let result = parse_numpy(docstring);
    let y = yields(&result);
    assert_eq!(y.len(), 1);
    assert_eq!(y[0].return_type().unwrap().text(result.source()), "int");
    assert_eq!(y[0].description().unwrap().text(result.source()), "The next value.");
}

#[test]
fn test_yields_named() {
    let docstring = "Summary.\n\nYields\n------\nvalue : str\n    The generated string.\n";
    let result = parse_numpy(docstring);
    let y = yields(&result);
    assert_eq!(y.len(), 1);
    assert_eq!(y[0].name().unwrap().text(result.source()), "value");
    assert_eq!(y[0].return_type().unwrap().text(result.source()), "str");
}

#[test]
fn test_yields_multiple() {
    let docstring = "Summary.\n\nYields\n------\nindex : int\n    The index.\nvalue : str\n    The value.\n";
    let result = parse_numpy(docstring);
    let y = yields(&result);
    assert_eq!(y.len(), 2);
    assert_eq!(y[0].name().unwrap().text(result.source()), "index");
    assert_eq!(y[1].name().unwrap().text(result.source()), "value");
}

/// Yields — `Yield` alias.
#[test]
fn test_yield_alias() {
    let docstring = "Summary.\n\nYield\n-----\nint\n    Next integer.\n";
    let result = parse_numpy(docstring);
    let y = yields(&result);
    assert_eq!(y.len(), 1);
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Yield");
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Yields
    );
}

/// Yields section body variant check.
#[test]
fn test_yields_section_body_variant() {
    let docstring = "Summary.\n\nYields\n------\nint\n    Value.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Yields);
    let items: Vec<_> = s.returns().collect();
    assert_eq!(items.len(), 1);
}
