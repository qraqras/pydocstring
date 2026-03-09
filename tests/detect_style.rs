use pydocstring::parse::{Style, detect_style};

#[test]
fn test_detect_numpy() {
    let input = "Summary.\n\nParameters\n----------\nx : int\n    Desc.";
    assert_eq!(detect_style(input), Style::NumPy);
}

#[test]
fn test_detect_google() {
    let input = "Summary.\n\nArgs:\n    x: Desc.";
    assert_eq!(detect_style(input), Style::Google);
}

#[test]
fn test_detect_plain_defaults_to_google() {
    assert_eq!(detect_style("Just a summary."), Style::Google);
}
