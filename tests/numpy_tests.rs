//! Integration tests for NumPy-style docstring parser.

use pydocstring::parser::numpy::parse_numpy;

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_numpy(docstring).unwrap();

    assert_eq!(result.summary.value, "This is a brief summary.");
    assert!(result.extended_summary.is_none());
    assert!(result.parameters().is_empty());
}

#[test]
fn test_summary_with_description() {
    let docstring = r#"Brief summary.

This is a longer description that provides
more details about the function.
"#;
    let result = parse_numpy(docstring).unwrap();

    assert_eq!(result.summary.value, "Brief summary.");
    assert!(result.extended_summary.is_some());
}

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
    let result = parse_numpy(docstring).unwrap();

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
    let result = parse_numpy(docstring).unwrap();

    assert_eq!(result.parameters().len(), 2);
    assert!(result.parameters()[0].optional.is_none());
    assert!(result.parameters()[1].optional.is_some());
    assert_eq!(result.parameters()[1].param_type.as_ref().map(|t| t.value.as_str()), Some("int"));

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
    let result = parse_numpy(docstring).unwrap();

    assert_eq!(result.raises().len(), 2);
    assert_eq!(result.raises()[0].exception_type.value, "ValueError");
    assert_eq!(result.raises()[1].exception_type.value, "TypeError");
}

#[test]
fn test_with_notes_section() {
    let docstring = r#"Function with notes.

Notes
-----
This is an important note about the function.
"#;
    let result = parse_numpy(docstring).unwrap();

    assert!(result.notes().is_some());
    assert!(result.notes().unwrap().value.contains("important note"));
}
