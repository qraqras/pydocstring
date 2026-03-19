use super::*;

// =============================================================================
// Attributes section
// =============================================================================

#[test]
fn test_attributes_basic() {
    let docstring = "Summary.\n\nAttributes\n----------\nname : str\n    The name.\nage : int\n    The age.\n";
    let result = parse_numpy(docstring);
    let a = attributes(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name().text(result.source()), "name");
    assert_eq!(a[0].r#type().unwrap().text(result.source()), "str");
    assert_eq!(a[0].description().unwrap().text(result.source()), "The name.");
    assert_eq!(a[1].name().text(result.source()), "age");
    assert_eq!(a[1].r#type().unwrap().text(result.source()), "int");
}

#[test]
fn test_attributes_no_type() {
    let docstring = "Summary.\n\nAttributes\n----------\nname\n    The name.\n";
    let result = parse_numpy(docstring);
    let a = attributes(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name().text(result.source()), "name");
    assert!(a[0].r#type().is_none());
    assert_eq!(a[0].description().unwrap().text(result.source()), "The name.");
}

#[test]
fn test_attributes_with_colon() {
    let docstring = "Summary.\n\nAttributes\n----------\nname : str\n    The name.\n";
    let result = parse_numpy(docstring);
    let a = attributes(&result);
    assert_eq!(a.len(), 1);
    assert!(a[0].colon().is_some());
    assert_eq!(a[0].colon().unwrap().text(result.source()), ":");
}

/// Attributes section body variant check.
#[test]
fn test_attributes_section_body_variant() {
    let docstring = "Summary.\n\nAttributes\n----------\nx : int\n    Value.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Attributes);
    let attrs: Vec<_> = s.attributes().collect();
    assert_eq!(attrs.len(), 1);
}

/// Attributes section kind check.
#[test]
fn test_attributes_section_kind() {
    let docstring = "Summary.\n\nAttributes\n----------\nx : int\n    Value.\n";
    let result = parse_numpy(docstring);
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Attributes
    );
}

// =============================================================================
// Methods section
// =============================================================================

#[test]
fn test_methods_basic() {
    let docstring =
        "Summary.\n\nMethods\n-------\nreset()\n    Reset the state.\nupdate(data)\n    Update with new data.\n";
    let result = parse_numpy(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].name().text(result.source()), "reset()");
    assert_eq!(m[0].description().unwrap().text(result.source()), "Reset the state.");
    assert_eq!(m[1].name().text(result.source()), "update(data)");
    assert_eq!(
        m[1].description().unwrap().text(result.source()),
        "Update with new data."
    );
}

#[test]
fn test_methods_with_colon() {
    let docstring = "Summary.\n\nMethods\n-------\nreset() : Reset the state.\n";
    let result = parse_numpy(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].name().text(result.source()), "reset()");
    assert!(m[0].colon().is_some());
    // Description may be inline or on next line depending on parser
    if let Some(desc) = m[0].description() {
        assert!(desc.text(result.source()).contains("Reset"));
    }
}

#[test]
fn test_methods_without_parens() {
    let docstring = "Summary.\n\nMethods\n-------\ndo_stuff\n    Performs the operation.\n";
    let result = parse_numpy(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].name().text(result.source()), "do_stuff");
    assert_eq!(
        m[0].description().unwrap().text(result.source()),
        "Performs the operation."
    );
}

/// Methods section body variant check.
#[test]
fn test_methods_section_body_variant() {
    let docstring = "Summary.\n\nMethods\n-------\nfoo()\n    Does bar.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Methods);
    let m: Vec<_> = s.methods().collect();
    assert_eq!(m.len(), 1);
}

/// Methods section kind check.
#[test]
fn test_methods_section_kind() {
    let docstring = "Summary.\n\nMethods\n-------\nfoo()\n    Does bar.\n";
    let result = parse_numpy(docstring);
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Methods
    );
}

// =============================================================================
// Unknown section
// =============================================================================

#[test]
fn test_unknown_section() {
    let docstring = "Summary.\n\nCustomSection\n-------------\nSome custom content.\n";
    let result = parse_numpy(docstring);
    let s = all_sections(&result);
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].section_kind(result.source()), NumPySectionKind::Unknown);
    assert_eq!(s[0].header().name().text(result.source()), "CustomSection");
}

#[test]
fn test_unknown_section_body_variant() {
    let docstring = "Summary.\n\nCustomSection\n-------------\nSome content.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Unknown);
    let text = s.body_text();
    assert!(text.is_some());
    assert!(text.unwrap().text(result.source()).contains("Some content."));
}

#[test]
fn test_unknown_section_with_known_sections() {
    let docstring = "Summary.\n\nParameters\n----------\nx : int\n    Value.\n\nCustom\n------\nExtra info.\n";
    let result = parse_numpy(docstring);
    let s = all_sections(&result);
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].section_kind(result.source()), NumPySectionKind::Parameters);
    assert_eq!(s[1].section_kind(result.source()), NumPySectionKind::Unknown);
}
