use super::*;

// =============================================================================
// Parameters section
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
    let result = parse_numpy(docstring);

    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Calculate the sum of two numbers."
    );
    assert_eq!(parameters(&result).len(), 2);

    assert_eq!(
        parameters(&result)[0].names[0].source_text(&result.source),
        "x"
    );
    assert_eq!(
        parameters(&result)[0]
            .r#type
            .as_ref()
            .map(|t| t.source_text(&result.source)),
        Some("int")
    );
    assert_eq!(
        parameters(&result)[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The first number."
    );

    assert_eq!(
        parameters(&result)[1].names[0].source_text(&result.source),
        "y"
    );
    assert_eq!(
        parameters(&result)[1]
            .r#type
            .as_ref()
            .map(|t| t.source_text(&result.source)),
        Some("int")
    );

    assert!(!returns(&result).is_empty());
    assert_eq!(
        returns(&result)[0]
            .return_type
            .as_ref()
            .map(|t| t.source_text(&result.source)),
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
    let result = parse_numpy(docstring);

    assert_eq!(parameters(&result).len(), 2);
    assert!(parameters(&result)[0].optional.is_none());
    assert!(parameters(&result)[1].optional.is_some());
    assert_eq!(
        parameters(&result)[1]
            .r#type
            .as_ref()
            .map(|t| t.source_text(&result.source)),
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
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 2);

    assert_eq!(
        parameters(&result)[0].names[0].source_text(&result.source),
        "x"
    );
    assert_eq!(
        parameters(&result)[1].names[0].source_text(&result.source),
        "y"
    );
    assert_eq!(
        parameters(&result)[0]
            .r#type
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "int"
    );
}

/// Parameters with no space before colon: `x: int`
#[test]
fn test_parameters_no_space_before_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx: int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].names[0].source_text(&result.source), "x");
    assert_eq!(
        p[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        p[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The value."
    );
}

/// Parameters with no space after colon: `x :int`
#[test]
fn test_parameters_no_space_after_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx :int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].names[0].source_text(&result.source), "x");
    assert_eq!(
        p[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

/// Parameters with no spaces around colon: `x:int`
#[test]
fn test_parameters_no_spaces_around_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx:int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].names[0].source_text(&result.source), "x");
    assert_eq!(
        p[0].r#type.as_ref().unwrap().source_text(&result.source),
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
    let result = parse_numpy(docstring);
    let p = &parameters(&result)[0];
    assert_eq!(p.names.len(), 2);
    assert_eq!(p.names[0].source_text(&result.source), "x1");
    assert_eq!(p.names[1].source_text(&result.source), "x2");
    assert_eq!(p.names[0].source_text(&result.source), "x1");
    assert_eq!(p.names[1].source_text(&result.source), "x2");
}

#[test]
fn test_description_with_colon_not_treated_as_param() {
    let docstring = r#"Brief summary.

Parameters
----------
x : int
    A value like key: value should not split.
"#;
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    assert_eq!(
        parameters(&result)[0].names[0].source_text(&result.source),
        "x"
    );
    assert!(
        parameters(&result)[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source)
            .contains("key: value")
    );
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
    let result = parse_numpy(docstring);
    let desc = &parameters(&result)[0]
        .description
        .as_ref()
        .unwrap()
        .source_text(&result.source);
    assert!(desc.contains("First paragraph of x."));
    assert!(desc.contains("Second paragraph of x."));
    assert!(desc.contains('\n'));
}

// =============================================================================
// Enum / choices type
// =============================================================================

#[test]
fn test_enum_type_as_string() {
    let docstring =
        "Summary.\n\nParameters\n----------\norder : {'C', 'F', 'A'}\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    let p = &params[0];
    assert_eq!(p.names[0].source_text(&result.source), "order");
    assert_eq!(
        p.r#type.as_ref().unwrap().source_text(&result.source),
        "{'C', 'F', 'A'}"
    );
    assert_eq!(
        p.description.as_ref().unwrap().source_text(&result.source),
        "Memory layout."
    );
}

#[test]
fn test_enum_type_with_optional() {
    let docstring =
        "Summary.\n\nParameters\n----------\norder : {'C', 'F'}, optional\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    let p = &params[0];

    assert!(p.optional.is_some());
    assert_eq!(
        p.r#type.as_ref().unwrap().source_text(&result.source),
        "{'C', 'F'}"
    );
}

#[test]
fn test_enum_type_with_default() {
    let docstring = "Summary.\n\nParameters\n----------\norder : {'C', 'F', 'A'}, default 'C'\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    let p = &params[0];

    assert_eq!(
        p.r#type.as_ref().unwrap().source_text(&result.source),
        "{'C', 'F', 'A'}"
    );
    assert_eq!(
        p.default_keyword
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "default"
    );
    assert!(p.default_separator.is_none());
    assert_eq!(
        p.default_value
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "'C'"
    );
}

// =============================================================================
// Parameters — aliases
// =============================================================================

/// `Params` alias for Parameters.
#[test]
fn test_params_alias() {
    let docstring = "Summary.\n\nParams\n------\nx : int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].names[0].source_text(&result.source), "x");
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Params"
    );
    assert_eq!(
        sections(&result)[0].header.kind,
        NumPySectionKind::Parameters
    );
}

/// `Param` alias for Parameters.
#[test]
fn test_param_alias() {
    let docstring = "Summary.\n\nParam\n-----\nx : int\n    The value.\n";
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Param"
    );
}

/// `Parameter` alias for Parameters.
#[test]
fn test_parameter_alias() {
    let docstring = "Summary.\n\nParameter\n---------\nx : int\n    The value.\n";
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Parameter"
    );
}

// =============================================================================
// Other Parameters section
// =============================================================================

#[test]
fn test_other_parameters_basic() {
    let docstring = "Summary.\n\nOther Parameters\n----------------\ndebug : bool\n    Enable debug mode.\nverbose : bool, optional\n    Verbose output.\n";
    let result = parse_numpy(docstring);
    let op = other_parameters(&result);
    assert_eq!(op.len(), 2);
    assert_eq!(op[0].names[0].source_text(&result.source), "debug");
    assert_eq!(
        op[0].r#type.as_ref().unwrap().source_text(&result.source),
        "bool"
    );
    assert_eq!(op[1].names[0].source_text(&result.source), "verbose");
    assert!(op[1].optional.is_some());
}

/// `Other Params` alias.
#[test]
fn test_other_params_alias() {
    let docstring = "Summary.\n\nOther Params\n------------\nx : int\n    Extra.\n";
    let result = parse_numpy(docstring);
    assert_eq!(other_parameters(&result).len(), 1);
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Other Params"
    );
    assert_eq!(
        sections(&result)[0].header.kind,
        NumPySectionKind::OtherParameters
    );
}

/// Other Parameters section body variant check.
#[test]
fn test_other_parameters_section_body_variant() {
    let docstring = "Summary.\n\nOther Parameters\n----------------\nx : int\n    Extra.\n";
    let result = parse_numpy(docstring);
    match &sections(&result)[0].body {
        NumPySectionBody::OtherParameters(params) => {
            assert_eq!(params.len(), 1);
        }
        other => panic!("Expected OtherParameters section body, got {:?}", other),
    }
}

// =============================================================================
// Receives section
// =============================================================================

#[test]
fn test_receives_basic() {
    let docstring = "Summary.\n\nReceives\n--------\ndata : bytes\n    The received data.\n";
    let result = parse_numpy(docstring);
    let r = receives(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].names[0].source_text(&result.source), "data");
    assert_eq!(
        r[0].r#type.as_ref().unwrap().source_text(&result.source),
        "bytes"
    );
    assert_eq!(
        r[0].description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "The received data."
    );
}

#[test]
fn test_receives_multiple() {
    let docstring = "Summary.\n\nReceives\n--------\nmsg : str\n    The message.\ndata : bytes\n    The payload.\n";
    let result = parse_numpy(docstring);
    let r = receives(&result);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].names[0].source_text(&result.source), "msg");
    assert_eq!(r[1].names[0].source_text(&result.source), "data");
}

/// `Receive` alias.
#[test]
fn test_receive_alias() {
    let docstring = "Summary.\n\nReceive\n-------\ndata : bytes\n    The data.\n";
    let result = parse_numpy(docstring);
    assert_eq!(receives(&result).len(), 1);
    assert_eq!(
        sections(&result)[0].header.name.source_text(&result.source),
        "Receive"
    );
    assert_eq!(sections(&result)[0].header.kind, NumPySectionKind::Receives);
}

/// Receives section body variant check.
#[test]
fn test_receives_section_body_variant() {
    let docstring = "Summary.\n\nReceives\n--------\ndata : bytes\n    Payload.\n";
    let result = parse_numpy(docstring);
    match &sections(&result)[0].body {
        NumPySectionBody::Receives(params) => {
            assert_eq!(params.len(), 1);
        }
        other => panic!("Expected Receives section body, got {:?}", other),
    }
}
