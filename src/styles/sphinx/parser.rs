//! Sphinx style docstring parser.
//!
//! **Note:** Sphinx style is not supported in v1. This module provides a
//! minimal placeholder that extracts only the summary line and emits an
//! error diagnostic. Full Sphinx support is planned for a future release.
//!
//! Sphinx format reference:
//! ```text
//! Brief summary.
//!
//! :param param1: Description of param1.
//! :type param1: type
//! :returns: Description of return value.
//! :rtype: type
//! ```

use crate::ast::Spanned;
use crate::error::{Diagnostic, ParseResult, Severity};
use crate::styles::sphinx::ast::SphinxDocstring;

/// Parse a Sphinx-style docstring.
///
/// **Sphinx style is not supported in v1.** This function extracts only the
/// summary line and returns an error diagnostic indicating that Sphinx style
/// is unsupported. Use [`crate::styles::google::parse_google`] or
/// [`crate::styles::numpy::parse_numpy`] for fully supported styles.
pub fn parse_sphinx(input: &str) -> ParseResult<SphinxDocstring> {
    let mut docstring = SphinxDocstring::new();
    docstring.source = input.to_string();

    // Extract summary (first line) as a best-effort fallback
    if let Some(first_line) = input.lines().next() {
        docstring.summary = Spanned::dummy(first_line.trim().to_string());
    }

    let mut result = ParseResult::ok(docstring);
    result.diagnostics.push(Diagnostic::new(
        crate::ast::TextRange::empty(),
        Severity::Error,
        "Sphinx style is not supported in this version. Use Google or NumPy style instead.",
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Severity;

    #[test]
    fn test_parse_sphinx_unsupported_diagnostic() {
        let result = parse_sphinx("Brief description.\n\n:param x: Desc.");
        assert_eq!(result.value.summary.value, "Brief description.");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("not supported"));
    }
}
