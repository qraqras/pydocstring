use super::*;

// =============================================================================
// Raises section
// =============================================================================

#[test]
fn test_with_raises() {
    let docstring = r#"Function that may raise exceptions.

Raises
------
ValueError
    If the input is invalid.
TypeError
    If the type is wrong.
"#;
    let result = parse_numpy(docstring);

    assert_eq!(raises(&result).len(), 2);
    assert_eq!(raises(&result)[0].r#type().text(result.source()), "ValueError");
    assert_eq!(raises(&result)[1].r#type().text(result.source()), "TypeError");
}

#[test]
fn test_raises_with_spans() {
    let docstring = r#"Summary.

Raises
------
ValueError
    If input is bad.
TypeError
    If type is wrong.
"#;
    let result = parse_numpy(docstring);
    assert_eq!(raises(&result).len(), 2);
    assert_eq!(raises(&result)[0].r#type().text(result.source()), "ValueError");
    assert_eq!(raises(&result)[1].r#type().text(result.source()), "TypeError");
}

// =============================================================================
// Raises — colon splitting
// =============================================================================

/// Raises with colon separating type and description on the same line.
#[test]
fn test_raises_colon_split() {
    let docstring =
        "Summary.\n\nRaises\n------\nValueError : If the input is invalid.\nTypeError : If the type is wrong.";
    let result = parse_numpy(docstring);
    let exc = raises(&result);
    assert_eq!(exc.len(), 2);
    assert_eq!(exc[0].r#type().text(result.source()), "ValueError");
    assert!(exc[0].colon().is_some());
    assert_eq!(
        exc[0].description().unwrap().text(result.source()),
        "If the input is invalid."
    );
    assert_eq!(exc[1].r#type().text(result.source()), "TypeError");
    assert!(exc[1].colon().is_some());
    assert_eq!(
        exc[1].description().unwrap().text(result.source()),
        "If the type is wrong."
    );
}

/// Raises without colon (bare type, description on next line).
#[test]
fn test_raises_no_colon() {
    let docstring = "Summary.\n\nRaises\n------\nValueError\n    If the input is invalid.";
    let result = parse_numpy(docstring);
    let exc = raises(&result);
    assert_eq!(exc.len(), 1);
    assert_eq!(exc[0].r#type().text(result.source()), "ValueError");
    assert!(exc[0].colon().is_none());
    assert_eq!(
        exc[0].description().unwrap().text(result.source()),
        "If the input is invalid."
    );
}

/// Raises with colon and continuation description on next lines.
#[test]
fn test_raises_colon_with_continuation() {
    let docstring = "Summary.\n\nRaises\n------\nValueError : If bad.\n    More detail here.";
    let result = parse_numpy(docstring);
    let exc = raises(&result);
    assert_eq!(exc.len(), 1);
    assert_eq!(exc[0].r#type().text(result.source()), "ValueError");
    assert!(exc[0].colon().is_some());
    let desc = exc[0].description().unwrap().text(result.source());
    assert!(desc.contains("If bad."), "desc = {:?}", desc);
    assert!(desc.contains("More detail here."), "desc = {:?}", desc);
}

/// `Raise` alias for Raises.
#[test]
fn test_raise_alias() {
    let docstring = "Summary.\n\nRaise\n-----\nValueError\n    Bad input.\n";
    let result = parse_numpy(docstring);
    let exc = raises(&result);
    assert_eq!(exc.len(), 1);
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Raise");
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Raises
    );
}

// =============================================================================
// Warns section
// =============================================================================

#[test]
fn test_warns_basic() {
    let docstring = "Summary.\n\nWarns\n-----\nDeprecationWarning\n    If the old API is used.\n";
    let result = parse_numpy(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].r#type().text(result.source()), "DeprecationWarning");
    assert_eq!(
        w[0].description().unwrap().text(result.source()),
        "If the old API is used."
    );
}

#[test]
fn test_warns_multiple() {
    let docstring = "Summary.\n\nWarns\n-----\nDeprecationWarning\n    Old API.\nUserWarning\n    Bad usage.\n";
    let result = parse_numpy(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].r#type().text(result.source()), "DeprecationWarning");
    assert_eq!(w[1].r#type().text(result.source()), "UserWarning");
}

/// Warns with colon separating type and description on the same line.
#[test]
fn test_warns_colon_split() {
    let docstring = "Summary.\n\nWarns\n-----\nUserWarning : If input is unusual.\n";
    let result = parse_numpy(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].r#type().text(result.source()), "UserWarning");
    assert!(w[0].colon().is_some());
    assert_eq!(
        w[0].description().unwrap().text(result.source()),
        "If input is unusual."
    );
}

/// `Warn` alias for Warns.
#[test]
fn test_warn_alias() {
    let docstring = "Summary.\n\nWarn\n----\nUserWarning\n    Bad usage.\n";
    let result = parse_numpy(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 1);
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Warn");
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Warns
    );
}

/// Warns section body variant check.
#[test]
fn test_warns_section_body_variant() {
    let docstring = "Summary.\n\nWarns\n-----\nUserWarning\n    Bad.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Warns);
    let items: Vec<_> = s.warnings().collect();
    assert_eq!(items.len(), 1);
}
