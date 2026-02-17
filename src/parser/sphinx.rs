//! Sphinx style docstring parser.
//!
//! Parses docstrings in Sphinx format:
//! ```text
//! Brief summary.
//!
//! Extended description.
//!
//! :param param1: Description of param1.
//! :type param1: type
//! :param param2: Description of param2.
//! :type param2: type
//! :return: Description of return value.
//! :rtype: type
//! ```

use crate::error::ParseResult;
use crate::types::SphinxDocstring;

/// Parse a Sphinx-style docstring.
///
/// Note: This is a placeholder implementation. Sphinx style parsing will be implemented
/// after NumPy and Google styles are complete.
pub fn parse_sphinx(input: &str) -> ParseResult<SphinxDocstring> {
    let mut docstring = SphinxDocstring::new();

    // Extract summary (first line)
    if let Some(first_line) = input.lines().next() {
        docstring.summary = first_line.trim().to_string();
    }

    // TODO: Implement full Sphinx-style parsing

    Ok(docstring)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sphinx() {
        let docstring = "Brief description.";
        let result = parse_sphinx(docstring).unwrap();
        assert_eq!(result.summary, "Brief description.");
    }
}
