//! NumPy style docstring parser.
//!
//! Parses docstrings in NumPy format:
//! ```text
//! Brief summary.
//!
//! Extended description.
//!
//! Parameters
//! ----------
//! param1 : type
//!     Description of param1.
//! param2 : type, optional
//!     Description of param2.
//!
//! Returns
//! -------
//! type
//!     Description of return value.
//! ```

use crate::error::ParseResult;
use crate::span::Span;
use crate::types::{NumPyDocstring, NumPyException, NumPyParameter, NumPyReturns};

/// Parse a NumPy-style docstring.
pub fn parse_numpy(input: &str) -> ParseResult<NumPyDocstring> {
    let mut docstring = NumPyDocstring::new();
    let lines: Vec<&str> = input.lines().collect();

    if lines.is_empty() {
        return Ok(docstring);
    }

    let mut i = 0;

    // Parse summary (first non-empty line)
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    if i < lines.len() {
        let first_line = lines[i].trim();

        // Check if first line is a function signature (contains parentheses)
        // Signature format: "function_name(args)"
        if first_line.contains('(') && first_line.ends_with(')') {
            // Check if next line is empty or a section header
            let next_i = i + 1;
            let is_signature = if next_i < lines.len() {
                let next_line = lines[next_i].trim();
                next_line.is_empty() || is_section_header(&lines, next_i)
            } else {
                next_i >= lines.len() // Last line
            };

            if is_signature {
                docstring.signature = Some(first_line.to_string());
                i += 1;
            } else {
                docstring.summary = first_line.to_string();
                i += 1;
            }
        } else {
            docstring.summary = first_line.to_string();
            i += 1;
        }
    }

    // Skip empty lines
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    // If we found a signature, the next non-empty line is the summary
    if docstring.signature.is_some() && i < lines.len() {
        let summary_line = lines[i].trim();
        if !summary_line.is_empty() && !is_section_header(&lines, i) {
            docstring.summary = summary_line.to_string();
            i += 1;
        }
    }

    // Skip empty lines again
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    // Parse description until we hit a section header
    let mut description_lines = Vec::new();
    while i < lines.len() {
        let line = lines[i].trim();
        if is_section_header(&lines, i) {
            break;
        }
        if !line.is_empty() {
            description_lines.push(line);
        }
        i += 1;
    }

    if !description_lines.is_empty() {
        docstring.extended_summary = Some(description_lines.join(" "));
    }

    // Parse sections
    while i < lines.len() {
        if is_section_header(&lines, i) {
            let section_name = lines[i].trim();
            i += 2; // Skip header and underline

            match section_name {
                "Parameters" => {
                    let (params, next_i) = parse_parameters(&lines, i)?;
                    docstring.parameters = params;
                    i = next_i;
                }
                "Returns" => {
                    let (returns, next_i) = parse_returns(&lines, i)?;
                    docstring.returns = returns;
                    i = next_i;
                }
                "Raises" => {
                    let (raises, next_i) = parse_raises(&lines, i)?;
                    docstring.raises = raises;
                    i = next_i;
                }
                "Yields" => {
                    let (yields, next_i) = parse_returns(&lines, i)?;
                    docstring.yields = yields;
                    i = next_i;
                }
                "Notes" => {
                    let content = parse_section_content(&lines, i)?;
                    docstring.notes = Some(content.0);
                    i = content.1;
                }
                "Examples" => {
                    let content = parse_section_content(&lines, i)?;
                    docstring.examples = Some(content.0);
                    i = content.1;
                }
                "Warnings" => {
                    let content = parse_section_content(&lines, i)?;
                    docstring.warnings = Some(content.0);
                    i = content.1;
                }
                _ => {
                    // Skip unknown sections for now
                    i = skip_section(&lines, i);
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(docstring)
}

/// Check if current line is a section header (followed by dashes).
fn is_section_header(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() {
        return false;
    }

    let current = lines[index].trim();
    let next = lines[index + 1].trim();

    !current.is_empty() && !next.is_empty() && next.chars().all(|c| c == '-')
}

/// Parse the Parameters section.
fn parse_parameters(lines: &[&str], start: usize) -> ParseResult<(Vec<NumPyParameter>, usize)> {
    let mut parameters = Vec::new();
    let mut i = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i].trim();

        // Check if this is a parameter definition (not indented or lightly indented)
        if !line.is_empty() && !lines[i].starts_with("    ") {
            // Parse parameter: "name : type" or "name : type, optional"
            if let Some((name_type, _)) = parse_parameter_header(line) {
                let (names, param_type, optional, default_val) = parse_name_and_type(&name_type);

                // Collect description lines
                i += 1;
                let mut desc_lines = Vec::new();
                while i < lines.len() {
                    let desc_line = lines[i];
                    if is_section_header(lines, i)
                        || (!desc_line.trim().is_empty() && !desc_line.starts_with("    "))
                    {
                        break;
                    }
                    if !desc_line.trim().is_empty() {
                        desc_lines.push(desc_line.trim());
                    }
                    i += 1;
                }

                parameters.push(NumPyParameter {
                    span: Span::empty(),
                    names,
                    param_type,
                    description: desc_lines.join(" "),
                    optional,
                    default: default_val,
                });
                continue;
            }
        }

        i += 1;
    }

    Ok((parameters, i))
}

/// Parse parameter header line.
fn parse_parameter_header(line: &str) -> Option<(String, usize)> {
    if line.contains(':') {
        Some((line.to_string(), 0))
    } else {
        None
    }
}

/// Parse parameter name and type from "name : type" or "name : type, optional".
/// Also extracts default values like "default True" or "default=True".
fn parse_name_and_type(text: &str) -> (Vec<String>, Option<String>, bool, Option<String>) {
    let parts: Vec<&str> = text.splitn(2, ':').collect();

    // Parse names (can be multiple like "x1, x2")
    let names: Vec<String> = parts[0]
        .trim()
        .split(',')
        .map(|n| n.trim().to_string())
        .collect();

    if parts.len() < 2 {
        return (names, None, false, None);
    }

    let type_part = parts[1].trim();
    let optional = type_part.contains("optional");

    // Extract default value (e.g., "default True", "default=True", "default: True")
    let mut default_val = None;
    let mut clean_type = type_part;

    if let Some(default_pos) = type_part.find("default") {
        let after_default = &type_part[default_pos + 7..].trim_start();
        // Remove leading '=' or ':' if present
        let after_default = after_default
            .strip_prefix('=')
            .or_else(|| after_default.strip_prefix(':'))
            .unwrap_or(after_default)
            .trim_start();

        // Find where the default value ends (before space or comma)
        let end = after_default
            .find(|c: char| c.is_whitespace() || c == ',')
            .unwrap_or(after_default.len());
        default_val = Some(after_default[..end].to_string());

        // Extract type without default part
        clean_type = &type_part[..default_pos].trim();
    }

    let param_type = if optional {
        Some(
            clean_type
                .replace(", optional", "")
                .replace(",optional", "")
                .trim()
                .to_string(),
        )
    } else {
        if clean_type.is_empty() {
            None
        } else {
            Some(clean_type.to_string())
        }
    };

    (names, param_type, optional, default_val)
}

/// Parse the Returns section.
fn parse_returns(lines: &[&str], start: usize) -> ParseResult<(Vec<NumPyReturns>, usize)> {
    let mut i = start;
    let mut return_type = None;
    let mut desc_lines = Vec::new();

    // First non-empty line should be the type
    while i < lines.len() {
        let line = lines[i].trim();
        if is_section_header(lines, i) {
            break;
        }

        if !line.is_empty() {
            if return_type.is_none() && !lines[i].starts_with("    ") {
                return_type = Some(line.to_string());
            } else if lines[i].starts_with("    ") {
                desc_lines.push(line);
            }
        }

        i += 1;
    }

    let returns = if return_type.is_some() || !desc_lines.is_empty() {
        vec![NumPyReturns {
            span: Span::empty(),
            name: None,
            return_type,
            description: desc_lines.join(" "),
        }]
    } else {
        Vec::new()
    };

    Ok((returns, i))
}

/// Parse the Raises section.
fn parse_raises(lines: &[&str], start: usize) -> ParseResult<(Vec<NumPyException>, usize)> {
    let mut raises = Vec::new();
    let mut i = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i].trim();

        if !line.is_empty() && !lines[i].starts_with("    ") {
            let exception_type = line.to_string();

            i += 1;
            let mut desc_lines = Vec::new();
            while i < lines.len() {
                let desc_line = lines[i];
                if is_section_header(lines, i)
                    || (!desc_line.trim().is_empty() && !desc_line.starts_with("    "))
                {
                    break;
                }
                if !desc_line.trim().is_empty() {
                    desc_lines.push(desc_line.trim());
                }
                i += 1;
            }

            raises.push(NumPyException {
                span: Span::empty(),
                exception_type,
                description: desc_lines.join(" "),
            });
            continue;
        }

        i += 1;
    }

    Ok((raises, i))
}

/// Parse section content.
fn parse_section_content(lines: &[&str], start: usize) -> ParseResult<(String, usize)> {
    let mut content_lines = Vec::new();
    let mut i = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i].trim();
        if !line.is_empty() {
            content_lines.push(line);
        }

        i += 1;
    }

    Ok((content_lines.join("\n"), i))
}

/// Skip a section we don't parse.
fn skip_section(lines: &[&str], start: usize) -> usize {
    let mut i = start;
    while i < lines.len() && !is_section_header(lines, i) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_numpy() {
        let docstring = "Brief description.";
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.summary, "Brief description.");
    }

    #[test]
    fn test_parse_with_parameters() {
        let docstring = r#"Brief description.

Parameters
----------
x : int
    The first parameter.
y : str, optional
    The second parameter.
"#;
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.summary, "Brief description.");
        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].names[0], "x");
        assert_eq!(result.parameters[0].param_type, Some("int".to_string()));
        assert!(!result.parameters[0].optional);
        assert_eq!(result.parameters[1].names[0], "y");
        assert!(result.parameters[1].optional);
    }

    #[test]
    fn test_parse_with_signature() {
        let docstring = r#"add(a, b)

The sum of two numbers.

Parameters
----------
a : int
    First number.
b : int
    Second number.
"#;
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.signature, Some("add(a, b)".to_string()));
        assert_eq!(result.summary, "The sum of two numbers.");
        assert_eq!(result.parameters.len(), 2);
    }
}
