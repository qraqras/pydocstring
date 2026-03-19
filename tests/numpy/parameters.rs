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
        doc(&result).summary().unwrap().text(result.source()),
        "Calculate the sum of two numbers."
    );
    assert_eq!(parameters(&result).len(), 2);

    let names0: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "x");
    assert_eq!(
        parameters(&result)[0].r#type().map(|t| t.text(result.source())),
        Some("int")
    );
    assert_eq!(
        parameters(&result)[0].description().unwrap().text(result.source()),
        "The first number."
    );

    let names1: Vec<_> = parameters(&result)[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "y");
    assert_eq!(
        parameters(&result)[1].r#type().map(|t| t.text(result.source())),
        Some("int")
    );

    assert!(!returns(&result).is_empty());
    assert_eq!(
        returns(&result)[0].return_type().map(|t| t.text(result.source())),
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
    assert!(parameters(&result)[0].optional().is_none());
    assert!(parameters(&result)[1].optional().is_some());
    assert_eq!(
        parameters(&result)[1].r#type().map(|t| t.text(result.source())),
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

    let names0: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "x");
    let names1: Vec<_> = parameters(&result)[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "y");
    assert_eq!(parameters(&result)[0].r#type().unwrap().text(result.source()), "int");
}

/// Parameters with no space before colon: `x: int`
#[test]
fn test_parameters_no_space_before_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx: int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    let names: Vec<_> = p[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(p[0].r#type().unwrap().text(result.source()), "int");
    assert_eq!(p[0].description().unwrap().text(result.source()), "The value.");
}

/// Parameters with no space after colon: `x :int`
#[test]
fn test_parameters_no_space_after_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx :int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    let names: Vec<_> = p[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(p[0].r#type().unwrap().text(result.source()), "int");
}

/// Parameters with no spaces around colon: `x:int`
#[test]
fn test_parameters_no_spaces_around_colon() {
    let docstring = "Summary.\n\nParameters\n----------\nx:int\n    The value.\n";
    let result = parse_numpy(docstring);
    let p = parameters(&result);
    assert_eq!(p.len(), 1);
    let names: Vec<_> = p[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(p[0].r#type().unwrap().text(result.source()), "int");
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
    let names: Vec<_> = p.names().collect();
    assert_eq!(names.len(), 2);
    assert_eq!(names[0].text(result.source()), "x1");
    assert_eq!(names[1].text(result.source()), "x2");
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
    let names: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert!(
        parameters(&result)[0]
            .description()
            .unwrap()
            .text(result.source())
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
    let desc = parameters(&result)[0].description().unwrap().text(result.source());
    assert!(desc.contains("First paragraph of x."));
    assert!(desc.contains("Second paragraph of x."));
    assert!(desc.contains('\n'));
}

// =============================================================================
// Enum / choices type
// =============================================================================

#[test]
fn test_enum_type_as_string() {
    let docstring = "Summary.\n\nParameters\n----------\norder : {'C', 'F', 'A'}\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    let p = &params[0];
    let names: Vec<_> = p.names().collect();
    assert_eq!(names[0].text(result.source()), "order");
    assert_eq!(p.r#type().unwrap().text(result.source()), "{'C', 'F', 'A'}");
    assert_eq!(p.description().unwrap().text(result.source()), "Memory layout.");
}

#[test]
fn test_enum_type_with_optional() {
    let docstring = "Summary.\n\nParameters\n----------\norder : {'C', 'F'}, optional\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    let p = &params[0];

    assert!(p.optional().is_some());
    assert_eq!(p.r#type().unwrap().text(result.source()), "{'C', 'F'}");
}

#[test]
fn test_enum_type_with_default() {
    let docstring = "Summary.\n\nParameters\n----------\norder : {'C', 'F', 'A'}, default 'C'\n    Memory layout.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    let p = &params[0];

    assert_eq!(p.r#type().unwrap().text(result.source()), "{'C', 'F', 'A'}");
    assert_eq!(p.default_keyword().unwrap().text(result.source()), "default");
    assert!(p.default_separator().is_none());
    assert_eq!(p.default_value().unwrap().text(result.source()), "'C'");
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
    let names: Vec<_> = p[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Params");
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Parameters
    );
}

/// `Param` alias for Parameters.
#[test]
fn test_param_alias() {
    let docstring = "Summary.\n\nParam\n-----\nx : int\n    The value.\n";
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    assert_eq!(all_sections(&result)[0].header().name().text(result.source()), "Param");
}

/// `Parameter` alias for Parameters.
#[test]
fn test_parameter_alias() {
    let docstring = "Summary.\n\nParameter\n---------\nx : int\n    The value.\n";
    let result = parse_numpy(docstring);
    assert_eq!(parameters(&result).len(), 1);
    assert_eq!(
        all_sections(&result)[0].header().name().text(result.source()),
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
    let names0: Vec<_> = op[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "debug");
    assert_eq!(op[0].r#type().unwrap().text(result.source()), "bool");
    let names1: Vec<_> = op[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "verbose");
    assert!(op[1].optional().is_some());
}

/// `Other Params` alias.
#[test]
fn test_other_params_alias() {
    let docstring = "Summary.\n\nOther Params\n------------\nx : int\n    Extra.\n";
    let result = parse_numpy(docstring);
    assert_eq!(other_parameters(&result).len(), 1);
    assert_eq!(
        all_sections(&result)[0].header().name().text(result.source()),
        "Other Params"
    );
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::OtherParameters
    );
}

/// Other Parameters section body variant check.
#[test]
fn test_other_parameters_section_body_variant() {
    let docstring = "Summary.\n\nOther Parameters\n----------------\nx : int\n    Extra.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::OtherParameters);
    let params: Vec<_> = s.parameters().collect();
    assert_eq!(params.len(), 1);
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
    let names: Vec<_> = r[0].names().collect();
    assert_eq!(names[0].text(result.source()), "data");
    assert_eq!(r[0].r#type().unwrap().text(result.source()), "bytes");
    assert_eq!(r[0].description().unwrap().text(result.source()), "The received data.");
}

#[test]
fn test_receives_multiple() {
    let docstring = "Summary.\n\nReceives\n--------\nmsg : str\n    The message.\ndata : bytes\n    The payload.\n";
    let result = parse_numpy(docstring);
    let r = receives(&result);
    assert_eq!(r.len(), 2);
    let names0: Vec<_> = r[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "msg");
    let names1: Vec<_> = r[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "data");
}

/// `Receive` alias.
#[test]
fn test_receive_alias() {
    let docstring = "Summary.\n\nReceive\n-------\ndata : bytes\n    The data.\n";
    let result = parse_numpy(docstring);
    assert_eq!(receives(&result).len(), 1);
    assert_eq!(
        all_sections(&result)[0].header().name().text(result.source()),
        "Receive"
    );
    assert_eq!(
        all_sections(&result)[0].section_kind(result.source()),
        NumPySectionKind::Receives
    );
}

/// Receives section body variant check.
#[test]
fn test_receives_section_body_variant() {
    let docstring = "Summary.\n\nReceives\n--------\ndata : bytes\n    Payload.\n";
    let result = parse_numpy(docstring);
    let s = &all_sections(&result)[0];
    assert_eq!(s.section_kind(result.source()), NumPySectionKind::Receives);
    let params: Vec<_> = s.parameters().collect();
    assert_eq!(params.len(), 1);
}

// =============================================================================
// Google-style entry format in NumPy sections
// =============================================================================

#[test]
fn test_google_style_entry_in_numpy_section() {
    let docstring = "Summary.\n\nParameters\n----------\nname (str): The name.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    let names: Vec<_> = params[0].names().collect();
    assert_eq!(names[0].text(result.source()), "name");
    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("str"));
    assert_eq!(
        params[0].description().map(|t| t.text(result.source())),
        Some("The name.")
    );
}

#[test]
fn test_google_style_entry_with_optional() {
    let docstring = "Summary.\n\nParameters\n----------\nname (str, optional): The name.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    let names: Vec<_> = params[0].names().collect();
    assert_eq!(names[0].text(result.source()), "name");
    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("str"));
    assert!(params[0].optional().is_some());
    assert_eq!(
        params[0].description().map(|t| t.text(result.source())),
        Some("The name.")
    );
}

#[test]
fn test_google_style_entry_no_description() {
    let docstring = "Summary.\n\nParameters\n----------\nname (int):\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    let names: Vec<_> = params[0].names().collect();
    assert_eq!(names[0].text(result.source()), "name");
    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("int"));
    assert!(params[0].description().is_none());
}

#[test]
fn test_google_style_entry_with_continuation() {
    let docstring = "Summary.\n\nParameters\n----------\nname (str): The name.\n    Continued here.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("str"));
    assert_eq!(
        params[0].description().map(|t| t.text(result.source())),
        Some("The name.\n    Continued here.")
    );
}

#[test]
fn test_google_style_entry_complex_type() {
    let docstring = "Summary.\n\nParameters\n----------\ndata (Dict[str, int]): The mapping.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    assert_eq!(
        params[0].r#type().map(|t| t.text(result.source())),
        Some("Dict[str, int]")
    );
    assert_eq!(
        params[0].description().map(|t| t.text(result.source())),
        Some("The mapping.")
    );
}

#[test]
fn test_google_style_mixed_with_numpy_style() {
    let docstring = "Summary.\n\nParameters\n----------\nx (int): First.\ny : str\n    Second.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 2);

    // Google-style entry
    assert_eq!(params[0].names().next().unwrap().text(result.source()), "x");
    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("int"));
    assert_eq!(params[0].description().map(|t| t.text(result.source())), Some("First."));

    // NumPy-style entry
    assert_eq!(params[1].names().next().unwrap().text(result.source()), "y");
    assert_eq!(params[1].r#type().map(|t| t.text(result.source())), Some("str"));
    assert_eq!(
        params[1].description().map(|t| t.text(result.source())),
        Some("Second.")
    );
}

#[test]
fn test_google_style_entry_no_colon_after_bracket() {
    let docstring = "Summary.\n\nParameters\n----------\nname (int)\n    Desc.\n";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);

    assert_eq!(params[0].names().next().unwrap().text(result.source()), "name");
    assert_eq!(params[0].r#type().map(|t| t.text(result.source())), Some("int"));
    assert_eq!(params[0].description().map(|t| t.text(result.source())), Some("Desc."));
}
