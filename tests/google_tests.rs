//! Integration tests for Google-style docstring parser.

use pydocstring::google::parse_google;
use pydocstring::ast::DocstringLike;
use pydocstring::GoogleSectionBody;

// =============================================================================
// Basic parsing
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.value, "This is a brief summary.");
}

#[test]
fn test_summary_span() {
    let docstring = "Brief description.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.span.start.line, 0);
    assert_eq!(result.summary.span.start.column, 0);
    assert_eq!(result.summary.span.end.column, 18);
    assert_eq!(
        result.summary.span.source_text(&result.source),
        "Brief description."
    );
}

#[test]
fn test_empty_docstring() {
    let result = parse_google("").unwrap();
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_google("   \n   \n").unwrap();
    assert_eq!(result.summary.value, "");
}

#[test]
fn test_summary_with_description() {
    let docstring = "Brief summary.\n\nExtended description that provides\nmore details about the function.";
    let result = parse_google(docstring).unwrap();

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
    let result = parse_google(docstring).unwrap();
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
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[0].description.value, "The value.");
}

#[test]
fn test_args_multiple() {
    let docstring = "Summary.\n\nArgs:\n    x (int): First.\n    y (str): Second.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[1].name.value, "y");
    assert_eq!(result.args()[1].arg_type.as_ref().unwrap().value, "str");
}

#[test]
fn test_args_no_type() {
    let docstring = "Summary.\n\nArgs:\n    x: The value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args()[0].name.value, "x");
    assert!(result.args()[0].arg_type.is_none());
    assert_eq!(result.args()[0].description.value, "The value.");
}

#[test]
fn test_args_optional() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): The value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert!(result.args()[0].optional.is_some());
}

#[test]
fn test_args_complex_type() {
    let docstring = "Summary.\n\nArgs:\n    data (Dict[str, List[int]]): The data.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "Dict[str, List[int]]"
    );
}

#[test]
fn test_args_tuple_type() {
    let docstring = "Summary.\n\nArgs:\n    pair (Tuple[int, str]): A pair of values.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "Tuple[int, str]"
    );
}

#[test]
fn test_args_multiline_description() {
    let docstring =
        "Summary.\n\nArgs:\n    x (int): First line.\n        Second line.\n        Third line.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.args()[0].description.value,
        "First line.\nSecond line.\nThird line."
    );
}

#[test]
fn test_args_description_on_next_line() {
    let docstring = "Summary.\n\nArgs:\n    x (int):\n        The description.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
    assert_eq!(result.args()[0].description.value, "The description.");
}

#[test]
fn test_args_varargs() {
    let docstring = "Summary.\n\nArgs:\n    *args: Positional args.\n    **kwargs: Keyword args.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.args()[0].name.value, "*args");
    assert_eq!(result.args()[0].description.value, "Positional args.");
    assert_eq!(result.args()[1].name.value, "**kwargs");
    assert_eq!(result.args()[1].description.value, "Keyword args.");
}

#[test]
fn test_args_kwargs_with_type() {
    let docstring = "Summary.\n\nArgs:\n    **kwargs (dict): Keyword arguments.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args()[0].name.value, "**kwargs");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "dict");
}

#[test]
fn test_arguments_alias() {
    let docstring = "Summary.\n\nArguments:\n    x (int): The value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
}

// =============================================================================
// Args span accuracy
// =============================================================================

#[test]
fn test_args_name_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring).unwrap();
    let arg = &result.args()[0];
    assert_eq!(arg.name.span.start.line, 3);
    assert_eq!(arg.name.span.start.column, 4);
    assert_eq!(arg.name.span.end.column, 5);
    assert_eq!(arg.name.span.source_text(&result.source), "x");
}

#[test]
fn test_args_type_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring).unwrap();
    let arg = &result.args()[0];
    let type_span = arg.arg_type.as_ref().unwrap();
    assert_eq!(type_span.span.start.line, 3);
    assert_eq!(type_span.span.source_text(&result.source), "int");
}

#[test]
fn test_args_optional_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): Value.";
    let result = parse_google(docstring).unwrap();
    let opt_span = result.args()[0].optional.unwrap();
    assert_eq!(opt_span.source_text(&result.source), "optional");
}

// =============================================================================
// Returns section
// =============================================================================

#[test]
fn test_returns_with_type() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.returns().len(), 1);
    assert_eq!(result.returns()[0].return_type.as_ref().unwrap().value, "int");
    assert_eq!(result.returns()[0].description.value, "The result.");
}

#[test]
fn test_returns_multiple() {
    let docstring = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.returns().len(), 2);
    assert_eq!(result.returns()[0].return_type.as_ref().unwrap().value, "int");
    assert_eq!(result.returns()[1].return_type.as_ref().unwrap().value, "str");
}

#[test]
fn test_returns_without_type() {
    let docstring = "Summary.\n\nReturns:\n    The computed result.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.returns().len(), 1);
    assert!(result.returns()[0].return_type.is_none());
    assert_eq!(result.returns()[0].description.value, "The computed result.");
}

#[test]
fn test_returns_multiline_description() {
    let docstring =
        "Summary.\n\nReturns:\n    int: The result\n        of the computation.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.returns()[0].description.value,
        "The result\nof the computation."
    );
}

#[test]
fn test_return_alias() {
    let docstring = "Summary.\n\nReturn:\n    int: The value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.returns().len(), 1);
}

// =============================================================================
// Yields section
// =============================================================================

#[test]
fn test_yields() {
    let docstring = "Summary.\n\nYields:\n    int: The next value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.yields().len(), 1);
    assert_eq!(result.yields()[0].return_type.as_ref().unwrap().value, "int");
    assert_eq!(result.yields()[0].description.value, "The next value.");
}

#[test]
fn test_yield_alias() {
    let docstring = "Summary.\n\nYield:\n    str: Next string.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.yields().len(), 1);
}

// =============================================================================
// Raises section
// =============================================================================

#[test]
fn test_raises_single() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the input is invalid.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.raises().len(), 1);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(
        result.raises()[0].description.value,
        "If the input is invalid."
    );
}

#[test]
fn test_raises_multiple() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If invalid.\n    TypeError: If wrong type.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.raises().len(), 2);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(result.raises()[1].exception_type.value, "TypeError");
}

#[test]
fn test_raises_multiline_description() {
    let docstring =
        "Summary.\n\nRaises:\n    ValueError: If the\n        input is invalid.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.raises()[0].description.value,
        "If the\ninput is invalid."
    );
}

#[test]
fn test_raises_exception_type_span() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.raises()[0]
            .exception_type
            .span
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
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.attributes().len(), 2);
    assert_eq!(result.attributes()[0].name.value, "name");
    assert_eq!(result.attributes()[0].attr_type.as_ref().unwrap().value, "str");
    assert_eq!(result.attributes()[1].name.value, "age");
}

#[test]
fn test_attributes_no_type() {
    let docstring = "Summary.\n\nAttributes:\n    name: The name.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.attributes()[0].name.value, "name");
    assert!(result.attributes()[0].attr_type.is_none());
}

// =============================================================================
// Free-text sections
// =============================================================================

#[test]
fn test_note_section() {
    let docstring = "Summary.\n\nNote:\n    This is a note.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.note().unwrap().value, "This is a note.");
}

#[test]
fn test_notes_alias() {
    let docstring = "Summary.\n\nNotes:\n    This is also a note.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.note().unwrap().value, "This is also a note.");
}

#[test]
fn test_example_section() {
    let docstring = "Summary.\n\nExample:\n    >>> func(1)\n    1";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.example().unwrap().value, ">>> func(1)\n1");
}

#[test]
fn test_examples_alias() {
    let docstring = "Summary.\n\nExamples:\n    >>> 1 + 1\n    2";
    let result = parse_google(docstring).unwrap();
    assert!(result.example().is_some());
}

#[test]
fn test_references_section() {
    let docstring = "Summary.\n\nReferences:\n    Author, Title, 2024.";
    let result = parse_google(docstring).unwrap();
    assert!(result.references().is_some());
}

#[test]
fn test_warnings_section() {
    let docstring = "Summary.\n\nWarnings:\n    This function is deprecated.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.warnings().unwrap().value,
        "This function is deprecated."
    );
}

// =============================================================================
// Todo section
// =============================================================================

#[test]
fn test_todo_with_bullets() {
    let docstring = "Summary.\n\nTodo:\n    * Item one.\n    * Item two.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.todo().len(), 2);
    assert_eq!(result.todo()[0].value, "Item one.");
    assert_eq!(result.todo()[1].value, "Item two.");
}

#[test]
fn test_todo_without_bullets() {
    let docstring = "Summary.\n\nTodo:\n    Implement feature X.\n    Fix bug Y.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.todo().len(), 2);
    assert_eq!(result.todo()[0].value, "Implement feature X.");
    assert_eq!(result.todo()[1].value, "Fix bug Y.");
}

#[test]
fn test_todo_multiline_item() {
    let docstring = "Summary.\n\nTodo:\n    * Item one that\n        continues here.\n    * Item two.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.todo().len(), 2);
    assert_eq!(result.todo()[0].value, "Item one that\ncontinues here.");
    assert_eq!(result.todo()[1].value, "Item two.");
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

    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.value, "Calculate the sum.");
    assert!(result.description.is_some());
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.returns().len(), 1);
    assert_eq!(result.raises().len(), 1);
    assert!(result.example().is_some());
    assert!(result.note().is_some());
}

#[test]
fn test_sections_with_blank_lines() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\n    y (str): Name.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args().len(), 2);
    assert_eq!(result.returns().len(), 1);
}

// =============================================================================
// Section order preservation
// =============================================================================

#[test]
fn test_section_order() {
    let docstring = "Summary.\n\nReturns:\n    int: Value.\n\nArgs:\n    x: Input.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.sections.len(), 2);
    assert_eq!(result.sections[0].header.name.value, "Returns");
    assert_eq!(result.sections[1].header.name.value, "Args");
}

#[test]
fn test_section_header_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).unwrap();
    let header = &result.sections[0].header;
    assert_eq!(header.name.value, "Args");
    assert_eq!(header.name.span.source_text(&result.source), "Args");
    assert_eq!(header.span.source_text(&result.source), "Args:");
}

#[test]
fn test_section_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).unwrap();
    let section = &result.sections[0];
    assert_eq!(
        section.span.source_text(&result.source),
        "Args:\n    x: Value."
    );
}

// =============================================================================
// Unknown sections
// =============================================================================

#[test]
fn test_unknown_section_preserved() {
    let docstring = "Summary.\n\nCustom:\n    Some custom content.";
    let result = parse_google(docstring).unwrap();
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
    let docstring = "Summary.\n\nArgs:\n    x: Value.\n\nCustom:\n    Content.\n\nReturns:\n    int: Result.";
    let result = parse_google(docstring).unwrap();
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
    let result = parse_google(docstring).unwrap();
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
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.value, "Summary.");
    assert_eq!(result.args().len(), 1);
    assert_eq!(result.args()[0].name.value, "x");
    assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
}

#[test]
fn test_indented_summary_span() {
    let docstring = "    Summary.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.span.start.column, 4);
    assert_eq!(result.summary.span.end.column, 12);
    assert_eq!(result.summary.span.source_text(&result.source), "Summary.");
}

// =============================================================================
// DocstringLike trait
// =============================================================================

#[test]
fn test_docstring_like_summary() {
    let docstring = "Summary.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary(), "Summary.");
}

#[test]
fn test_docstring_like_parameters() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n    y (str): Name.";
    let result = parse_google(docstring).unwrap();
    let params = result.parameters();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name.value, "x");
    assert_eq!(params[0].param_type.as_ref().unwrap().value, "int");
    assert_eq!(params[1].name.value, "y");
}

#[test]
fn test_docstring_like_returns() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring).unwrap();
    let returns = DocstringLike::returns(&result);
    assert_eq!(returns.len(), 1);
    assert_eq!(returns[0].return_type.as_ref().unwrap().value, "int");
}

#[test]
fn test_docstring_like_raises() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring).unwrap();
    let raises = DocstringLike::raises(&result);
    assert_eq!(raises.len(), 1);
    assert_eq!(raises[0].exception_type.value, "ValueError");
}

// =============================================================================
// Span round-trip
// =============================================================================

#[test]
fn test_span_source_text_round_trip() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring).unwrap();

    // Summary
    assert_eq!(
        result.summary.span.source_text(&result.source),
        "Summary."
    );

    // Arg name
    assert_eq!(
        result.args()[0].name.span.source_text(&result.source),
        "x"
    );

    // Arg type
    assert_eq!(
        result.args()[0]
            .arg_type
            .as_ref()
            .unwrap()
            .span
            .source_text(&result.source),
        "int"
    );

    // Return type
    assert_eq!(
        result.returns()[0]
            .return_type
            .as_ref()
            .unwrap()
            .span
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
    let result = parse_google(docstring).unwrap();
    // "Args:" at base_indent=0 is a section header, so summary remains empty
    assert_eq!(result.args().len(), 1);
}

#[test]
fn test_leading_blank_lines() {
    let docstring = "\n\n\nSummary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.summary.value, "Summary.");
    assert_eq!(result.args().len(), 1);
}

#[test]
fn test_optional_only_in_parens() {
    let docstring = "Summary.\n\nArgs:\n    x (optional): Value.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(result.args()[0].name.value, "x");
    assert!(result.args()[0].arg_type.is_none());
    assert!(result.args()[0].optional.is_some());
}

#[test]
fn test_complex_optional_type() {
    let docstring = "Summary.\n\nArgs:\n    x (List[int], optional): Values.";
    let result = parse_google(docstring).unwrap();
    assert_eq!(
        result.args()[0].arg_type.as_ref().unwrap().value,
        "List[int]"
    );
    assert!(result.args()[0].optional.is_some());
}
