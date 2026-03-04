use super::*;

// =============================================================================
// Args section — basic
// =============================================================================

#[test]
fn test_args_basic() {
    let docstring = "Summary.\n\nArgs:\n    x (int): The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The value."
    );
}

#[test]
fn test_args_multiple() {
    let docstring = "Summary.\n\nArgs:\n    x (int): First.\n    y (str): Second.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(a[1].name.source_text(&result.source), "y");
    assert_eq!(
        a[1].r#type.as_ref().unwrap().source_text(&result.source),
        "str"
    );
}

#[test]
fn test_args_no_type() {
    let docstring = "Summary.\n\nArgs:\n    x: The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert!(a[0].r#type.is_none());
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The value."
    );
}

/// Colon with no space after it: `name:description`
#[test]
fn test_args_no_space_after_colon() {
    let docstring = "Summary.\n\nArgs:\n    x:The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The value."
    );
}

/// Colon with extra spaces: `name:   description`
#[test]
fn test_args_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nArgs:\n    x:   The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The value."
    );
}

#[test]
fn test_args_optional() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert!(a[0].optional.is_some());
}

#[test]
fn test_args_complex_type() {
    let docstring = "Summary.\n\nArgs:\n    data (Dict[str, List[int]]): The data.";
    let result = parse_google(docstring);
    assert_eq!(
        args(&result)[0]
            .r#type
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "Dict[str, List[int]]"
    );
}

#[test]
fn test_args_tuple_type() {
    let docstring = "Summary.\n\nArgs:\n    pair (Tuple[int, str]): A pair of values.";
    let result = parse_google(docstring);
    assert_eq!(
        args(&result)[0]
            .r#type
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "Tuple[int, str]"
    );
}

#[test]
fn test_args_multiline_description() {
    let docstring =
        "Summary.\n\nArgs:\n    x (int): First line.\n        Second line.\n        Third line.";
    let result = parse_google(docstring);
    assert_eq!(
        args(&result)[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "First line.\n        Second line.\n        Third line."
    );
}

#[test]
fn test_args_description_on_next_line() {
    let docstring = "Summary.\n\nArgs:\n    x (int):\n        The description.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The description."
    );
}

#[test]
fn test_args_varargs() {
    let docstring = "Summary.\n\nArgs:\n    *args: Positional args.\n    **kwargs: Keyword args.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name.source_text(&result.source), "*args");
    assert_eq!(
        a[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "Positional args."
    );
    assert_eq!(a[1].name.source_text(&result.source), "**kwargs");
    assert_eq!(
        a[1].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "Keyword args."
    );
}

#[test]
fn test_args_kwargs_with_type() {
    let docstring = "Summary.\n\nArgs:\n    **kwargs (dict): Keyword arguments.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "**kwargs");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "dict"
    );
}

// =============================================================================
// Args — aliases
// =============================================================================

#[test]
fn test_arguments_alias() {
    let docstring = "Summary.\n\nArguments:\n    x (int): The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
}

#[test]
fn test_parameters_alias() {
    let docstring = "Summary.\n\nParameters:\n    x (int): The value.";
    let result = parse_google(docstring);
    assert_eq!(args(&result).len(), 1);
    assert_eq!(args(&result)[0].name.source_text(&result.source), "x");
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Parameters"
    );
}

#[test]
fn test_params_alias() {
    let docstring = "Summary.\n\nParams:\n    x (int): The value.";
    let result = parse_google(docstring);
    assert_eq!(args(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Params"
    );
}

// =============================================================================
// Args — span accuracy
// =============================================================================

#[test]
fn test_args_name_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring);
    let arg = &args(&result)[0];
    let index = LineIndex::from_source(&result.source);
    let (line, col) = index.line_col(arg.name.start());
    assert_eq!(line, 3);
    assert_eq!(col, 4);
    assert_eq!(arg.name.end(), TextSize::new(arg.name.start().raw() + 1));
    assert_eq!(arg.name.source_text(&result.source), "x");
}

#[test]
fn test_args_type_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.";
    let result = parse_google(docstring);
    let arg = &args(&result)[0];
    let type_span = arg.r#type.as_ref().unwrap();
    let index = LineIndex::from_source(&result.source);
    let (line, _col) = index.line_col(type_span.start());
    assert_eq!(line, 3);
    assert_eq!(type_span.source_text(&result.source), "int");
}

#[test]
fn test_args_optional_span() {
    let docstring = "Summary.\n\nArgs:\n    x (int, optional): Value.";
    let result = parse_google(docstring);
    let opt_span = args(&result)[0].optional.as_ref().unwrap();
    assert_eq!(opt_span.source_text(&result.source), "optional");
}

// =============================================================================
// Args — bracket types
// =============================================================================

#[test]
fn test_args_square_bracket_type() {
    let docstring = "Summary.\n\nArgs:\n    x [int]: The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(a.name.source_text(&result.source), "x");
    assert_eq!(
        a.r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "["
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "]"
    );
    assert_eq!(
        a.description.as_ref().unwrap().source_text(&result.source),
        "The value."
    );
}

#[test]
fn test_args_curly_bracket_type() {
    let docstring = "Summary.\n\nArgs:\n    x {int}: The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(a.name.source_text(&result.source), "x");
    assert_eq!(
        a.r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "{"
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "}"
    );
    assert_eq!(
        a.description.as_ref().unwrap().source_text(&result.source),
        "The value."
    );
}

#[test]
fn test_args_paren_bracket_spans() {
    let docstring = "Summary.\n\nArgs:\n    x (int): The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "("
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "("
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        ")"
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        ")"
    );
}

#[test]
fn test_args_no_bracket_fields_when_no_type() {
    let docstring = "Summary.\n\nArgs:\n    x: The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert!(a.open_bracket.is_none());
    assert!(a.close_bracket.is_none());
    assert!(a.r#type.is_none());
}

#[test]
fn test_args_square_bracket_optional() {
    let docstring = "Summary.\n\nArgs:\n    x [int, optional]: The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(a.name.source_text(&result.source), "x");
    assert_eq!(
        a.r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "["
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "]"
    );
    assert!(a.optional.is_some());
}

#[test]
fn test_args_square_bracket_complex_type() {
    let docstring = "Summary.\n\nArgs:\n    items [List[int]]: The items.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(a.name.source_text(&result.source), "items");
    assert_eq!(
        a.r#type.as_ref().unwrap().source_text(&result.source),
        "List[int]"
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "["
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "]"
    );
}

#[test]
fn test_args_angle_bracket_type() {
    let docstring = "Summary.\n\nArgs:\n    x <int>: The value.";
    let result = parse_google(docstring);
    let a = &args(&result)[0];
    assert_eq!(a.name.source_text(&result.source), "x");
    assert_eq!(
        a.r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        a.open_bracket.as_ref().unwrap().source_text(&result.source),
        "<"
    );
    assert_eq!(
        a.close_bracket
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        ">"
    );
    assert_eq!(
        a.description.as_ref().unwrap().source_text(&result.source),
        "The value."
    );
}

// =============================================================================
// Args — optional edge cases
// =============================================================================

#[test]
fn test_optional_only_in_parens() {
    let docstring = "Summary.\n\nArgs:\n    x (optional): Value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert!(a[0].r#type.is_none());
    assert!(a[0].optional.is_some());
}

#[test]
fn test_complex_optional_type() {
    let docstring = "Summary.\n\nArgs:\n    x (List[int], optional): Values.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "List[int]"
    );
    assert!(a[0].optional.is_some());
}

// =============================================================================
// Keyword Args section
// =============================================================================

#[test]
fn test_keyword_args_basic() {
    let docstring = "Summary.\n\nKeyword Args:\n    timeout (int): Timeout in seconds.\n    retries (int): Number of retries.";
    let result = parse_google(docstring);
    let ka = keyword_args(&result);
    assert_eq!(ka.len(), 2);
    assert_eq!(ka[0].name.source_text(&result.source), "timeout");
    assert_eq!(
        ka[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(ka[1].name.source_text(&result.source), "retries");
}

#[test]
fn test_keyword_arguments_alias() {
    let docstring = "Summary.\n\nKeyword Arguments:\n    key (str): The key.";
    let result = parse_google(docstring);
    assert_eq!(keyword_args(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Keyword Arguments"
    );
}

#[test]
fn test_keyword_args_section_body_variant() {
    let docstring = "Summary.\n\nKeyword Args:\n    k (str): Key.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::KeywordArgs(args) => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected KeywordArgs section body"),
    }
}

// =============================================================================
// Other Parameters section
// =============================================================================

#[test]
fn test_other_parameters() {
    let docstring = "Summary.\n\nOther Parameters:\n    debug (bool): Enable debug mode.\n    verbose (bool, optional): Verbose output.";
    let result = parse_google(docstring);
    let op = other_parameters(&result);
    assert_eq!(op.len(), 2);
    assert_eq!(op[0].name.source_text(&result.source), "debug");
    assert_eq!(op[1].name.source_text(&result.source), "verbose");
    assert!(op[1].optional.is_some());
}

#[test]
fn test_other_parameters_section_body_variant() {
    let docstring = "Summary.\n\nOther Parameters:\n    x (int): Extra.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
        GoogleSectionBody::OtherParameters(args) => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected OtherParameters section body"),
    }
}

// =============================================================================
// Receives section
// =============================================================================

#[test]
fn test_receives() {
    let docstring = "Summary.\n\nReceives:\n    data (bytes): The received data.";
    let result = parse_google(docstring);
    let r = receives(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].name.source_text(&result.source), "data");
    assert_eq!(
        r[0].r#type.as_ref().unwrap().source_text(&result.source),
        "bytes"
    );
}

#[test]
fn test_receive_alias() {
    let docstring = "Summary.\n\nReceive:\n    msg (str): The message.";
    let result = parse_google(docstring);
    assert_eq!(receives(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Receive"
    );
}

// =============================================================================
// Convenience accessors
// =============================================================================

#[test]
fn test_docstring_like_parameters() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n    y (str): Name.";
    let result = parse_google(docstring);
    let params = args(&result);
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name.source_text(&result.source), "x");
    assert_eq!(
        params[0]
            .r#type
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "int"
    );
    assert_eq!(params[1].name.source_text(&result.source), "y");
}
