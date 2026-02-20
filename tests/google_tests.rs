//! Integration tests for Google-style docstring parser.

use pydocstring::google::parse_google;
use pydocstring::{GoogleSectionBody, Severity};
use pydocstring::{LineIndex, TextSize};

// =============================================================================
// Basic parsing
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "This is a brief summary.");
}

#[test]
fn test_summary_span() {
    let docstring = "Brief description.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.range.start(), TextSize::new(0));
    assert_eq!(result.summary.range.end(), TextSize::new(18));
    assert_eq!(
        result.summary.range.source_text(&result.source),
        "Brief description."
    );
}

#[test]
fn test_empty_docstring() {
    let result = parse_google("").value;
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_google("   \n   \n").value;
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_summary_with_description() {
    let docstring =
        "Brief summary.\n\nExtended description that provides\nmore details about the function.";
    let result = parse_google(docstring).value;

    assert_eq!(result.summary.value, "Brief summary.");
    let desc = result.description.as_ref().unwrap();
    assert_eq!(
        desc.value,
        "Extended description that provides\nmore details about the function."
    );
}

#[test]
fn test_summary_with_multiline_description() {
    let docstring = r#"Brief summary.

First paragraph of description.

Second paragraph of description."#;
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "Brief summary.");
    let desc = result.description.as_ref().unwrap();
    assert!(desc.value.contains("First paragraph"));
    assert!(desc.value.contains("Second paragraph"));
}

// =============================================================================
// Args section
// =============================================================================

#[test]
fn test_args_basic() {
    let docstring = "Summary.\n\nArgs:\n    x (int): The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[0].description.value, "The value.");
}

#[test]
fn test_args_multiple() {
    let docstring = "Summary.\n\nArgs:\n    x (int): First.\n    y (str): Second.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[1].name.value, "y");
    assert_eq!(result.args()[1].arg_type.as_ref().unwrap().value, "str");
}

#[test]
fn test_args_no_type() {
    let docstring = "Summary.\n\nArgs:\n    x: The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args()[0].name.value, "x");
    assert!(result.args()[0].arg_type.is_none());
    assert_eq!(result.args()[0].description.value, "The value.");
}

#[test]
fn test_args_optional() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert!(result.args()[0].optional.is_some());
}

#[test]
fn test_args_complex_type() {
    let docstring = "Summary.\n\nArgs:\n    data (Dict[str, List[int]]): The data.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "Dict[str, List[int]]"
    );
}

#[test]
fn test_args_tuple_type() {
    let docstring = "Summary.\n\nArgs:\n    pair (Tuple[int, str]): A pair of values.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "Tuple[int, str]"
    );
}

#[test]
fn test_args_multiline_description() {
    let docstring =
        "Summary.\n\nArgs:\n    x (int): First line.\n        Second line.\n        Third line.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.args()[0].description.value,
        "First line.\nSecond line.\nThird line."
    );
}

#[test]
fn test_args_description_on_next_line() {
    let docstring = "Summary.\n\nArgs:\n    x (int):\n        The description.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[0].description.value, "The description.");
}

#[test]
fn test_args_varargs() {
    let docstring = "Summary.\n\nArgs:\n    *args: Positional args.\n    **kwargs: Keyword args.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.args()[0].name.value, "*args");
    assert_eq!(result.args()[0].description.value, "Positional args.");
    assert_eq!(result.args()[1].name.value, "**kwargs");
    assert_eq!(result.args()[1].description.value, "Keyword args.");
}

#[test]
fn test_args_kwargs_with_type() {
    let docstring = "Summary.\n\nArgs:\n    **kwargs (dict): Keyword arguments.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args()[0].name.value, "**kwargs");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "dict");
}

#[test]
fn test_arguments_alias() {
    let docstring = "Summary.\n\nArguments:\n    x (int): The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
}

// =============================================================================
// Args span accuracy
// =============================================================================

#[test]
fn test_args_name_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring).value;
    let arg = &result.args()[0];
    let index = LineIndex::from_source(&result.source);
    let (line, col) = index.line_col(arg.name.range.start());
    assert_eq!(line, 3);
    assert_eq!(col, 4);
    assert_eq!(
        arg.name.range.end(),
        TextSize::new(arg.name.range.start().raw() + 1)
    );
    assert_eq!(arg.name.range.source_text(&result.source), "x");
}

#[test]
fn test_args_type_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring).value;
    let arg = &result.args()[0];
    let type_span = arg.arg_type.as_ref().unwrap();
    let index = LineIndex::from_source(&result.source);
    let (line, _col) = index.line_col(type_span.range.start());
    assert_eq!(line, 3);
    assert_eq!(type_span.range.source_text(&result.source), "int");
}

#[test]
fn test_args_optional_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): Value.";
    let result = parse_google(docstring).value;
    let opt_span = result.args()[0].optional.as_ref().unwrap();
    assert_eq!(opt_span.range.source_text(&result.source), "optional");
}

// =============================================================================
// Returns section
// =============================================================================

#[test]
fn test_returns_with_type() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring).value;
    assert_eq!(result.returns().len(), 1);
    assert_eq!(
        result.returns()[0].return_type.as_ref().unwrap().value,
        "int"
    );
    assert_eq!(result.returns()[0].description.value, "The result.");
}

#[test]
fn test_returns_multiple() {
    let docstring = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
    let result = parse_google(docstring).value;
    assert_eq!(result.returns().len(), 2);
    assert_eq!(
        result.returns()[0].return_type.as_ref().unwrap().value,
        "int"
    );
    assert_eq!(
        result.returns()[1].return_type.as_ref().unwrap().value,
        "str"
    );
}

#[test]
fn test_returns_without_type() {
    let docstring = "Summary.\n\nReturns:\n    The computed result.";
    let result = parse_google(docstring).value;
    assert_eq!(result.returns().len(), 1);
    assert!(result.returns()[0].return_type.is_none());
    assert_eq!(
        result.returns()[0].description.value,
        "The computed result."
    );
}

#[test]
fn test_returns_multiline_description() {
    let docstring = "Summary.\n\nReturns:\n    int: The result\n        of the computation.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.returns()[0].description.value,
        "The result\nof the computation."
    );
}

#[test]
fn test_return_alias() {
    let docstring = "Summary.\n\nReturn:\n    int: The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.returns().len(), 1);
}

// =============================================================================
// Yields section
// =============================================================================

#[test]
fn test_yields() {
    let docstring = "Summary.\n\nYields:\n    int: The next value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.yields().len(), 1);
    assert_eq!(
        result.yields()[0].return_type.as_ref().unwrap().value,
        "int"
    );
    assert_eq!(result.yields()[0].description.value, "The next value.");
}

#[test]
fn test_yield_alias() {
    let docstring = "Summary.\n\nYield:\n    str: Next string.";
    let result = parse_google(docstring).value;
    assert_eq!(result.yields().len(), 1);
}

// =============================================================================
// Raises section
// =============================================================================

#[test]
fn test_raises_single() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the input is invalid.";
    let result = parse_google(docstring).value;
    assert_eq!(result.raises().len(), 1);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(
        result.raises()[0].description.value,
        "If the input is invalid."
    );
}

#[test]
fn test_raises_multiple() {
    let docstring =
        "Summary.\n\nRaises:\n    ValueError: If invalid.\n    TypeError: If wrong type.";
    let result = parse_google(docstring).value;
    assert_eq!(result.raises().len(), 2);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(result.raises()[1].exception_type.value, "TypeError");
}

#[test]
fn test_raises_multiline_description() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the\n        input is invalid.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.raises()[0].description.value,
        "If the\ninput is invalid."
    );
}

#[test]
fn test_raises_exception_type_span() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.raises()[0]
            .exception_type
            .range
            .source_text(&result.source),
        "ValueError"
    );
}

// =============================================================================
// Attributes section
// =============================================================================

#[test]
fn test_attributes() {
    let docstring = "Summary.\n\nAttributes:\n    name (str): The name.\n    age (int): The age.";
    let result = parse_google(docstring).value;
    assert_eq!(result.attributes().len(), 2);
    assert_eq!(result.attributes()[0].name.value, "name");
    assert_eq!(
        result.attributes()[0].attr_type.as_ref().unwrap().value,
        "str"
    );
    assert_eq!(result.attributes()[1].name.value, "age");
}

#[test]
fn test_attributes_no_type() {
    let docstring = "Summary.\n\nAttributes:\n    name: The name.";
    let result = parse_google(docstring).value;
    assert_eq!(result.attributes()[0].name.value, "name");
    assert!(result.attributes()[0].attr_type.is_none());
}

// =============================================================================
// Free-text sections
// =============================================================================

#[test]
fn test_note_section() {
    let docstring = "Summary.\n\nNote:\n    This is a note.";
    let result = parse_google(docstring).value;
    assert_eq!(result.notes().unwrap().value, "This is a note.");
}

#[test]
fn test_notes_alias() {
    let docstring = "Summary.\n\nNotes:\n    This is also a note.";
    let result = parse_google(docstring).value;
    assert_eq!(result.notes().unwrap().value, "This is also a note.");
}

#[test]
fn test_example_section() {
    let docstring = "Summary.\n\nExample:\n    >>> func(1)\n    1";
    let result = parse_google(docstring).value;
    assert_eq!(result.examples().unwrap().value, ">>> func(1)\n1");
}

#[test]
fn test_examples_alias() {
    let docstring = "Summary.\n\nExamples:\n    >>> 1 + 1\n    2";
    let result = parse_google(docstring).value;
    assert!(result.examples().is_some());
}

#[test]
fn test_references_section() {
    let docstring = "Summary.\n\nReferences:\n    Author, Title, 2024.";
    let result = parse_google(docstring).value;
    assert!(result.references().is_some());
}

#[test]
fn test_warnings_section() {
    let docstring = "Summary.\n\nWarnings:\n    This function is deprecated.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.warnings().unwrap().value,
        "This function is deprecated."
    );
}

// =============================================================================
// Todo section
// =============================================================================

#[test]
fn test_todo_freetext() {
    let docstring = "Summary.\n\nTodo:\n    * Item one.\n    * Item two.";
    let result = parse_google(docstring).value;
    let todo = result.todo().unwrap();
    assert!(todo.value.contains("Item one."));
    assert!(todo.value.contains("Item two."));
}

#[test]
fn test_todo_without_bullets() {
    let docstring = "Summary.\n\nTodo:\n    Implement feature X.\n    Fix bug Y.";
    let result = parse_google(docstring).value;
    let todo = result.todo().unwrap();
    assert!(todo.value.contains("Implement feature X."));
    assert!(todo.value.contains("Fix bug Y."));
}

#[test]
fn test_todo_multiline() {
    let docstring =
        "Summary.\n\nTodo:\n    * Item one that\n        continues here.\n    * Item two.";
    let result = parse_google(docstring).value;
    let todo = result.todo().unwrap();
    assert!(todo.value.contains("Item one that"));
    assert!(todo.value.contains("continues here."));
    assert!(todo.value.contains("Item two."));
}

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

    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "Calculate the sum.");
    assert!(result.description.is_some());
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.returns().len(), 1);
    assert_eq!(result.raises().len(), 1);
    assert!(result.examples().is_some());
    assert!(result.notes().is_some());
}

#[test]
fn test_sections_with_blank_lines() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\n    y (str): Name.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.returns().len(), 1);
}

// =============================================================================
// Section order preservation
// =============================================================================

#[test]
fn test_section_order() {
    let docstring = "Summary.\n\nReturns:\n    int: Value.\n\nArgs:\n    x: Input.";
    let result = parse_google(docstring).value;
    assert_eq!(result.sections.len(), 2);
    assert_eq!(result.sections[0].header.name.value, "Returns");
    assert_eq!(result.sections[1].header.name.value, "Args");
}

#[test]
fn test_section_header_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).value;
    let header = &result.sections[0].header;
    assert_eq!(header.name.value, "Args");
    assert_eq!(header.name.range.source_text(&result.source), "Args");
    assert_eq!(header.range.source_text(&result.source), "Args:");
}

#[test]
fn test_section_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).value;
    let section = &result.sections[0];
    assert_eq!(
        section.range.source_text(&result.source),
        "Args:\n    x: Value."
    );
}

// =============================================================================
// Unknown sections
// =============================================================================

#[test]
fn test_unknown_section_preserved() {
    let docstring = "Summary.\n\nCustom:\n    Some custom content.";
    let result = parse_google(docstring).value;
    assert_eq!(result.sections.len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Custom");
    match &result.sections[0].body {
        GoogleSectionBody::Unknown(text) => {
            assert_eq!(text.value, "Some custom content.");
        }
        _ => panic!("Expected Unknown section body"),
    }
}

#[test]
fn test_unknown_section_with_known() {
    let docstring =
        "Summary.\n\nArgs:\n    x: Value.\n\nCustom:\n    Content.\n\nReturns:\n    int: Result.";
    let result = parse_google(docstring).value;
    assert_eq!(result.sections.len(), 3);
    assert_eq!(result.sections[0].header.name.value, "Args");
    assert_eq!(result.sections[1].header.name.value, "Custom");
    assert_eq!(result.sections[2].header.name.value, "Returns");
    // Known sections still accessible via convenience methods
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.returns().len(), 1);
}

#[test]
fn test_multiple_unknown_sections() {
    let docstring = "Summary.\n\nCustom One:\n    First.\n\nCustom Two:\n    Second.";
    let result = parse_google(docstring).value;
    assert_eq!(result.sections.len(), 2);
    assert_eq!(result.sections[0].header.name.value, "Custom One");
    assert_eq!(result.sections[1].header.name.value, "Custom Two");
}

// =============================================================================
// Indented docstring (non-zero base indent)
// =============================================================================

#[test]
fn test_indented_docstring() {
    let docstring = "    Summary.\n\n    Args:\n        x (int): Value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "Summary.");
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
}

#[test]
fn test_indented_summary_span() {
    let docstring = "    Summary.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.range.start(), TextSize::new(4));
    assert_eq!(result.summary.range.end(), TextSize::new(12));
    assert_eq!(result.summary.range.source_text(&result.source), "Summary.");
}

// =============================================================================
// Convenience accessors
// =============================================================================

#[test]
fn test_docstring_like_summary() {
    let docstring = "Summary.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "Summary.");
}

#[test]
fn test_docstring_like_parameters() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n    y (str): Name.";
    let result = parse_google(docstring).value;
    let params = result.args();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name.value, "x");
    assert_eq!(params[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(params[1].name.value, "y");
}

#[test]
fn test_docstring_like_returns() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring).value;
    let returns = result.returns();
    assert_eq!(returns.len(), 1);
    assert_eq!(returns[0].return_type.as_ref().unwrap().value, "int");
}

#[test]
fn test_docstring_like_raises() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring).value;
    let raises = result.raises();
    assert_eq!(raises.len(), 1);
    assert_eq!(raises[0].exception_type.value, "ValueError");
}

// =============================================================================
// Span round-trip
// =============================================================================

#[test]
fn test_span_source_text_round_trip() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring).value;

    // Summary
    assert_eq!(result.summary.range.source_text(&result.source), "Summary.");

    // Arg name
    assert_eq!(result.args()[0].name.range.source_text(&result.source), "x");

    // Arg type
    assert_eq!(
        result.args()[0]
            .arg_type
            .as_ref()
            .unwrap()
            .range
            .source_text(&result.source),
        "int"
    );

    // Return type
    assert_eq!(
        result.returns()[0]
            .return_type
            .as_ref()
            .unwrap()
            .range
            .source_text(&result.source),
        "bool"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_section_only_no_summary() {
    let docstring = "Args:\n    x (int): Value.";
    let result = parse_google(docstring).value;
    // "Args:" at base_indent=0 is a section header, so summary remains empty
    assert_eq!(result.args().len(), 1);
}

#[test]
fn test_leading_blank_lines() {
    let docstring = "\n\n\nSummary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.summary.value, "Summary.");
    assert_eq!(result.args().len(), 1);
}

#[test]
fn test_optional_only_in_parens() {
    let docstring = "Summary.\n\nArgs:\n    x (optional): Value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args()[0].name.value, "x");
    assert!(result.args()[0].arg_type.is_none());
    assert!(result.args()[0].optional.is_some());
}

#[test]
fn test_complex_optional_type() {
    let docstring = "Summary.\n\nArgs:\n    x (List[int], optional): Values.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "List[int]"
    );
    assert!(result.args()[0].optional.is_some());
}

// =============================================================================
// Diagnostic generation tests
// =============================================================================

#[test]
fn test_no_diagnostics_for_valid_input() {
    let input = "Summary.\n\nArgs:\n    x (int): The value.\n\nReturns:\n    int: The result.";
    let result = parse_google(input);
    assert!(
        result.diagnostics.is_empty(),
        "expected no diagnostics for valid input, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_diag_missing_colon_in_args() {
    let input = "Summary.\n\nArgs:\n    x just a value";
    let result = parse_google(input);
    assert!(
        !result.diagnostics.is_empty(),
        "expected diagnostics for missing colon"
    );
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing ':'"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    assert!(warn.message.contains("'x'"));
    // The parser should still produce a partial arg entry
    assert_eq!(result.value.args().len(), 1);
}

#[test]
fn test_diag_unclosed_paren_in_args() {
    let input = "Summary.\n\nArgs:\n    x (int: The value.";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("unclosed parenthesis"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    // The parser should still produce a partial arg entry
    assert_eq!(result.value.args().len(), 1);
}

#[test]
fn test_diag_missing_description_in_args() {
    let input = "Summary.\n\nArgs:\n    x (int):";
    let result = parse_google(input);
    let hint = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing description"))
        .unwrap();
    assert_eq!(hint.severity, Severity::Hint);
    assert!(hint.message.contains("'x'"));
}

#[test]
fn test_diag_empty_section_body() {
    let input = "Summary.\n\nArgs:\n\nReturns:\n    int: The result.";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("empty section body"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    assert!(warn.message.contains("'Args'"));
}

#[test]
fn test_diag_empty_freetext_section() {
    let input = "Summary.\n\nNote:\n\nArgs:\n    x (int): The value.";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("empty section body"))
        .unwrap();
    assert!(warn.message.contains("'Note'"));
}

#[test]
fn test_diag_missing_colon_in_raises() {
    let input = "Summary.\n\nRaises:\n    ValueError";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing ':'"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    assert!(warn.message.contains("'ValueError'"));
    // Still parses the exception type
    assert_eq!(result.value.raises().len(), 1);
    assert_eq!(result.value.raises()[0].exception_type.value, "ValueError");
}

#[test]
fn test_diag_missing_description_in_raises() {
    let input = "Summary.\n\nRaises:\n    ValueError:";
    let result = parse_google(input);
    let hint = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing description"))
        .unwrap();
    assert_eq!(hint.severity, Severity::Hint);
    assert!(hint.message.contains("'ValueError'"));
}

#[test]
fn test_diag_missing_description_in_returns() {
    let input = "Summary.\n\nReturns:\n    int:";
    let result = parse_google(input);
    let hint = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing description"))
        .unwrap();
    assert_eq!(hint.severity, Severity::Hint);
    assert!(hint.message.contains("'int'"));
}

#[test]
fn test_diag_unclosed_paren_in_attributes() {
    let input = "Summary.\n\nAttributes:\n    x (Dict[str: The data.";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("unclosed parenthesis"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    assert_eq!(result.value.attributes().len(), 1);
}

#[test]
fn test_diag_missing_colon_in_attributes() {
    let input = "Summary.\n\nAttributes:\n    x some value";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing ':'"))
        .unwrap();
    assert_eq!(warn.severity, Severity::Warning);
    assert!(warn.message.contains("'x'"));
}

#[test]
fn test_diag_multiple_diagnostics() {
    let input = "Summary.\n\nArgs:\n    x just text\n    y (int:\n    z (str): ";
    let result = parse_google(input);
    // x: missing colon + missing description
    // y: unclosed paren (+ missing colon through fallback + missing description)
    // z: missing description
    assert!(
        result.diagnostics.len() >= 3,
        "expected at least 3 diagnostics, got: {:?}",
        result.diagnostics
    );
    // Partial AST still produced
    assert_eq!(result.value.args().len(), 3);
}

#[test]
fn test_diag_span_accuracy() {
    let input = "Summary.\n\nArgs:\n    x just text";
    let result = parse_google(input);
    let warn = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("missing ':'"))
        .unwrap();
    // "x just text" starts at line 3, column 4
    let index = LineIndex::from_source(input);
    let (line, col) = index.line_col(warn.range.start());
    assert_eq!(line, 3);
    assert_eq!(col, 4);
}

#[test]
fn test_diag_valid_input_no_false_positives() {
    let input = "\
Summary.

Args:
    x (int): The value.
    y (str, optional): The name.
    *args: Positional.
    **kwargs (dict): Keywords.

Returns:
    int: The result.

Raises:
    ValueError: If invalid.
    TypeError: If wrong type.

Attributes:
    name (str): The name.

Note:
    Some implementation notes.

Example:
    >>> do_something()
";
    let result = parse_google(input);
    assert!(
        result.diagnostics.is_empty(),
        "expected no diagnostics for valid input, got: {:?}",
        result.diagnostics
    );
}

// =============================================================================
// Napoleon: Parameters / Params aliases
// =============================================================================

#[test]
fn test_parameters_alias() {
    let docstring = "Summary.\n\nParameters:\n    x (int): The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.sections[0].header.name.value, "Parameters");
}

#[test]
fn test_params_alias() {
    let docstring = "Summary.\n\nParams:\n    x (int): The value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Params");
}

// =============================================================================
// Napoleon: Keyword Args section
// =============================================================================

#[test]
fn test_keyword_args_basic() {
    let docstring = "Summary.\n\nKeyword Args:\n    timeout (int): Timeout in seconds.\n    retries (int): Number of retries.";
    let result = parse_google(docstring).value;
    assert_eq!(result.keyword_args().len(), 2);
    assert_eq!(result.keyword_args()[0].name.value, "timeout");
    assert_eq!(
        result.keyword_args()[0].arg_type.as_ref().unwrap().value,
        "int"
    );
    assert_eq!(result.keyword_args()[1].name.value, "retries");
}

#[test]
fn test_keyword_arguments_alias() {
    let docstring = "Summary.\n\nKeyword Arguments:\n    key (str): The key.";
    let result = parse_google(docstring).value;
    assert_eq!(result.keyword_args().len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Keyword Arguments");
}

#[test]
fn test_keyword_args_section_body_variant() {
    let docstring = "Summary.\n\nKeyword Args:\n    k (str): Key.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::KeywordArgs(args) => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected KeywordArgs section body"),
    }
}

// =============================================================================
// Napoleon: Other Parameters section
// =============================================================================

#[test]
fn test_other_parameters() {
    let docstring = "Summary.\n\nOther Parameters:\n    debug (bool): Enable debug mode.\n    verbose (bool, optional): Verbose output.";
    let result = parse_google(docstring).value;
    assert_eq!(result.other_parameters().len(), 2);
    assert_eq!(result.other_parameters()[0].name.value, "debug");
    assert_eq!(result.other_parameters()[1].name.value, "verbose");
    assert!(result.other_parameters()[1].optional.is_some());
}

#[test]
fn test_other_parameters_section_body_variant() {
    let docstring = "Summary.\n\nOther Parameters:\n    x (int): Extra.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::OtherParameters(args) => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected OtherParameters section body"),
    }
}

// =============================================================================
// Napoleon: Receives section
// =============================================================================

#[test]
fn test_receives() {
    let docstring = "Summary.\n\nReceives:\n    data (bytes): The received data.";
    let result = parse_google(docstring).value;
    assert_eq!(result.receives().len(), 1);
    assert_eq!(result.receives()[0].name.value, "data");
    assert_eq!(
        result.receives()[0].arg_type.as_ref().unwrap().value,
        "bytes"
    );
}

#[test]
fn test_receive_alias() {
    let docstring = "Summary.\n\nReceive:\n    msg (str): The message.";
    let result = parse_google(docstring).value;
    assert_eq!(result.receives().len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Receive");
}

// =============================================================================
// Napoleon: Raise alias
// =============================================================================

#[test]
fn test_raise_alias() {
    let docstring = "Summary.\n\nRaise:\n    ValueError: If invalid.";
    let result = parse_google(docstring).value;
    assert_eq!(result.raises().len(), 1);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(result.sections[0].header.name.value, "Raise");
}

// =============================================================================
// Napoleon: Warns section
// =============================================================================

#[test]
fn test_warns_basic() {
    let docstring = "Summary.\n\nWarns:\n    DeprecationWarning: If using old API.";
    let result = parse_google(docstring).value;
    assert_eq!(result.warns().len(), 1);
    assert_eq!(result.warns()[0].warning_type.value, "DeprecationWarning");
    assert_eq!(result.warns()[0].description.value, "If using old API.");
}

#[test]
fn test_warns_multiple() {
    let docstring =
        "Summary.\n\nWarns:\n    DeprecationWarning: Old API.\n    UserWarning: Bad config.";
    let result = parse_google(docstring).value;
    assert_eq!(result.warns().len(), 2);
    assert_eq!(result.warns()[0].warning_type.value, "DeprecationWarning");
    assert_eq!(result.warns()[1].warning_type.value, "UserWarning");
}

#[test]
fn test_warn_alias() {
    let docstring = "Summary.\n\nWarn:\n    FutureWarning: Will change.";
    let result = parse_google(docstring).value;
    assert_eq!(result.warns().len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Warn");
}

#[test]
fn test_warns_multiline_description() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: First line.\n        Second line.";
    let result = parse_google(docstring).value;
    assert_eq!(
        result.warns()[0].description.value,
        "First line.\nSecond line."
    );
}

#[test]
fn test_warns_section_body_variant() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: Desc.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Warns(warns) => {
            assert_eq!(warns.len(), 1);
        }
        _ => panic!("Expected Warns section body"),
    }
}

// =============================================================================
// Napoleon: Warning alias
// =============================================================================

#[test]
fn test_warning_singular_alias() {
    let docstring = "Summary.\n\nWarning:\n    This is deprecated.";
    let result = parse_google(docstring).value;
    assert_eq!(result.warnings().unwrap().value, "This is deprecated.");
    assert_eq!(result.sections[0].header.name.value, "Warning");
}

// =============================================================================
// Napoleon: Attribute alias
// =============================================================================

#[test]
fn test_attribute_singular_alias() {
    let docstring = "Summary.\n\nAttribute:\n    name (str): The name.";
    let result = parse_google(docstring).value;
    assert_eq!(result.attributes().len(), 1);
    assert_eq!(result.sections[0].header.name.value, "Attribute");
}

// =============================================================================
// Napoleon: Methods section
// =============================================================================

#[test]
fn test_methods_basic() {
    let docstring = "Summary.\n\nMethods:\n    reset(): Reset the state.\n    update(data): Update with new data.";
    let result = parse_google(docstring).value;
    assert_eq!(result.methods().len(), 2);
    assert_eq!(result.methods()[0].name.value, "reset()");
    assert_eq!(result.methods()[0].description.value, "Reset the state.");
    assert_eq!(result.methods()[1].name.value, "update(data)");
}

#[test]
fn test_methods_without_parens() {
    let docstring = "Summary.\n\nMethods:\n    do_stuff: Performs the operation.";
    let result = parse_google(docstring).value;
    assert_eq!(result.methods().len(), 1);
    assert_eq!(result.methods()[0].name.value, "do_stuff");
    assert_eq!(
        result.methods()[0].description.value,
        "Performs the operation."
    );
}

#[test]
fn test_methods_section_body_variant() {
    let docstring = "Summary.\n\nMethods:\n    foo(): Does bar.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Methods(methods) => {
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected Methods section body"),
    }
}

// =============================================================================
// Napoleon: See Also section
// =============================================================================

#[test]
fn test_see_also_basic() {
    let docstring = "Summary.\n\nSee Also:\n    other_func: Does something else.";
    let result = parse_google(docstring).value;
    assert_eq!(result.see_also().len(), 1);
    assert_eq!(result.see_also()[0].names.len(), 1);
    assert_eq!(result.see_also()[0].names[0].value, "other_func");
    assert_eq!(
        result.see_also()[0].description.as_ref().unwrap().value,
        "Does something else."
    );
}

#[test]
fn test_see_also_multiple_names() {
    let docstring = "Summary.\n\nSee Also:\n    func_a, func_b, func_c";
    let result = parse_google(docstring).value;
    assert_eq!(result.see_also().len(), 1);
    assert_eq!(result.see_also()[0].names.len(), 3);
    assert_eq!(result.see_also()[0].names[0].value, "func_a");
    assert_eq!(result.see_also()[0].names[1].value, "func_b");
    assert_eq!(result.see_also()[0].names[2].value, "func_c");
    assert!(result.see_also()[0].description.is_none());
}

#[test]
fn test_see_also_mixed() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Description.\n    func_b, func_c";
    let result = parse_google(docstring).value;
    assert_eq!(result.see_also().len(), 2);
    assert_eq!(result.see_also()[0].names[0].value, "func_a");
    assert!(result.see_also()[0].description.is_some());
    assert_eq!(result.see_also()[1].names.len(), 2);
    assert!(result.see_also()[1].description.is_none());
}

#[test]
fn test_see_also_section_body_variant() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Desc.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::SeeAlso(items) => {
            assert_eq!(items.len(), 1);
        }
        _ => panic!("Expected SeeAlso section body"),
    }
}

// =============================================================================
// Napoleon: Admonition sections
// =============================================================================

#[test]
fn test_attention_section() {
    let docstring = "Summary.\n\nAttention:\n    This requires careful handling.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Attention(text) => {
            assert_eq!(text.value, "This requires careful handling.");
        }
        _ => panic!("Expected Attention section body"),
    }
}

#[test]
fn test_caution_section() {
    let docstring = "Summary.\n\nCaution:\n    Use with care.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Caution(text) => {
            assert_eq!(text.value, "Use with care.");
        }
        _ => panic!("Expected Caution section body"),
    }
}

#[test]
fn test_danger_section() {
    let docstring = "Summary.\n\nDanger:\n    May cause data loss.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Danger(text) => {
            assert_eq!(text.value, "May cause data loss.");
        }
        _ => panic!("Expected Danger section body"),
    }
}

#[test]
fn test_error_section() {
    let docstring = "Summary.\n\nError:\n    Known issue with large inputs.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Error(text) => {
            assert_eq!(text.value, "Known issue with large inputs.");
        }
        _ => panic!("Expected Error section body"),
    }
}

#[test]
fn test_hint_section() {
    let docstring = "Summary.\n\nHint:\n    Try using a smaller batch size.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Hint(text) => {
            assert_eq!(text.value, "Try using a smaller batch size.");
        }
        _ => panic!("Expected Hint section body"),
    }
}

#[test]
fn test_important_section() {
    let docstring = "Summary.\n\nImportant:\n    Must be called before init().";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Important(text) => {
            assert_eq!(text.value, "Must be called before init().");
        }
        _ => panic!("Expected Important section body"),
    }
}

#[test]
fn test_tip_section() {
    let docstring = "Summary.\n\nTip:\n    Use vectorized operations for speed.";
    let result = parse_google(docstring).value;
    match &result.sections[0].body {
        GoogleSectionBody::Tip(text) => {
            assert_eq!(text.value, "Use vectorized operations for speed.");
        }
        _ => panic!("Expected Tip section body"),
    }
}

// =============================================================================
// Napoleon: Case-insensitive section headers
// =============================================================================

#[test]
fn test_napoleon_case_insensitive() {
    let docstring = "Summary.\n\nkeyword args:\n    x (int): Value.";
    let result = parse_google(docstring).value;
    assert_eq!(result.keyword_args().len(), 1);
}

#[test]
fn test_see_also_case_insensitive() {
    let docstring = "Summary.\n\nsee also:\n    func_a: Description.";
    let result = parse_google(docstring).value;
    assert_eq!(result.see_also().len(), 1);
}

// =============================================================================
// Napoleon: Full docstring with all sections
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
    let doc = &result.value;
    assert_eq!(doc.summary.value, "Calculate something.");
    assert!(doc.description.is_some());
    assert_eq!(doc.args().len(), 1);
    assert_eq!(doc.keyword_args().len(), 1);
    assert_eq!(doc.returns().len(), 1);
    assert_eq!(doc.raises().len(), 1);
    assert_eq!(doc.warns().len(), 1);
    assert_eq!(doc.see_also().len(), 1);
    assert!(doc.notes().is_some());
    assert!(doc.examples().is_some());
    assert!(
        result.diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        result.diagnostics
    );
}
