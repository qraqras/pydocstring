//! Integration tests for NumPy-style docstring parser.

use pydocstring::numpy::parse_numpy;
use pydocstring::{TextSize, LineIndex};

// =============================================================================
// Basic parsing
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "This is a brief summary.");
    assert!(result.extended_summary.is_none());
    assert!(result.parameters().is_empty());
}

#[test]
fn test_parse_simple_span() {
    let docstring = "Brief description.";
    let result = parse_numpy(docstring).value;
    assert_eq!(result.summary.value, "Brief description.");
    assert_eq!(result.summary.range.start(), TextSize::new(0));
    assert_eq!(result.summary.range.end(), TextSize::new(18));
    assert_eq!(
        result.summary.range.source_text(&result.source),
        "Brief description."
    );
}

#[test]
fn test_summary_with_description() {
    let docstring = r#"Brief summary.

This is a longer description that provides
more details about the function.
"#;
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Brief summary.");
    assert!(result.extended_summary.is_some());
}

#[test]
fn test_empty_docstring() {
    let result = parse_numpy("").value;
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_numpy("   \n\n   ").value;
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_docstring_span_covers_entire_input() {
    let docstring = "First line.\n\nSecond line.";
    let result = parse_numpy(docstring).value;
    assert_eq!(result.range.start(), TextSize::new(0));
    assert_eq!(result.range.end().raw() as usize, docstring.len());
}

// =============================================================================
// Signature
// =============================================================================

#[test]
fn test_parse_with_signature() {
    let docstring = r#"add(a, b)

The sum of two numbers.

Parameters
----------
a : int
    First number.
b : int
    Second number.
"#;
    let result = parse_numpy(docstring).value;
    assert_eq!(
        result.signature.as_ref().map(|s| s.value.as_str()),
        Some("add(a, b)")
    );
    assert_eq!(result.summary.value, "The sum of two numbers.");
    assert_eq!(result.parameters().len(), 2);
}

// =============================================================================
// Extended summary
// =============================================================================

#[test]
fn test_extended_summary_preserves_paragraphs() {
    let docstring = r#"Summary.

First paragraph of extended.

Second paragraph of extended.

Parameters
----------
x : int
    Desc.
"#;
    let result = parse_numpy(docstring).value;
    let ext = result.extended_summary.as_ref().unwrap();
    assert!(ext.value.contains("First paragraph"));
    assert!(ext.value.contains("Second paragraph"));
    assert!(ext.value.contains('\n'));
}

// =============================================================================
// Parameters
// =============================================================================

#[test]
fn test_with_parameters() {
    let docstring = r#"Calculate the sum of two numbers.

Parameters
----------
x : int
    The first number.
y : int
    The second number.

Returns
-------
int
    The sum of x and y.
"#;
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Calculate the sum of two numbers.");
    assert_eq!(result.parameters().len(), 2);

    assert_eq!(result.parameters()[0].names[0].value, "x");
    assert_eq!(
        result.parameters()[0]
            .param_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );
    assert_eq!(
        result.parameters()[0].description.value,
        "The first number."
    );

    assert_eq!(result.parameters()[1].names[0].value, "y");
    assert_eq!(
        result.parameters()[1]
            .param_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );

    assert!(!result.returns().is_empty());
    assert_eq!(
        result.returns()[0]
            .return_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );
}

#[test]
fn test_optional_parameters() {
    let docstring = r#"Function with optional parameters.

Parameters
----------
required : str
    A required parameter.
optional : int, optional
    An optional parameter.
"#;
    let result = parse_numpy(docstring).value;

    assert_eq!(result.parameters().len(), 2);
    assert!(result.parameters()[0].optional.is_none());
    assert!(result.parameters()[1].optional.is_some());
    assert_eq!(
        result.parameters()[1]
            .param_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );
}

#[test]
fn test_parse_with_parameters_spans() {
    let docstring = r#"Brief description.

Parameters
----------
x : int
    The first parameter.
y : str, optional
    The second parameter.
"#;
    let result = parse_numpy(docstring).value;
    assert_eq!(result.parameters().len(), 2);

    // Verify name spans point to correct source text
    assert_eq!(
        result.parameters()[0].names[0]
            .range
            .source_text(&result.source),
        "x"
    );
    assert_eq!(
        result.parameters()[1].names[0]
            .range
            .source_text(&result.source),
        "y"
    );
    // Verify type spans
    assert_eq!(
        result.parameters()[0]
            .param_type
            .as_ref()
            .unwrap()
            .range
            .source_text(&result.source),
        "int"
    );
}

#[test]
fn test_multiple_parameter_names() {
    let docstring = r#"Summary.

Parameters
----------
x1, x2 : array_like
    Input arrays.
"#;
    let result = parse_numpy(docstring).value;
    let p = &result.parameters()[0];
    assert_eq!(p.names.len(), 2);
    assert_eq!(p.names[0].value, "x1");
    assert_eq!(p.names[1].value, "x2");
    assert_eq!(p.names[0].range.source_text(&result.source), "x1");
    assert_eq!(p.names[1].range.source_text(&result.source), "x2");
}

#[test]
fn test_description_with_colon_not_treated_as_param() {
    let docstring = r#"Brief summary.

Parameters
----------
x : int
    A value like key: value should not split.
"#;
    let result = parse_numpy(docstring).value;
    assert_eq!(result.parameters().len(), 1);
    assert_eq!(result.parameters()[0].names[0].value, "x");
    assert!(result.parameters()[0]
        .description
        .value
        .contains("key: value"));
}

#[test]
fn test_multi_paragraph_description() {
    let docstring = r#"Summary.

Parameters
----------
x : int
    First paragraph of x.

    Second paragraph of x.
"#;
    let result = parse_numpy(docstring).value;
    let desc = &result.parameters()[0].description.value;
    assert!(desc.contains("First paragraph of x."));
    assert!(desc.contains("Second paragraph of x."));
    assert!(desc.contains('\n'));
}

// =============================================================================
// Returns
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
    let result = parse_numpy(docstring).value;
    assert_eq!(result.returns().len(), 2);
    assert_eq!(
        result.returns()[0].name.as_ref().map(|n| n.value.as_str()),
        Some("x")
    );
    assert_eq!(
        result.returns()[0]
            .return_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );
    assert_eq!(result.returns()[0].description.value, "The first value.");
    assert_eq!(
        result.returns()[1].name.as_ref().map(|n| n.value.as_str()),
        Some("y")
    );
}

// =============================================================================
// Raises
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
    let result = parse_numpy(docstring).value;

    assert_eq!(result.raises().len(), 2);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(result.raises()[1].exception_type.value, "TypeError");
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
    let result = parse_numpy(docstring).value;
    assert_eq!(result.raises().len(), 2);
    assert_eq!(
        result.raises()[0]
            .exception_type
            .range
            .source_text(&result.source),
        "ValueError"
    );
    assert_eq!(
        result.raises()[1]
            .exception_type
            .range
            .source_text(&result.source),
        "TypeError"
    );
}

// =============================================================================
// Notes / See Also / References / Examples
// =============================================================================

#[test]
fn test_with_notes_section() {
    let docstring = r#"Function with notes.

Notes
-----
This is an important note about the function.
"#;
    let result = parse_numpy(docstring).value;

    assert!(result.notes().is_some());
    assert!(result.notes().unwrap().value.contains("important note"));
}

#[test]
fn test_see_also_parsing() {
    let docstring = r#"Summary.

See Also
--------
func_a : Does something.
func_b, func_c
"#;
    let result = parse_numpy(docstring).value;
    let items = result.see_also();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].names[0].value, "func_a");
    assert_eq!(
        items[0].description.as_ref().map(|d| d.value.as_str()),
        Some("Does something.")
    );
    assert_eq!(items[1].names.len(), 2);
    assert_eq!(items[1].names[0].value, "func_b");
    assert_eq!(items[1].names[1].value, "func_c");
}

#[test]
fn test_references_parsing() {
    let docstring = r#"Summary.

References
----------
.. [1] Author A, "Title A", 2020.
.. [2] Author B, "Title B", 2021.
"#;
    let result = parse_numpy(docstring).value;
    let refs = result.references();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].number, 1);
    assert!(refs[0].content.value.contains("Author A"));
    assert_eq!(refs[1].number, 2);
    assert!(refs[1].content.value.contains("Author B"));
}

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
    let result = parse_numpy(docstring).value;
    assert_eq!(result.parameters().len(), 1);
    assert_eq!(result.parameters()[0].names[0].value, "x");
    assert_eq!(result.returns().len(), 1);
    assert!(result.notes().is_some());
    // Original text is preserved in header
    assert_eq!(result.sections[0].header.name.value, "parameters");
    assert_eq!(result.sections[2].header.name.value, "NOTES");
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
    let result = parse_numpy(docstring).value;
    let hdr = &result.sections[0].header;
    assert_eq!(hdr.name.range.source_text(&result.source), "Parameters");
    assert_eq!(hdr.underline.source_text(&result.source), "----------");
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
    let result = parse_numpy(docstring).value;
    let src = &result.source;

    assert_eq!(result.summary.range.source_text(src), "Summary line.");
    assert_eq!(
        result.sections[0].header.name.range.source_text(src),
        "Parameters"
    );
    let underline = result.sections[0].header.underline.source_text(src);
    assert!(underline.chars().all(|c| c == '-'));

    let p = &result.parameters()[0];
    assert_eq!(p.names[0].range.source_text(src), "x");
    assert_eq!(p.param_type.as_ref().unwrap().range.source_text(src), "int");
    assert_eq!(p.description.range.source_text(src), "Description of x.");
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
    let result = parse_numpy(docstring).value;
    let dep = result
        .deprecation
        .as_ref()
        .expect("deprecation should be parsed");
    assert_eq!(dep.version.value, "1.6.0");
    assert_eq!(dep.description.value, "Use `new_func` instead.");
    assert_eq!(dep.version.range.source_text(&result.source), "1.6.0");
}

// =============================================================================
// Indented docstrings (class/method bodies)
// =============================================================================

#[test]
fn test_indented_docstring() {
    let docstring = "    Summary line.\n\n    Parameters\n    ----------\n    x : int\n        Description of x.\n    y : str, optional\n        Description of y.\n\n    Returns\n    -------\n    bool\n        The result.\n";
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Summary line.");
    assert_eq!(result.parameters().len(), 2);
    assert_eq!(result.parameters()[0].names[0].value, "x");
    assert_eq!(
        result.parameters()[0]
            .param_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("int")
    );
    assert_eq!(result.parameters()[1].names[0].value, "y");
    assert!(result.parameters()[1].optional.is_some());
    assert_eq!(result.returns().len(), 1);
    assert_eq!(
        result.returns()[0]
            .return_type
            .as_ref()
            .map(|t| t.value.as_str()),
        Some("bool")
    );

    // Spans point to correct positions in indented source
    assert_eq!(
        result.summary.range.source_text(&result.source),
        "Summary line."
    );
    assert_eq!(
        result.parameters()[0].names[0]
            .range
            .source_text(&result.source),
        "x"
    );
    assert_eq!(
        result.parameters()[0]
            .param_type
            .as_ref()
            .unwrap()
            .range
            .source_text(&result.source),
        "int"
    );
}

#[test]
fn test_deeply_indented_docstring() {
    let docstring = "        Brief.\n\n        Parameters\n        ----------\n        a : float\n            The value.\n\n        Raises\n        ------\n        ValueError\n            If bad.\n";
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Brief.");
    assert_eq!(result.parameters().len(), 1);
    assert_eq!(result.parameters()[0].names[0].value, "a");
    assert_eq!(result.raises().len(), 1);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(
        result.raises()[0]
            .exception_type
            .range
            .source_text(&result.source),
        "ValueError"
    );
}

#[test]
fn test_indented_with_deprecation() {
    let docstring = "    Summary.\n\n    .. deprecated:: 2.0.0\n        Use new_func instead.\n\n    Parameters\n    ----------\n    x : int\n        Desc.\n";
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Summary.");
    let dep = result
        .deprecation
        .as_ref()
        .expect("should have deprecation");
    assert_eq!(dep.version.value, "2.0.0");
    assert!(dep.description.value.contains("new_func"));
    assert_eq!(result.parameters().len(), 1);
    assert_eq!(result.parameters()[0].names[0].value, "x");
}

#[test]
fn test_mixed_indent_first_line() {
    let docstring =
        "Summary.\n\n    Parameters\n    ----------\n    x : int\n        Description.\n";
    let result = parse_numpy(docstring).value;

    assert_eq!(result.summary.value, "Summary.");
    assert_eq!(result.parameters().len(), 1);
    assert_eq!(result.parameters()[0].names[0].value, "x");
    assert_eq!(result.parameters()[0].description.value, "Description.");
}
