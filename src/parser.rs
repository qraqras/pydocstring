//! Top-level parsing interface and style detection.
//!
//! This module provides:
//! - [`detect_style`] for automatic style detection
//! - [`parse`] for automatic parsing

use crate::ast::{Docstring, Style};
use crate::error::ParseResult;

// =============================================================================
// Style detection
// =============================================================================

/// Detect the docstring style from its content.
///
/// Uses heuristics to identify the style:
/// 1. **Sphinx**: `:param `, `:type `, `:returns:`, `:rtype:` field markers
/// 2. **NumPy**: Section headers followed by `---` underlines
/// 3. **Google**: Section headers ending with `:` (e.g., `Args:`, `Returns:`)
/// 4. Falls back to `Google` if no style-specific patterns are found
///
/// # Example
///
/// ```rust
/// use pydocstring::detect_style;
/// use pydocstring::Style;
///
/// let numpy = "Summary.\n\nParameters\n----------\nx : int\n    Description.";
/// assert_eq!(detect_style(numpy), Style::NumPy);
///
/// let sphinx = "Summary.\n\n:param x: Description.";
/// assert_eq!(detect_style(sphinx), Style::Sphinx);
///
/// let google = "Summary.\n\nArgs:\n    x: Description.";
/// assert_eq!(detect_style(google), Style::Google);
/// ```
pub fn detect_style(input: &str) -> Style {
    if has_sphinx_markers(input) {
        return Style::Sphinx;
    }
    if has_numpy_sections(input) {
        return Style::NumPy;
    }
    if has_google_sections(input) {
        return Style::Google;
    }
    Style::Google
}

/// Parse a docstring with automatic style detection.
///
/// Detects the style using [`detect_style`], then delegates to the
/// appropriate style-specific parser.
///
/// # Example
///
/// ```rust
/// use pydocstring::{parse, DocstringLike, Style};
///
/// let input = "Summary.\n\nParameters\n----------\nx : int\n    The value.";
/// let result = parse(input);
/// let doc = &result.value;
///
/// assert_eq!(doc.style(), Style::NumPy);
/// assert_eq!(doc.summary(), "Summary.");
/// assert_eq!(doc.parameters().len(), 1);
/// ```
pub fn parse(input: &str) -> ParseResult<Docstring> {
    match detect_style(input) {
        Style::NumPy => crate::styles::numpy::parse_numpy(input).map(Docstring::NumPy),
        Style::Google => crate::styles::google::parse_google(input).map(Docstring::Google),
        Style::Sphinx => crate::styles::sphinx::parse_sphinx(input).map(Docstring::Sphinx),
    }
}

// =============================================================================
// Style detection helpers
// =============================================================================

fn has_sphinx_markers(input: &str) -> bool {
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(":param ")
            || trimmed.starts_with(":type ")
            || trimmed.starts_with(":returns:")
            || trimmed.starts_with(":return:")
            || trimmed.starts_with(":rtype:")
            || trimmed.starts_with(":raises ")
            || trimmed.starts_with(":raise ")
            || trimmed.starts_with(":var ")
            || trimmed.starts_with(":ivar ")
            || trimmed.starts_with(":cvar ")
        {
            return true;
        }
    }
    false
}

fn has_numpy_sections(input: &str) -> bool {
    let lines: Vec<&str> = input.lines().collect();
    for i in 0..lines.len().saturating_sub(1) {
        let current = lines[i].trim();
        let next = lines[i + 1].trim();
        if !current.is_empty()
            && !next.is_empty()
            && next.len() >= 3
            && next.chars().all(|c| c == '-')
        {
            return true;
        }
    }
    false
}

const GOOGLE_SECTIONS: &[&str] = &[
    "Args:",
    "Arguments:",
    "Returns:",
    "Return:",
    "Raises:",
    "Yields:",
    "Yield:",
    "Example:",
    "Examples:",
    "Note:",
    "Notes:",
    "Attributes:",
    "Todo:",
    "References:",
    "Warnings:",
];

fn has_google_sections(input: &str) -> bool {
    for line in input.lines() {
        let trimmed = line.trim();
        if GOOGLE_SECTIONS.contains(&trimmed) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::DocstringLike;
    use crate::ast::Style;

    #[test]
    fn test_detect_sphinx() {
        assert_eq!(detect_style(":param x: value"), Style::Sphinx);
        assert_eq!(detect_style("Summary.\n\n:type x: int"), Style::Sphinx);
        assert_eq!(detect_style("Summary.\n\n:returns: value"), Style::Sphinx);
        assert_eq!(detect_style("Summary.\n\n:rtype: int"), Style::Sphinx);
    }

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

    #[test]
    fn test_parse_auto_numpy() {
        let input = "Summary.\n\nParameters\n----------\nx : int\n    Desc.";
        let doc = &parse(input).value;
        assert_eq!(doc.style(), Style::NumPy);
        assert_eq!(doc.summary(), "Summary.");
    }

    #[test]
    fn test_parse_auto_google() {
        let input = "Summary.";
        let doc = &parse(input).value;
        assert_eq!(doc.style(), Style::Google);
        assert_eq!(doc.summary(), "Summary.");
    }

    #[test]
    fn test_parse_auto_sphinx() {
        let input = "Summary.\n\n:param x: Desc.";
        let doc = &parse(input).value;
        assert_eq!(doc.style(), Style::Sphinx);
        assert_eq!(doc.summary(), "Summary.");
    }
}
