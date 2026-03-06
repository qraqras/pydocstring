//! Docstring style implementations.
//!
//! Each sub-module provides an AST and parser for its respective style.
//! This module also provides [`detect_style`] for automatic style detection.

use core::fmt;

use google::GoogleSectionKind;

pub mod google;
pub mod numpy;
pub(crate) mod utils;

// =============================================================================
// Style
// =============================================================================

/// Docstring style identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// NumPy style (section headers with underlines).
    NumPy,
    /// Google style (section headers with colons).
    Google,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::NumPy => write!(f, "numpy"),
            Style::Google => write!(f, "google"),
        }
    }
}

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

fn has_google_sections(input: &str) -> bool {
    for line in input.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_suffix(':') {
            if GoogleSectionKind::is_known(&name.to_ascii_lowercase()) {
                return true;
            }
        }
    }
    false
}
