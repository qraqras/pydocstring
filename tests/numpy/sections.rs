use super::*;

// =============================================================================
// Case insensitive sections
// =============================================================================

#[test]
fn test_case_insensitive_sections() {
    let docstring = r#"Brief summary.

parameters
----------
x : int
    First param.

returns
-------
int
    The result.

NOTES
-----
Some notes here.
"#;
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    let names: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(returns(&result).len(), 1);
    assert!(notes(&result).is_some());
    assert_eq!(
        all_sections(&result)[0]
            .header()
            .name()
            .text(result.source()),
        "parameters"
    );
    assert_eq!(
        all_sections(&result)[2]
            .header()
            .name()
            .text(result.source()),
        "NOTES"
    );
}

// =============================================================================
// Section header spans
// =============================================================================

#[test]
fn test_section_header_spans() {
    let docstring = r#"Summary.

Parameters
----------
x : int
    Desc.
"#;
    let result = parse_numpy(docstring);
    let hdr = all_sections(&result)[0].header();
    assert_eq!(hdr.name().text(result.source()), "Parameters");
    assert_eq!(hdr.underline().text(result.source()), "----------");
}

// =============================================================================
// Span round-trip
// =============================================================================

#[test]
fn test_span_source_text_round_trip() {
    let docstring = r#"Summary line.

Parameters
----------
x : int
    Description of x.
"#;
    let result = parse_numpy(docstring);
    let src = result.source();

    assert_eq!(doc(&result).summary().unwrap().text(src), "Summary line.");
    assert_eq!(
        all_sections(&result)[0].header().name().text(src),
        "Parameters"
    );
    let underline = all_sections(&result)[0]
        .header()
        .underline()
        .text(result.source());
    assert!(underline.chars().all(|c| c == '-'));

    let p = &parameters(&result)[0];
    let names: Vec<_> = p.names().collect();
    assert_eq!(names[0].text(src), "x");
    assert_eq!(p.r#type().unwrap().text(src), "int");
    assert_eq!(p.description().unwrap().text(src), "Description of x.");
}

// =============================================================================
// Deprecation
// =============================================================================

#[test]
fn test_deprecation_directive() {
    let docstring = r#"Summary.

.. deprecated:: 1.6.0
    Use `new_func` instead.

Parameters
----------
x : int
    Desc.
"#;
    let result = parse_numpy(docstring);
    let dep = doc(&result)
        .deprecation()
        .expect("deprecation should be parsed");
    assert_eq!(dep.version().text(result.source()), "1.6.0");
    assert_eq!(
        dep.description().unwrap().text(result.source()),
        "Use `new_func` instead."
    );
    assert_eq!(dep.version().text(result.source()), "1.6.0");
}

// =============================================================================
// Section ordering
// =============================================================================

#[test]
fn test_section_order_preserved() {
    let docstring = r#"Summary.

Parameters
----------
x : int
    Desc.

Returns
-------
int
    Result.

Raises
------
ValueError
    Bad input.

Notes
-----
Some notes.
"#;
    let result = parse_numpy(docstring);
    let s = all_sections(&result);
    assert_eq!(s.len(), 4);
    assert_eq!(
        s[0].section_kind(result.source()),
        NumPySectionKind::Parameters
    );
    assert_eq!(
        s[1].section_kind(result.source()),
        NumPySectionKind::Returns
    );
    assert_eq!(s[2].section_kind(result.source()), NumPySectionKind::Raises);
    assert_eq!(s[3].section_kind(result.source()), NumPySectionKind::Notes);
}

#[test]
fn test_all_section_kinds_exist() {
    // Verify ALL is correct and contains no Unknown
    assert_eq!(NumPySectionKind::ALL.len(), 14);
    for kind in NumPySectionKind::ALL {
        assert_ne!(*kind, NumPySectionKind::Unknown);
    }
}

#[test]
fn test_section_kind_from_name_unknown() {
    assert_eq!(
        NumPySectionKind::from_name("nonexistent"),
        NumPySectionKind::Unknown
    );
    assert!(!NumPySectionKind::is_known("nonexistent"));
    assert!(NumPySectionKind::is_known("parameters"));
}

#[test]
fn test_stray_lines() {
    let docstring =
        "Summary.\n\nThis line is not a section.\n\nParameters\n----------\nx : int\n    Desc.\n";
    let result = parse_numpy(docstring);
    // The non-section line might be treated as extended summary or stray line
    // depending on parser behavior. Just verify parameters are still parsed.
    assert_eq!(parameters(&result).len(), 1);
}

// =============================================================================
// Display impl
// =============================================================================

#[test]
fn test_section_kind_display() {
    assert_eq!(format!("{}", NumPySectionKind::Parameters), "Parameters");
    assert_eq!(format!("{}", NumPySectionKind::Returns), "Returns");
    assert_eq!(format!("{}", NumPySectionKind::Yields), "Yields");
    assert_eq!(format!("{}", NumPySectionKind::Receives), "Receives");
    assert_eq!(
        format!("{}", NumPySectionKind::OtherParameters),
        "Other Parameters"
    );
    assert_eq!(format!("{}", NumPySectionKind::Raises), "Raises");
    assert_eq!(format!("{}", NumPySectionKind::Warns), "Warns");
    assert_eq!(format!("{}", NumPySectionKind::Warnings), "Warnings");
    assert_eq!(format!("{}", NumPySectionKind::SeeAlso), "See Also");
    assert_eq!(format!("{}", NumPySectionKind::Notes), "Notes");
    assert_eq!(format!("{}", NumPySectionKind::References), "References");
    assert_eq!(format!("{}", NumPySectionKind::Examples), "Examples");
    assert_eq!(format!("{}", NumPySectionKind::Attributes), "Attributes");
    assert_eq!(format!("{}", NumPySectionKind::Methods), "Methods");
    assert_eq!(format!("{}", NumPySectionKind::Unknown), "Unknown");
}

#[test]
fn test_docstring_display() {
    let docstring = "My summary.";
    let result = parse_numpy(docstring);
    // The root node covers the full source text
    assert_eq!(
        doc(&result).syntax().range().source_text(result.source()),
        "My summary."
    );
}
