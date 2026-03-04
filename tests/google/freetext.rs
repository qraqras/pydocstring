use super::*;

// =============================================================================
// Notes section
// =============================================================================

#[test]
fn test_note_section() {
    let docstring = "Summary.\n\nNote:\n    This is a note.";
    let result = parse_google(docstring);
    assert_eq!(
        notes(&result).unwrap().source_text(&result.source),
        "This is a note."
    );
}

#[test]
fn test_notes_alias() {
    let docstring = "Summary.\n\nNotes:\n    This is also a note.";
    let result = parse_google(docstring);
    assert_eq!(
        notes(&result).unwrap().source_text(&result.source),
        "This is also a note."
    );
}

// =============================================================================
// Examples section
// =============================================================================

#[test]
fn test_example_section() {
    let docstring = "Summary.\n\nExample:\n    >>> func(1)\n    1";
    let result = parse_google(docstring);
    assert_eq!(
        examples(&result).unwrap().source_text(&result.source),
        ">>> func(1)\n    1"
    );
}

#[test]
fn test_examples_alias() {
    let docstring = "Summary.\n\nExamples:\n    >>> 1 + 1\n    2";
    let result = parse_google(docstring);
    assert!(examples(&result).is_some());
}

// =============================================================================
// References section
// =============================================================================

#[test]
fn test_references_section() {
    let docstring = "Summary.\n\nReferences:\n    Author, Title, 2024.";
    let result = parse_google(docstring);
    assert!(references(&result).is_some());
}

// =============================================================================
// Warnings section (free-text)
// =============================================================================

#[test]
fn test_warnings_section() {
    let docstring = "Summary.\n\nWarnings:\n    This function is deprecated.";
    let result = parse_google(docstring);
    assert_eq!(
        warnings(&result).unwrap().source_text(&result.source),
        "This function is deprecated."
    );
}

// =============================================================================
// Todo section
// =============================================================================

#[test]
fn test_todo_freetext() {
    let docstring = "Summary.\n\nTodo:\n    * Item one.\n    * Item two.";
    let result = parse_google(docstring);
    let t = todo(&result).unwrap();
    assert!(t.source_text(&result.source).contains("Item one."));
    assert!(t.source_text(&result.source).contains("Item two."));
}

#[test]
fn test_todo_without_bullets() {
    let docstring = "Summary.\n\nTodo:\n    Implement feature X.\n    Fix bug Y.";
    let result = parse_google(docstring);
    let t = todo(&result).unwrap();
    assert!(
        t.source_text(&result.source)
            .contains("Implement feature X.")
    );
    assert!(t.source_text(&result.source).contains("Fix bug Y."));
}

#[test]
fn test_todo_multiline() {
    let docstring =
        "Summary.\n\nTodo:\n    * Item one that\n        continues here.\n    * Item two.";
    let result = parse_google(docstring);
    let t = todo(&result).unwrap();
    assert!(t.source_text(&result.source).contains("Item one that"));
    assert!(t.source_text(&result.source).contains("continues here."));
    assert!(t.source_text(&result.source).contains("Item two."));
}

// =============================================================================
// Admonition sections
// =============================================================================

#[test]
fn test_attention_section() {
    let docstring = "Summary.\n\nAttention:\n    This requires careful handling.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Attention(text) => {
            assert_eq!(
                text.source_text(&result.source),
                "This requires careful handling."
            );
        }
        _ => panic!("Expected Attention section body"),
    }
}

#[test]
fn test_caution_section() {
    let docstring = "Summary.\n\nCaution:\n    Use with care.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Caution(text) => {
            assert_eq!(text.source_text(&result.source), "Use with care.");
        }
        _ => panic!("Expected Caution section body"),
    }
}

#[test]
fn test_danger_section() {
    let docstring = "Summary.\n\nDanger:\n    May cause data loss.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Danger(text) => {
            assert_eq!(text.source_text(&result.source), "May cause data loss.");
        }
        _ => panic!("Expected Danger section body"),
    }
}

#[test]
fn test_error_section() {
    let docstring = "Summary.\n\nError:\n    Known issue with large inputs.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Error(text) => {
            assert_eq!(
                text.source_text(&result.source),
                "Known issue with large inputs."
            );
        }
        _ => panic!("Expected Error section body"),
    }
}

#[test]
fn test_hint_section() {
    let docstring = "Summary.\n\nHint:\n    Try using a smaller batch size.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Hint(text) => {
            assert_eq!(
                text.source_text(&result.source),
                "Try using a smaller batch size."
            );
        }
        _ => panic!("Expected Hint section body"),
    }
}

#[test]
fn test_important_section() {
    let docstring = "Summary.\n\nImportant:\n    Must be called before init().";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Important(text) => {
            assert_eq!(
                text.source_text(&result.source),
                "Must be called before init()."
            );
        }
        _ => panic!("Expected Important section body"),
    }
}

#[test]
fn test_tip_section() {
    let docstring = "Summary.\n\nTip:\n    Use vectorized operations for speed.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::Tip(text) => {
            assert_eq!(
                text.source_text(&result.source),
                "Use vectorized operations for speed."
            );
        }
        _ => panic!("Expected Tip section body"),
    }
}
