//! Top-level style detection.
//!
//! This module provides [`detect_style`] for automatic style detection.

use crate::ast::Style;

// =============================================================================
// Style detection
// =============================================================================

/// Detect the docstring style from its content.
///
/// Uses heuristics to identify the style:
/// 1. **NumPy**: Section headers followed by `---` underlines
/// 2. **Google**: Section headers ending with `:` (e.g., `Args:`, `Returns:`)
/// 3. Falls back to `Google` if no style-specific patterns are found
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
/// let google = "Summary.\n\nArgs:\n    x: Description.";
/// assert_eq!(detect_style(google), Style::Google);
/// ```
pub fn detect_style(input: &str) -> Style {
    if has_numpy_sections(input) {
        return Style::NumPy;
    }
    if has_google_sections(input) {
        return Style::Google;
    }
    Style::Google
}

// =============================================================================
// Style detection helpers
// =============================================================================

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
    use crate::ast::Style;

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
}
