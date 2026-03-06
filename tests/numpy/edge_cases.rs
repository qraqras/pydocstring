use super::*;

// =============================================================================
// Indented docstrings (class/method bodies)
// =============================================================================

#[test]
fn test_indented_docstring() {
    let docstring = "    Summary line.\n\n    Parameters\n    ----------\n    x : int\n        Description of x.\n    y : str, optional\n        Description of y.\n\n    Returns\n    -------\n    bool\n        The result.\n";
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary line."
    );
    assert_eq!(parameters(&result).len(), 2);
    let names0: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "x");
    assert_eq!(
        parameters(&result)[0]
            .r#type()
            .map(|t| t.text(result.source())),
        Some("int")
    );
    let names1: Vec<_> = parameters(&result)[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "y");
    assert!(parameters(&result)[1].optional().is_some());
    assert_eq!(returns(&result).len(), 1);
    assert_eq!(
        returns(&result)[0]
            .return_type()
            .map(|t| t.text(result.source())),
        Some("bool")
    );

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary line."
    );
    let names0b: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names0b[0].text(result.source()), "x");
    assert_eq!(
        parameters(&result)[0]
            .r#type()
            .unwrap()
            .text(result.source()),
        "int"
    );
}

#[test]
fn test_deeply_indented_docstring() {
    let docstring = "        Brief.\n\n        Parameters\n        ----------\n        a : float\n            The value.\n\n        Raises\n        ------\n        ValueError\n            If bad.\n";
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Brief."
    );
    assert_eq!(parameters(&result).len(), 1);
    let names: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names[0].text(result.source()), "a");
    assert_eq!(raises(&result).len(), 1);
    assert_eq!(
        raises(&result)[0].r#type().text(result.source()),
        "ValueError"
    );
    assert_eq!(
        raises(&result)[0].r#type().text(result.source()),
        "ValueError"
    );
}

#[test]
fn test_indented_with_deprecation() {
    let docstring = "    Summary.\n\n    .. deprecated:: 2.0.0\n        Use new_func instead.\n\n    Parameters\n    ----------\n    x : int\n        Desc.\n";
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary."
    );
    let dep = doc(&result).deprecation().expect("should have deprecation");
    assert_eq!(dep.version().text(result.source()), "2.0.0");
    assert!(
        dep.description()
            .unwrap()
            .text(result.source())
            .contains("new_func")
    );
    assert_eq!(parameters(&result).len(), 1);
    let names: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
}

#[test]
fn test_mixed_indent_first_line() {
    let docstring =
        "Summary.\n\n    Parameters\n    ----------\n    x : int\n        Description.\n";
    let result = parse_numpy(docstring);

    assert_eq!(
        doc(&result).summary().unwrap().text(result.source()),
        "Summary."
    );
    assert_eq!(parameters(&result).len(), 1);
    let names: Vec<_> = parameters(&result)[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    assert_eq!(
        parameters(&result)[0]
            .description()
            .unwrap()
            .text(result.source()),
        "Description."
    );
}

// =============================================================================
// Tab indentation tests
// =============================================================================

/// Parameters section with tab-indented descriptions.
#[test]
fn test_tab_indented_parameters() {
    let docstring = "Summary.\n\nParameters\n----------\nx : int\n\tDescription of x.\ny : str\n\tDescription of y.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 2);
    let names0: Vec<_> = params[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "x");
    assert_eq!(
        params[0].description().unwrap().text(result.source()),
        "Description of x."
    );
    let names1: Vec<_> = params[1].names().collect();
    assert_eq!(names1[0].text(result.source()), "y");
    assert_eq!(
        params[1].description().unwrap().text(result.source()),
        "Description of y."
    );
}

/// Mixed tabs and spaces: header at 0 indent, description indented with tab.
#[test]
fn test_mixed_tab_space_parameters() {
    let docstring = "Summary.\n\nParameters\n----------\nx : int\n\tThe value.\n\t  More detail.";
    let result = parse_numpy(docstring);
    let params = parameters(&result);
    assert_eq!(params.len(), 1);
    let names: Vec<_> = params[0].names().collect();
    assert_eq!(names[0].text(result.source()), "x");
    let desc = params[0].description().unwrap().text(result.source());
    assert!(desc.contains("The value."), "desc = {:?}", desc);
}

/// Returns section with tab-indented descriptions.
#[test]
fn test_tab_indented_returns() {
    let docstring = "Summary.\n\nReturns\n-------\nint\n\tThe result value.";
    let result = parse_numpy(docstring);
    let rets = returns(&result);
    assert_eq!(rets.len(), 1);
    assert_eq!(
        rets[0].description().unwrap().text(result.source()),
        "The result value."
    );
}

/// Raises section with tab-indented description.
#[test]
fn test_tab_indented_raises() {
    let docstring = "Summary.\n\nRaises\n------\nValueError\n\tIf the input is invalid.";
    let result = parse_numpy(docstring);
    let exc = raises(&result);
    assert_eq!(exc.len(), 1);
    assert_eq!(exc[0].r#type().text(result.source()), "ValueError");
    assert_eq!(
        exc[0].description().unwrap().text(result.source()),
        "If the input is invalid."
    );
}
