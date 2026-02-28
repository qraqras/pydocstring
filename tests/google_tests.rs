//! Integration tests for Google-style docstring parser.

use pydocstring::GoogleSectionBody;
use pydocstring::google::parse_google;
use pydocstring::google::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleDocstringItem, GoogleException,
    GoogleMethod, GoogleReturns, GoogleSection, GoogleSeeAlsoItem, GoogleWarning,
};
use pydocstring::{LineIndex, TextSize};

// =============================================================================
// Test-local helpers
// =============================================================================

fn all_sections(doc: &GoogleDocstring) -> Vec<&GoogleSection> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .collect()
}

fn args(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Args(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn returns(doc: &GoogleDocstring) -> Option<&GoogleReturns> {
    doc.items.iter().find_map(|item| match item {
        GoogleDocstringItem::Section(s) => match &s.body {
            GoogleSectionBody::Returns(r) => Some(r),
            _ => None,
        },
        _ => None,
    })
}

fn yields(doc: &GoogleDocstring) -> Option<&GoogleReturns> {
    doc.items.iter().find_map(|item| match item {
        GoogleDocstringItem::Section(s) => match &s.body {
            GoogleSectionBody::Yields(r) => Some(r),
            _ => None,
        },
        _ => None,
    })
}

fn raises(doc: &GoogleDocstring) -> Vec<&GoogleException> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn attributes(doc: &GoogleDocstring) -> Vec<&GoogleAttribute> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Attributes(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn keyword_args(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::KeywordArgs(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn other_parameters(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::OtherParameters(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn receives(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Receives(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn warns(doc: &GoogleDocstring) -> Vec<&GoogleWarning> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Warns(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn see_also(doc: &GoogleDocstring) -> Vec<&GoogleSeeAlsoItem> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::SeeAlso(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn methods(doc: &GoogleDocstring) -> Vec<&GoogleMethod> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Methods(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

fn notes(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Notes(v) => Some(v),
            _ => None,
        })
}

fn examples(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Examples(v) => Some(v),
            _ => None,
        })
}

fn todo(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Todo(v) => Some(v),
            _ => None,
        })
}

fn references(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::References(v) => Some(v),
            _ => None,
        })
}

fn warnings(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Warnings(v) => Some(v),
            _ => None,
        })
}

// =============================================================================
// Basic parsing
// =============================================================================

#[test]
fn test_simple_summary() {
    let docstring = "This is a brief summary.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "This is a brief summary."
    );
}

#[test]
fn test_summary_span() {
    let docstring = "Brief description.";
    let result = parse_google(docstring);
    assert_eq!(result.summary.as_ref().unwrap().start(), TextSize::new(0));
    assert_eq!(result.summary.as_ref().unwrap().end(), TextSize::new(18));
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Brief description."
    );
}

#[test]
fn test_empty_docstring() {
    let result = parse_google("");
    assert!(result.summary.is_none());
}

#[test]
fn test_whitespace_only_docstring() {
    let result = parse_google("   \n   \n");
    assert!(result.summary.is_none());
}

#[test]
fn test_summary_with_description() {
    let docstring =
        "Brief summary.\n\nExtended description that provides\nmore details about the function.";
    let result = parse_google(docstring);

    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Brief summary."
    );
    let desc = result.extended_summary.as_ref().unwrap();
    assert_eq!(
        desc.source_text(&result.source),
        "Extended description that provides\nmore details about the function."
    );
}

#[test]
fn test_summary_with_multiline_description() {
    let docstring = r#"Brief summary.

First paragraph of description.

Second paragraph of description."#;
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Brief summary."
    );
    let desc = result.extended_summary.as_ref().unwrap();
    assert!(desc.source_text(&result.source).contains("First paragraph"));
    assert!(
        desc.source_text(&result.source)
            .contains("Second paragraph")
    );
}

#[test]
fn test_multiline_summary() {
    let docstring = "This is a long summary\nthat spans two lines.\n\nExtended description.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "This is a long summary\nthat spans two lines."
    );
    let desc = result.extended_summary.as_ref().unwrap();
    assert_eq!(desc.source_text(&result.source), "Extended description.");
}

#[test]
fn test_multiline_summary_no_extended() {
    let docstring = "Summary line one\ncontinues here.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary line one\ncontinues here."
    );
    assert!(result.extended_summary.is_none());
}

#[test]
fn test_multiline_summary_then_section() {
    let docstring = "Summary line one\ncontinues here.\nArgs:\n    x (int): val";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary line one\ncontinues here."
    );
    assert!(result.extended_summary.is_none());
    assert_eq!(result.items.len(), 1);
}

// =============================================================================
// Args section
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
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
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
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
}

/// Colon with no space after it: `name:description`
#[test]
fn test_args_no_space_after_colon() {
    let docstring = "Summary.\n\nArgs:\n    x:The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
}

/// Colon with extra spaces: `name:   description`
#[test]
fn test_args_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nArgs:\n    x:   The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
}

/// Returns entry with no space after colon.
#[test]
fn test_returns_no_space_after_colon() {
    let docstring = "Summary.\n\nReturns:\n    int:The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(r.description.source_text(&result.source), "The result.");
}

/// Returns entry with extra spaces after colon.
#[test]
fn test_returns_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nReturns:\n    int:   The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(r.description.source_text(&result.source), "The result.");
}

/// Raises entry with no space after colon.
#[test]
fn test_raises_no_space_after_colon() {
    let docstring = "Summary.\n\nRaises:\n    ValueError:If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(r[0].description.source_text(&result.source), "If invalid.");
}

/// Raises entry with extra spaces after colon.
#[test]
fn test_raises_extra_spaces_after_colon() {
    let docstring = "Summary.\n\nRaises:\n    ValueError:   If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(r[0].description.source_text(&result.source), "If invalid.");
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
        args(&result)[0].description.source_text(&result.source),
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
        a[0].description.source_text(&result.source),
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
        a[0].description.source_text(&result.source),
        "Positional args."
    );
    assert_eq!(a[1].name.source_text(&result.source), "**kwargs");
    assert_eq!(
        a[1].description.source_text(&result.source),
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

#[test]
fn test_arguments_alias() {
    let docstring = "Summary.\n\nArguments:\n    x (int): The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
}

// =============================================================================
// Args span accuracy
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
    assert_eq!(a.description.source_text(&result.source), "The value.");
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
    assert_eq!(a.description.source_text(&result.source), "The value.");
}

#[test]
fn test_args_paren_bracket_spans() {
    // Verify that the standard () brackets are also tracked.
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
    assert_eq!(a.description.source_text(&result.source), "The value.");
}

// =============================================================================
// Returns section
// =============================================================================

#[test]
fn test_returns_with_type() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(r.description.source_text(&result.source), "The result.");
}

#[test]
fn test_returns_multiple_lines() {
    let docstring = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    // Only the first line is checked for type: the rest becomes description.
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(
        r.description.source_text(&result.source),
        "The count.\n    str: The message."
    );
}

#[test]
fn test_returns_without_type() {
    let docstring = "Summary.\n\nReturns:\n    The computed result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert!(r.return_type.is_none());
    assert_eq!(
        r.description.source_text(&result.source),
        "The computed result."
    );
}

#[test]
fn test_returns_multiline_description() {
    let docstring = "Summary.\n\nReturns:\n    int: The result\n        of the computation.";
    let result = parse_google(docstring);
    assert_eq!(
        returns(&result)
            .unwrap()
            .description
            .source_text(&result.source),
        "The result\n        of the computation."
    );
}

#[test]
fn test_return_alias() {
    let docstring = "Summary.\n\nReturn:\n    int: The value.";
    let result = parse_google(docstring);
    assert!(returns(&result).is_some());
}

// =============================================================================
// Yields section
// =============================================================================

#[test]
fn test_yields() {
    let docstring = "Summary.\n\nYields:\n    int: The next value.";
    let result = parse_google(docstring);
    let y = yields(&result).unwrap();
    assert_eq!(
        y.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
    assert_eq!(y.description.source_text(&result.source), "The next value.");
}

#[test]
fn test_yield_alias() {
    let docstring = "Summary.\n\nYield:\n    str: Next string.";
    let result = parse_google(docstring);
    assert!(yields(&result).is_some());
}

// =============================================================================
// Raises section
// =============================================================================

#[test]
fn test_raises_single() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the input is invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        r[0].description.source_text(&result.source),
        "If the input is invalid."
    );
}

#[test]
fn test_raises_multiple() {
    let docstring =
        "Summary.\n\nRaises:\n    ValueError: If invalid.\n    TypeError: If wrong type.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(r[1].r#type.source_text(&result.source), "TypeError");
}

#[test]
fn test_raises_multiline_description() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If the\n        input is invalid.";
    let result = parse_google(docstring);
    assert_eq!(
        raises(&result)[0].description.source_text(&result.source),
        "If the\n        input is invalid."
    );
}

#[test]
fn test_raises_exception_type_span() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring);
    assert_eq!(
        raises(&result)[0].r#type.source_text(&result.source),
        "ValueError"
    );
}

// =============================================================================
// Attributes section
// =============================================================================

#[test]
fn test_attributes() {
    let docstring = "Summary.\n\nAttributes:\n    name (str): The name.\n    age (int): The age.";
    let result = parse_google(docstring);
    let a = attributes(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name.source_text(&result.source), "name");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "str"
    );
    assert_eq!(a[1].name.source_text(&result.source), "age");
}

#[test]
fn test_attributes_no_type() {
    let docstring = "Summary.\n\nAttributes:\n    name: The name.";
    let result = parse_google(docstring);
    let a = attributes(&result);
    assert_eq!(a[0].name.source_text(&result.source), "name");
    assert!(a[0].r#type.is_none());
}

// =============================================================================
// Free-text sections
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

#[test]
fn test_references_section() {
    let docstring = "Summary.\n\nReferences:\n    Author, Title, 2024.";
    let result = parse_google(docstring);
    assert!(references(&result).is_some());
}

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

    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Calculate the sum."
    );
    assert!(result.extended_summary.is_some());
    assert_eq!(args(&result).len(), 2);
    assert!(returns(&result).is_some());
    assert_eq!(raises(&result).len(), 1);
    assert!(examples(&result).is_some());
    assert!(notes(&result).is_some());
}

#[test]
fn test_sections_with_blank_lines() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\n    y (str): Name.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring);
    assert_eq!(args(&result).len(), 2);
    assert!(returns(&result).is_some());
}

// =============================================================================
// Section order preservation
// =============================================================================

#[test]
fn test_section_order() {
    let docstring = "Summary.\n\nReturns:\n    int: Value.\n\nArgs:\n    x: Input.";
    let result = parse_google(docstring);
    let sections: Vec<_> = all_sections(&result);
    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[0].header.name.source_text(&result.source),
        "Returns"
    );
    assert_eq!(sections[1].header.name.source_text(&result.source), "Args");
}

#[test]
fn test_section_header_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    let header = &all_sections(&result).into_iter().next().unwrap().header;
    assert_eq!(header.name.source_text(&result.source), "Args");
    assert_eq!(header.name.source_text(&result.source), "Args");
    assert_eq!(header.range.source_text(&result.source), "Args:");
}

#[test]
fn test_section_span() {
    let docstring = "Summary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    let section = all_sections(&result).into_iter().next().unwrap();
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
    let result = parse_google(docstring);
    let sections: Vec<_> = all_sections(&result);
    assert_eq!(sections.len(), 1);
    assert_eq!(
        sections[0].header.name.source_text(&result.source),
        "Custom"
    );
    match &sections[0].body {
        GoogleSectionBody::Unknown(text) => {
            assert_eq!(text.source_text(&result.source), "Some custom content.");
        }
        _ => panic!("Expected Unknown section body"),
    }
}

#[test]
fn test_unknown_section_with_known() {
    let docstring =
        "Summary.\n\nArgs:\n    x: Value.\n\nCustom:\n    Content.\n\nReturns:\n    int: Result.";
    let result = parse_google(docstring);
    let sections: Vec<_> = all_sections(&result);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].header.name.source_text(&result.source), "Args");
    assert_eq!(
        sections[1].header.name.source_text(&result.source),
        "Custom"
    );
    assert_eq!(
        sections[2].header.name.source_text(&result.source),
        "Returns"
    );
    // Known sections still accessible via helpers
    assert_eq!(args(&result).len(), 1);
    assert!(returns(&result).is_some());
}

#[test]
fn test_multiple_unknown_sections() {
    let docstring = "Summary.\n\nCustom One:\n    First.\n\nCustom Two:\n    Second.";
    let result = parse_google(docstring);
    let sections: Vec<_> = all_sections(&result);
    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[0].header.name.source_text(&result.source),
        "Custom One"
    );
    assert_eq!(
        sections[1].header.name.source_text(&result.source),
        "Custom Two"
    );
}

// =============================================================================
// Indented docstring (non-zero base indent)
// =============================================================================

#[test]
fn test_indented_docstring() {
    let docstring = "    Summary.\n\n    Args:\n        x (int): Value.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary."
    );
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

#[test]
fn test_indented_summary_span() {
    let docstring = "    Summary.";
    let result = parse_google(docstring);
    assert_eq!(result.summary.as_ref().unwrap().start(), TextSize::new(4));
    assert_eq!(result.summary.as_ref().unwrap().end(), TextSize::new(12));
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary."
    );
}

// =============================================================================
// Convenience accessors
// =============================================================================

#[test]
fn test_docstring_like_summary() {
    let docstring = "Summary.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary."
    );
}

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

#[test]
fn test_docstring_like_returns() {
    let docstring = "Summary.\n\nReturns:\n    int: The result.";
    let result = parse_google(docstring);
    let r = returns(&result).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

#[test]
fn test_docstring_like_raises() {
    let docstring = "Summary.\n\nRaises:\n    ValueError: If bad.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
}

// =============================================================================
// Span round-trip
// =============================================================================

#[test]
fn test_span_source_text_round_trip() {
    let docstring = "Summary.\n\nArgs:\n    x (int): Value.\n\nReturns:\n    bool: Success.";
    let result = parse_google(docstring);

    // Summary
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary."
    );

    // Arg name
    assert_eq!(args(&result)[0].name.source_text(&result.source), "x");

    // Arg type
    assert_eq!(
        args(&result)[0]
            .r#type
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "int"
    );

    // Return type
    assert_eq!(
        returns(&result)
            .unwrap()
            .return_type
            .as_ref()
            .unwrap()
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
    let result = parse_google(docstring);
    // "Args:" at base_indent=0 is a section header, so summary remains empty
    assert_eq!(args(&result).len(), 1);
}

#[test]
fn test_leading_blank_lines() {
    let docstring = "\n\n\nSummary.\n\nArgs:\n    x: Value.";
    let result = parse_google(docstring);
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Summary."
    );
    assert_eq!(args(&result).len(), 1);
}

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
// Napoleon: Parameters / Params aliases
// =============================================================================

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
// Napoleon: Keyword Args section
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
// Napoleon: Other Parameters section
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
// Napoleon: Receives section
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
// Napoleon: Raise alias
// =============================================================================

#[test]
fn test_raise_alias() {
    let docstring = "Summary.\n\nRaise:\n    ValueError: If invalid.";
    let result = parse_google(docstring);
    let r = raises(&result);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Raise"
    );
}

// =============================================================================
// Napoleon: Warns section
// =============================================================================

#[test]
fn test_warns_basic() {
    let docstring = "Summary.\n\nWarns:\n    DeprecationWarning: If using old API.";
    let result = parse_google(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 1);
    assert_eq!(
        w[0].warning_type.source_text(&result.source),
        "DeprecationWarning"
    );
    assert_eq!(
        w[0].description.source_text(&result.source),
        "If using old API."
    );
}

#[test]
fn test_warns_multiple() {
    let docstring =
        "Summary.\n\nWarns:\n    DeprecationWarning: Old API.\n    UserWarning: Bad config.";
    let result = parse_google(docstring);
    let w = warns(&result);
    assert_eq!(w.len(), 2);
    assert_eq!(
        w[0].warning_type.source_text(&result.source),
        "DeprecationWarning"
    );
    assert_eq!(w[1].warning_type.source_text(&result.source), "UserWarning");
}

#[test]
fn test_warn_alias() {
    let docstring = "Summary.\n\nWarn:\n    FutureWarning: Will change.";
    let result = parse_google(docstring);
    assert_eq!(warns(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Warn"
    );
}

#[test]
fn test_warns_multiline_description() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: First line.\n        Second line.";
    let result = parse_google(docstring);
    assert_eq!(
        warns(&result)[0].description.source_text(&result.source),
        "First line.\n        Second line."
    );
}

#[test]
fn test_warns_section_body_variant() {
    let docstring = "Summary.\n\nWarns:\n    UserWarning: Desc.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
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
    let result = parse_google(docstring);
    assert_eq!(
        warnings(&result).unwrap().source_text(&result.source),
        "This is deprecated."
    );
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Warning"
    );
}

// =============================================================================
// Napoleon: Attribute alias
// =============================================================================

#[test]
fn test_attribute_singular_alias() {
    let docstring = "Summary.\n\nAttribute:\n    name (str): The name.";
    let result = parse_google(docstring);
    assert_eq!(attributes(&result).len(), 1);
    assert_eq!(
        all_sections(&result)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Attribute"
    );
}

// =============================================================================
// Napoleon: Methods section
// =============================================================================

#[test]
fn test_methods_basic() {
    let docstring = "Summary.\n\nMethods:\n    reset(): Reset the state.\n    update(data): Update with new data.";
    let result = parse_google(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].name.source_text(&result.source), "reset()");
    assert_eq!(
        m[0].description.source_text(&result.source),
        "Reset the state."
    );
    assert_eq!(m[1].name.source_text(&result.source), "update(data)");
}

#[test]
fn test_methods_without_parens() {
    let docstring = "Summary.\n\nMethods:\n    do_stuff: Performs the operation.";
    let result = parse_google(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].name.source_text(&result.source), "do_stuff");
    assert_eq!(
        m[0].description.source_text(&result.source),
        "Performs the operation."
    );
}

#[test]
fn test_methods_section_body_variant() {
    let docstring = "Summary.\n\nMethods:\n    foo(): Does bar.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
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
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 1);
    assert_eq!(sa[0].names.len(), 1);
    assert_eq!(sa[0].names[0].source_text(&result.source), "other_func");
    assert_eq!(
        sa[0]
            .description
            .as_ref()
            .unwrap()
            .source_text(&result.source),
        "Does something else."
    );
}

#[test]
fn test_see_also_multiple_names() {
    let docstring = "Summary.\n\nSee Also:\n    func_a, func_b, func_c";
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 1);
    assert_eq!(sa[0].names.len(), 3);
    assert_eq!(sa[0].names[0].source_text(&result.source), "func_a");
    assert_eq!(sa[0].names[1].source_text(&result.source), "func_b");
    assert_eq!(sa[0].names[2].source_text(&result.source), "func_c");
    assert!(sa[0].description.is_none());
}

#[test]
fn test_see_also_mixed() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Description.\n    func_b, func_c";
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 2);
    assert_eq!(sa[0].names[0].source_text(&result.source), "func_a");
    assert!(sa[0].description.is_some());
    assert_eq!(sa[1].names.len(), 2);
    assert!(sa[1].description.is_none());
}

#[test]
fn test_see_also_section_body_variant() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Desc.";
    let result = parse_google(docstring);
    match &all_sections(&result).into_iter().next().unwrap().body {
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

// =============================================================================
// Napoleon: Case-insensitive section headers
// =============================================================================

#[test]
fn test_napoleon_case_insensitive() {
    let docstring = "Summary.\n\nkeyword args:\n    x (int): Value.";
    let result = parse_google(docstring);
    assert_eq!(keyword_args(&result).len(), 1);
}

#[test]
fn test_see_also_case_insensitive() {
    let docstring = "Summary.\n\nsee also:\n    func_a: Description.";
    let result = parse_google(docstring);
    assert_eq!(see_also(&result).len(), 1);
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
    assert_eq!(
        result.summary.as_ref().unwrap().source_text(&result.source),
        "Calculate something."
    );
    assert!(result.extended_summary.is_some());
    assert_eq!(args(&result).len(), 1);
    assert_eq!(keyword_args(&result).len(), 1);
    assert!(returns(&result).is_some());
    assert_eq!(raises(&result).len(), 1);
    assert_eq!(warns(&result).len(), 1);
    assert_eq!(see_also(&result).len(), 1);
    assert!(notes(&result).is_some());
    assert!(examples(&result).is_some());
}

// =============================================================================
// Space-before-colon and colonless header tests
// =============================================================================

/// `Args :` (space before colon) should be dispatched as Args, not Unknown.
#[test]
fn test_section_header_space_before_colon() {
    let input = "Summary.\n\nArgs :\n    x (int): The value.";
    let result = parse_google(input);
    let doc = &result;
    let a = args(doc);
    assert_eq!(a.len(), 1, "expected 1 arg from 'Args :'");
    assert_eq!(a[0].name.source_text(&result.source), "x");

    // Header name should be "Args" (trimmed), not "Args "
    assert_eq!(
        all_sections(doc)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Args"
    );
    // Colon should still be present
    assert!(
        all_sections(doc)
            .into_iter()
            .next()
            .unwrap()
            .header
            .colon
            .is_some()
    );
}

/// `Returns :` with space before colon.
#[test]
fn test_returns_space_before_colon() {
    let input = "Summary.\n\nReturns :\n    int: The result.";
    let result = parse_google(input);
    let doc = &result;
    let r = returns(doc).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

/// Colonless `Args` should be parsed as Args section.
#[test]
fn test_section_header_no_colon() {
    let input = "Summary.\n\nArgs\n    x (int): The value.";
    let result = parse_google(input);
    let doc = &result;
    let a = args(doc);
    assert_eq!(a.len(), 1, "expected 1 arg from colonless 'Args'");
    assert_eq!(a[0].name.source_text(&result.source), "x");

    // Header name should be "Args"
    assert_eq!(
        all_sections(doc)
            .into_iter()
            .next()
            .unwrap()
            .header
            .name
            .source_text(&result.source),
        "Args"
    );
    // Colon should be None
    assert!(
        all_sections(doc)
            .into_iter()
            .next()
            .unwrap()
            .header
            .colon
            .is_none()
    );
}

/// Missing colon on section header should emit a diagnostic.
/// Colonless `Returns` should be parsed as Returns section.
#[test]
fn test_returns_no_colon() {
    let input = "Summary.\n\nReturns\n    int: The result.";
    let result = parse_google(input);
    let doc = &result;
    let r = returns(doc).unwrap();
    assert_eq!(
        r.return_type.as_ref().unwrap().source_text(&result.source),
        "int"
    );
}

/// Colonless `Raises` should be parsed as Raises section.
#[test]
fn test_raises_no_colon() {
    let input = "Summary.\n\nRaises\n    ValueError: If invalid.";
    let result = parse_google(input);
    let doc = &result;
    let r = raises(doc);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
}

/// Multiline type annotation spanning multiple lines in Args section.
#[test]
fn test_args_multiline_type() {
    let docstring = "Summary.\n\nArgs:\n    x (Dict[str,\n            int]): The value.";
    let result = parse_google(docstring);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(
        a[0].r#type.as_ref().unwrap().source_text(&result.source),
        "Dict[str,\n            int]"
    );
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
}

/// Unknown names without colon should NOT be treated as headers.
#[test]
fn test_unknown_name_without_colon_not_header() {
    let input = "Summary.\n\nSomeWord\n    x (int): value.";
    let result = parse_google(input);
    let doc = &result;
    // "SomeWord" is not a known section name, so it becomes extended description
    assert!(
        all_sections(doc).is_empty(),
        "unknown colonless name should not become a section"
    );
}

/// Multiple sections with mixed colon styles.
#[test]
fn test_mixed_colon_styles() {
    let input = "Summary.\n\nArgs:\n    x: value.\n\nReturns\n    int: result.\n\nRaises :\n    ValueError: If bad.";
    let result = parse_google(input);
    let doc = &result;
    assert_eq!(args(doc).len(), 1);
    assert!(returns(doc).is_some());
    assert_eq!(raises(doc).len(), 1);
}

// =============================================================================
// Tab indentation tests
// =============================================================================

/// Args section with tab-indented entries.
#[test]
fn test_tab_indented_args() {
    let input = "Summary.\n\nArgs:\n\tx: The value.\n\ty: Another value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    assert_eq!(a[0].description.source_text(&result.source), "The value.");
    assert_eq!(a[1].name.source_text(&result.source), "y");
    assert_eq!(
        a[1].description.source_text(&result.source),
        "Another value."
    );
}

/// Args entries with tab indent and descriptions with deeper tab+space indent.
#[test]
fn test_tab_args_with_continuation() {
    let input = "Summary.\n\nArgs:\n\tx: First line.\n\t    Continuation.\n\ty: Second.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name.source_text(&result.source), "x");
    let desc = a[0].description.source_text(&result.source);
    assert!(desc.contains("First line."), "desc = {:?}", desc);
    assert!(desc.contains("Continuation."), "desc = {:?}", desc);
}

/// Returns section with tab-indented entry.
#[test]
fn test_tab_indented_returns() {
    let input = "Summary.\n\nReturns:\n\tint: The result.";
    let result = parse_google(input);
    let r = returns(&result);
    assert!(r.is_some());
    let r = r.unwrap();
    assert_eq!(r.return_type.unwrap().source_text(&result.source), "int");
    assert_eq!(r.description.source_text(&result.source), "The result.");
}

/// Raises section with tab-indented entries.
#[test]
fn test_tab_indented_raises() {
    let input = "Summary.\n\nRaises:\n\tValueError: If bad.\n\tTypeError: If wrong type.";
    let result = parse_google(input);
    let r = raises(&result);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].r#type.source_text(&result.source), "ValueError");
    assert_eq!(r[1].r#type.source_text(&result.source), "TypeError");
}

/// Section header detection with tab indentation matches.
#[test]
fn test_tab_indented_section_header() {
    // Section header at tab indent (4 cols), entry at tab+spaces (>4 cols)
    let input = "\tSummary.\n\n\tArgs:\n\t\tx: The value.";
    let result = parse_google(input);
    let a = args(&result);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name.source_text(&result.source), "x");
}
