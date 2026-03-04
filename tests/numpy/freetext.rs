use super::*;

// =============================================================================
// Notes section
// =============================================================================

#[test]
fn test_with_notes_section() {
    let docstring = r#"Function with notes.

Notes
-----
This is an important note about the function.
"#;
    let result = parse_numpy(docstring);

    assert!(notes(&result).is_some());
    assert!(
        notes(&result)
            .unwrap()
            .source_text(&result.source)
            .contains("important note")
    );
}

/// `Note` alias for Notes.
#[test]
fn test_note_alias() {
    let docstring = "Summary.\n\nNote\n----\nThis is a note.\n";
    let result = parse_numpy(docstring);
    assert!(notes(&result).is_some());
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Note"
    );
    assert_eq!(sections(&result)[0].header.kind, NumPySectionKind::Notes);
}

/// Notes with multi-paragraph content.
#[test]
fn test_notes_multi_paragraph() {
    let docstring = "Summary.\n\nNotes\n-----\nFirst paragraph.\n\nSecond paragraph.\n";
    let result = parse_numpy(docstring);
    let n = notes(&result).unwrap().source_text(&result.source);
    assert!(n.contains("First paragraph."));
    assert!(n.contains("Second paragraph."));
}

// =============================================================================
// Warnings section (free-text)
// =============================================================================

#[test]
fn test_warnings_section() {
    let docstring = "Summary.\n\nWarnings\n--------\nThis function is deprecated.\n";
    let result = parse_numpy(docstring);
    assert_eq!(
        warnings_text(&result).unwrap().source_text(&result.source),
        "This function is deprecated."
    );
}

/// `Warning` alias for Warnings.
#[test]
fn test_warning_alias() {
    let docstring = "Summary.\n\nWarning\n-------\nBe careful.\n";
    let result = parse_numpy(docstring);
    assert!(warnings_text(&result).is_some());
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Warning"
    );
    assert_eq!(sections(&result)[0].header.kind, NumPySectionKind::Warnings);
}

/// Warnings section body variant check.
#[test]
fn test_warnings_section_body_variant() {
    let docstring = "Summary.\n\nWarnings\n--------\nDo not use.\n";
    let result = parse_numpy(docstring);
    match &sections(&result)[0].body {
        NumPySectionBody::Warnings(text) => {
            assert!(text.is_some());
        }
        other => panic!("Expected Warnings section body, got {:?}", other),
    }
}

// =============================================================================
// Examples section
// =============================================================================

#[test]
fn test_examples_basic() {
    let docstring = "Summary.\n\nExamples\n--------\n>>> func(1)\n1\n";
    let result = parse_numpy(docstring);
    let ex = examples(&result).unwrap().source_text(&result.source);
    assert!(ex.contains(">>> func(1)"));
    assert!(ex.contains("1"));
}

/// `Example` alias for Examples.
#[test]
fn test_example_alias() {
    let docstring = "Summary.\n\nExample\n-------\n>>> 1 + 1\n2\n";
    let result = parse_numpy(docstring);
    assert!(examples(&result).is_some());
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Example"
    );
    assert_eq!(sections(&result)[0].header.kind, NumPySectionKind::Examples);
}

/// Examples with narrative text and doctest.
#[test]
fn test_examples_with_narrative() {
    let docstring = "Summary.\n\nExamples\n--------\nHere is an example:\n\n>>> func(2)\n4\n";
    let result = parse_numpy(docstring);
    let ex = examples(&result).unwrap().source_text(&result.source);
    assert!(ex.contains("Here is an example:"));
    assert!(ex.contains(">>> func(2)"));
}

/// Examples section body variant check.
#[test]
fn test_examples_section_body_variant() {
    let docstring = "Summary.\n\nExamples\n--------\n>>> pass\n";
    let result = parse_numpy(docstring);
    match &sections(&result)[0].body {
        NumPySectionBody::Examples(text) => {
            assert!(text.is_some());
        }
        other => panic!("Expected Examples section body, got {:?}", other),
    }
}

// =============================================================================
// See Also section
// =============================================================================

#[test]
fn test_see_also_parsing() {
    let docstring = r#"Summary.

See Also
--------
func_a : Does something.
func_b, func_c
"#;
    let result = parse_numpy(docstring);
    let items = see_also(&result);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].names[0].source_text(&result.source), "func_a");
    assert_eq!(
        items[0]
            .description
            .as_ref()
            .map(|d| d.source_text(&result.source)),
        Some("Does something.")
    );
    assert_eq!(items[1].names.len(), 2);
    assert_eq!(items[1].names[0].source_text(&result.source), "func_b");
    assert_eq!(items[1].names[1].source_text(&result.source), "func_c");
}

/// See Also with no space before colon.
#[test]
fn test_see_also_no_space_before_colon() {
    let docstring = "Summary.\n\nSee Also\n--------\nfunc_a: Description of func_a.\n";
    let result = parse_numpy(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 1);
    assert_eq!(sa[0].names[0].source_text(&result.source), "func_a");
    assert!(
        sa[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source)
            .contains("Description")
    );
}

/// See Also with multiple items with descriptions.
#[test]
fn test_see_also_multiple_with_descriptions() {
    let docstring =
        "Summary.\n\nSee Also\n--------\nfunc_a : First function.\nfunc_b : Second function.\n";
    let result = parse_numpy(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 2);
    assert_eq!(sa[0].names[0].source_text(&result.source), "func_a");
    assert_eq!(
        sa[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "First function."
    );
    assert_eq!(sa[1].names[0].source_text(&result.source), "func_b");
}

/// See Also section body variant check.
#[test]
fn test_see_also_section_body_variant() {
    let docstring = "Summary.\n\nSee Also\n--------\nfunc_a : Desc.\n";
    let result = parse_numpy(docstring);
    match &sections(&result)[0].body {
        NumPySectionBody::SeeAlso(items) => {
            assert_eq!(items.len(), 1);
        }
        other => panic!("Expected SeeAlso section body, got {:?}", other),
    }
}

// =============================================================================
// References section
// =============================================================================

#[test]
fn test_references_parsing() {
    let docstring = r#"Summary.

References
----------
.. [1] Author A, "Title A", 2020.
.. [2] Author B, "Title B", 2021.
"#;
    let result = parse_numpy(docstring);
    let refs = references(&result);
    assert_eq!(refs.len(), 2);
    assert_eq!(
        refs[0].number.as_ref().unwrap().source_text(&result.source),
        "1"
    );
    assert!(
        refs[0]
            .content
            .as_ref()
            .unwrap()
            .source_text(&result.source)
            .contains("Author A")
    );
    assert_eq!(
        refs[1].number.as_ref().unwrap().source_text(&result.source),
        "2"
    );
    assert!(
        refs[1]
            .content
            .as_ref()
            .unwrap()
            .source_text(&result.source)
            .contains("Author B")
    );
}

/// References with directive markers.
#[test]
fn test_references_directive_markers() {
    let docstring = "Summary.\n\nReferences\n----------\n.. [1] Some ref.\n";
    let result = parse_numpy(docstring);
    let refs = references(&result);
    assert_eq!(refs.len(), 1);
    assert!(refs[0].directive_marker.is_some());
    assert_eq!(
        refs[0]
            .directive_marker
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        ".."
    );
    assert!(refs[0].open_bracket.is_some());
    assert!(refs[0].close_bracket.is_some());
}
