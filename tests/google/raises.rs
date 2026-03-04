use super::*;

// =============================================================================
// Raises section
// =============================================================================

#[test]
fn test_raises_single() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the input is invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        r[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "If the input is invalid."
    );
}

#[test]
fn test_raises_multiple() {
    let docstring =
        "Summary.\n\nRaises:\n    ValueError: If invalid.\n    TypeError: If wrong type.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(r[1].r#type.source_text(&result.source), "TypeError");
}

#[test]
fn test_raises_multiline_description() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the\n        input is invalid.";
    let result = parse_google(docstring);
    assert_eq!(
        raises(&result)[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "If the\n        input is invalid."
    );
}

#[test]
fn test_raises_exception_type_span() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring);
    assert_eq!(
        raises(&result)[0].r#type.source_text(&result.source),
        "ValueError"
    );
}

/// Raises entry with no space after colon.
#[test]
fn test_raises_no_space_after_colon() {
    let docstring = "Summary.\n\nRaises:\n    ValueError:If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        r[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "If invalid."
    );
}

/// Raises entry with extra spaces after colon.
#[test]
fn test_raises_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nRaises:\n    ValueError:   If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        r[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "If invalid."
    );
}

#[test]
fn test_raise_alias() {
    let docstring = "Summary.\n\nRaise:\n    ValueError: If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Raise"
    );
}

#[test]
fn test_docstring_like_raises() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
}

// =============================================================================
// Warns section
// =============================================================================

#[test]
fn test_warns_basic() {
    let docstring = "Summary.\n\nWarns:\n    DeprecationWarning: If using old API.";
    let result = parse_google(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 1);
    assert_eq!(
        w[0].warning_type.source_text(&result.source),
        "DeprecationWarning"
    );
    assert_eq!(
        w[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "If using old API."
    );
}

#[test]
fn test_warns_multiple() {
    let docstring =
        "Summary.\n\nWarns:\n    DeprecationWarning: Old API.\n    UserWarning: Bad config.";
    let result = parse_google(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 2);
    assert_eq!(
        w[0].warning_type.source_text(&result.source),
        "DeprecationWarning"
    );
    assert_eq!(w[1].warning_type.source_text(&result.source), "UserWarning");
}

#[test]
fn test_warn_alias() {
    let docstring = "Summary.\n\nWarn:\n    FutureWarning: Will change.";
    let result = parse_google(docstring);
    assert_eq!(warns(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Warn"
    );
}

#[test]
fn test_warns_multiline_description() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: First line.\n        Second line.";
    let result = parse_google(docstring);
    assert_eq!(
        warns(&result)[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "First line.\n        Second line."
    );
}

#[test]
fn test_warns_section_body_variant() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: Desc.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Warns(warns) => {
            assert_eq!(warns.len(), 1);
        }
        _ => panic!("Expected Warns section body"),
    }
}

// =============================================================================
// Warning alias (free-text, not Warns)
// =============================================================================

#[test]
fn test_warning_singular_alias() {
    let docstring = "Summary.\n\nWarning:\n    This is deprecated.";
    let result = parse_google(docstring);
    assert_eq!(
        warnings(&result).unwrap().source_text(&result.source),
        "This is deprecated."
    );
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Warning"
    );
}
