use super::*;

// =============================================================================
// Attributes section
// =============================================================================

#[test]
fn test_attributes() {
    let docstring = "Summary.\n\nAttributes:\n    name (str): The name.\n    age (int): The age.";
    let result = parse_google(docstring);
    let a = attributes(&result);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].name().text(result.source()), "name");
    assert_eq!(a[0].r#type().unwrap().text(result.source()), "str");
    assert_eq!(a[1].name().text(result.source()), "age");
}

#[test]
fn test_attributes_no_type() {
    let docstring = "Summary.\n\nAttributes:\n    name: The name.";
    let result = parse_google(docstring);
    let a = attributes(&result);
    assert_eq!(a[0].name().text(result.source()), "name");
    assert!(a[0].r#type().is_none());
}

#[test]
fn test_attribute_singular_alias() {
    let docstring = "Summary.\n\nAttribute:\n    name (str): The name.";
    let result = parse_google(docstring);
    assert_eq!(attributes(&result).len(), 1);
    assert_eq!(
        all_sections(&result)[0].header().name().text(result.source()),
        "Attribute"
    );
}

// =============================================================================
// Methods section
// =============================================================================

#[test]
fn test_methods_basic() {
    let docstring = "Summary.\n\nMethods:\n    reset(): Reset the state.\n    update(data): Update with new data.";
    let result = parse_google(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].name().text(result.source()), "reset()");
    assert_eq!(m[0].description().unwrap().text(result.source()), "Reset the state.");
    assert_eq!(m[1].name().text(result.source()), "update(data)");
}

#[test]
fn test_methods_without_parens() {
    let docstring = "Summary.\n\nMethods:\n    do_stuff: Performs the operation.";
    let result = parse_google(docstring);
    let m = methods(&result);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].name().text(result.source()), "do_stuff");
    assert_eq!(
        m[0].description().unwrap().text(result.source()),
        "Performs the operation."
    );
}

#[test]
fn test_methods_section_body_variant() {
    let docstring = "Summary.\n\nMethods:\n    foo(): Does bar.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections[0].section_kind(result.source()), GoogleSectionKind::Methods);
    assert_eq!(sections[0].methods().count(), 1);
}

// =============================================================================
// See Also section
// =============================================================================

#[test]
fn test_see_also_basic() {
    let docstring = "Summary.\n\nSee Also:\n    other_func: Does something else.";
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 1);
    let names: Vec<_> = sa[0].names().collect();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].text(result.source()), "other_func");
    assert_eq!(
        sa[0].description().unwrap().text(result.source()),
        "Does something else."
    );
}

#[test]
fn test_see_also_multiple_names() {
    let docstring = "Summary.\n\nSee Also:\n    func_a, func_b, func_c";
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 1);
    let names: Vec<_> = sa[0].names().collect();
    assert_eq!(names.len(), 3);
    assert_eq!(names[0].text(result.source()), "func_a");
    assert_eq!(names[1].text(result.source()), "func_b");
    assert_eq!(names[2].text(result.source()), "func_c");
    assert!(sa[0].description().is_none());
}

#[test]
fn test_see_also_mixed() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Description.\n    func_b, func_c";
    let result = parse_google(docstring);
    let sa = see_also(&result);
    assert_eq!(sa.len(), 2);
    let names0: Vec<_> = sa[0].names().collect();
    assert_eq!(names0[0].text(result.source()), "func_a");
    assert!(sa[0].description().is_some());
    let names1: Vec<_> = sa[1].names().collect();
    assert_eq!(names1.len(), 2);
    assert!(sa[1].description().is_none());
}

#[test]
fn test_see_also_section_body_variant() {
    let docstring = "Summary.\n\nSee Also:\n    func_a: Desc.";
    let result = parse_google(docstring);
    let sections = all_sections(&result);
    assert_eq!(sections[0].section_kind(result.source()), GoogleSectionKind::SeeAlso);
    assert_eq!(sections[0].see_also_items().count(), 1);
}
