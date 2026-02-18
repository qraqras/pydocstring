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
use crate::span::{Span, Spanned};
use crate::types::{
    NumPyDocstring, NumPyException, NumPyParameter, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader,
};

/// Parse a NumPy-style docstring.
pub fn parse_numpy(input: &str) -> ParseResult<NumPyDocstring> {
    let mut docstring = NumPyDocstring::new();
    docstring.source = input.to_string();
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
                docstring.signature = Some(Spanned::dummy(first_line.to_string()));
                i += 1;
            } else {
                docstring.summary = Spanned::dummy(first_line.to_string());
                i += 1;
            }
        } else {
            docstring.summary = Spanned::dummy(first_line.to_string());
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
            docstring.summary = Spanned::dummy(summary_line.to_string());
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
        docstring.extended_summary = Some(Spanned::dummy(description_lines.join(" ")));
    }

    // Parse sections
    while i < lines.len() {
        if is_section_header(&lines, i) {
            let section_name = lines[i].trim();

            let header = NumPySectionHeader {
                span: Span::empty(),
                name: Spanned::dummy(section_name.to_string()),
                underline: Span::empty(),
            };

            i += 2; // Skip header and underline

            // Normalize to lowercase for case-insensitive matching.
            // The original text is preserved in header.name.value.
            let normalized = section_name.to_ascii_lowercase();
            let (body, next_i) = match normalized.as_str() {
                "parameters" | "params" => {
                    let (params, next_i) = parse_parameters(&lines, i)?;
                    (NumPySectionBody::Parameters(params), next_i)
                }
                "returns" | "return" => {
                    let (returns, next_i) = parse_returns(&lines, i)?;
                    (NumPySectionBody::Returns(returns), next_i)
                }
                "raises" | "raise" => {
                    let (raises, next_i) = parse_raises(&lines, i)?;
                    (NumPySectionBody::Raises(raises), next_i)
                }
                "yields" | "yield" => {
                    let (yields, next_i) = parse_returns(&lines, i)?;
                    (NumPySectionBody::Yields(yields), next_i)
                }
                "receives" | "receive" => {
                    let (receives, next_i) = parse_parameters(&lines, i)?;
                    (NumPySectionBody::Receives(receives), next_i)
                }
                "other parameters" | "other params" => {
                    let (params, next_i) = parse_parameters(&lines, i)?;
                    (NumPySectionBody::OtherParameters(params), next_i)
                }
                "warns" | "warn" => {
                    let (raises, next_i) = parse_raises(&lines, i)?;
                    let warns = raises
                        .into_iter()
                        .map(|e| crate::types::NumPyWarning {
                            span: e.span,
                            warning_type: e.exception_type,
                            description: e.description,
                        })
                        .collect();
                    (NumPySectionBody::Warns(warns), next_i)
                }
                "notes" | "note" => {
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (NumPySectionBody::Notes(Spanned::dummy(content)), next_i)
                }
                "examples" | "example" => {
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (NumPySectionBody::Examples(Spanned::dummy(content)), next_i)
                }
                "warnings" | "warning" => {
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (NumPySectionBody::Warnings(Spanned::dummy(content)), next_i)
                }
                "see also" => {
                    // TODO: parse structured see-also items
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (
                        NumPySectionBody::SeeAlso(vec![crate::types::SeeAlsoItem {
                            span: Span::empty(),
                            names: vec![Spanned::dummy(content)],
                            description: None,
                        }]),
                        next_i,
                    )
                }
                "references" => {
                    // TODO: parse structured references
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (
                        NumPySectionBody::References(vec![crate::types::NumPyReference {
                            span: Span::empty(),
                            number: 1,
                            content: Spanned::dummy(content),
                        }]),
                        next_i,
                    )
                }
                "attributes" => {
                    let (params, next_i) = parse_parameters(&lines, i)?;
                    let attrs = params
                        .into_iter()
                        .map(|p| crate::types::NumPyAttribute {
                            span: p.span,
                            name: p
                                .names
                                .into_iter()
                                .next()
                                .unwrap_or_else(Spanned::empty_string),
                            attr_type: p.param_type,
                            description: p.description,
                        })
                        .collect();
                    (NumPySectionBody::Attributes(attrs), next_i)
                }
                "methods" => {
                    let (params, next_i) = parse_parameters(&lines, i)?;
                    let methods = params
                        .into_iter()
                        .map(|p| crate::types::NumPyMethod {
                            span: p.span,
                            name: p
                                .names
                                .into_iter()
                                .next()
                                .unwrap_or_else(Spanned::empty_string),
                            description: p.description,
                        })
                        .collect();
                    (NumPySectionBody::Methods(methods), next_i)
                }
                _ => {
                    let (content, next_i) = parse_section_content(&lines, i)?;
                    (NumPySectionBody::Unknown(Spanned::dummy(content)), next_i)
                }
            };

            docstring.sections.push(NumPySection {
                span: Span::empty(),
                header,
                body,
            });

            i = next_i;
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
                    description: Spanned::dummy(desc_lines.join(" ")),
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
///
/// NumPy convention: parameter name and type are separated by ` : ` (with spaces).
/// A bare `:` without surrounding spaces is not treated as a separator.
fn parse_parameter_header(line: &str) -> Option<(String, usize)> {
    if line.contains(" : ") || line.ends_with(" :") {
        Some((line.to_string(), 0))
    } else {
        None
    }
}

/// Parse parameter name and type from "name : type" or "name : type, optional".
/// Also extracts default values like "default True" or "default=True".
fn parse_name_and_type(
    text: &str,
) -> (
    Vec<Spanned<String>>,
    Option<Spanned<String>>,
    Option<Span>,
    Option<Spanned<String>>,
) {
    // Split on " : " (NumPy convention) to avoid splitting inside types like dict[str: str]
    let (name_part, type_part_opt) = if let Some(pos) = text.find(" : ") {
        (&text[..pos], Some(text[pos + 3..].trim()))
    } else if text.ends_with(" :") {
        (&text[..text.len() - 2], Some(""))
    } else {
        (text.as_ref(), None)
    };

    // Parse names (can be multiple like "x1, x2")
    let names: Vec<Spanned<String>> = name_part
        .trim()
        .split(',')
        .map(|n| Spanned::dummy(n.trim().to_string()))
        .collect();

    let type_part = match type_part_opt {
        Some(t) => t,
        None => return (names, None, None, None),
    };
    let optional = if type_part.contains("optional") {
        Some(Span::empty())
    } else {
        None
    };

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
        default_val = Some(Spanned::dummy(after_default[..end].to_string()));

        // Extract type without default part
        clean_type = &type_part[..default_pos].trim();
    }

    let param_type = if optional.is_some() {
        Some(Spanned::dummy(
            clean_type
                .replace(", optional", "")
                .replace(",optional", "")
                .trim()
                .to_string(),
        ))
    } else {
        if clean_type.is_empty() {
            None
        } else {
            Some(Spanned::dummy(clean_type.to_string()))
        }
    };

    (names, param_type, optional, default_val)
}

/// Parse the Returns section.
///
/// Supports both unnamed and named return values:
/// ```text
/// Returns
/// -------
/// int                       # unnamed, type only
///     Description.
///
/// result : int              # named
///     Description.
///
/// x : int                   # multiple named
///     First.
/// y : int
///     Second.
/// ```
fn parse_returns(lines: &[&str], start: usize) -> ParseResult<(Vec<NumPyReturns>, usize)> {
    let mut returns = Vec::new();
    let mut i = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i].trim();

        // Non-empty, non-indented line = start of a return entry
        if !line.is_empty() && !lines[i].starts_with("    ") {
            let (name, return_type) = if line.contains(" : ") {
                // Named return: "name : type"
                let parts: Vec<&str> = line.splitn(2, " : ").collect();
                (
                    Some(Spanned::dummy(parts[0].trim().to_string())),
                    Some(Spanned::dummy(parts[1].trim().to_string())),
                )
            } else {
                // Unnamed return: type only
                (None, Some(Spanned::dummy(line.to_string())))
            };

            // Collect indented description lines
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

            returns.push(NumPyReturns {
                span: Span::empty(),
                name,
                return_type,
                description: Spanned::dummy(desc_lines.join(" ")),
            });
            continue;
        }

        i += 1;
    }

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
                exception_type: Spanned::dummy(exception_type),
                description: Spanned::dummy(desc_lines.join(" ")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_numpy() {
        let docstring = "Brief description.";
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.summary.value, "Brief description.");
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
        assert_eq!(result.summary.value, "Brief description.");
        assert_eq!(result.parameters().len(), 2);
        assert_eq!(result.parameters()[0].names[0].value, "x");
        assert_eq!(
            result.parameters()[0]
                .param_type
                .as_ref()
                .map(|t| t.value.as_str()),
            Some("int")
        );
        assert!(result.parameters()[0].optional.is_none());
        assert_eq!(result.parameters()[1].names[0].value, "y");
        assert!(result.parameters()[1].optional.is_some());
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
        assert_eq!(
            result.signature.as_ref().map(|s| s.value.as_str()),
            Some("add(a, b)")
        );
        assert_eq!(result.summary.value, "The sum of two numbers.");
        assert_eq!(result.parameters().len(), 2);
    }

    #[test]
    fn test_parse_named_returns() {
        let docstring = r#"Compute values.

Returns
-------
x : int
    The first value.
y : float
    The second value.
"#;
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.returns().len(), 2);
        assert_eq!(
            result.returns()[0].name.as_ref().map(|n| n.value.as_str()),
            Some("x")
        );
        assert_eq!(
            result.returns()[0]
                .return_type
                .as_ref()
                .map(|t| t.value.as_str()),
            Some("int")
        );
        assert_eq!(result.returns()[0].description.value, "The first value.");
        assert_eq!(
            result.returns()[1].name.as_ref().map(|n| n.value.as_str()),
            Some("y")
        );
    }

    #[test]
    fn test_description_with_colon_not_treated_as_param() {
        let docstring = r#"Brief summary.

Parameters
----------
x : int
    A value like key: value should not split.
"#;
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.parameters().len(), 1);
        assert_eq!(result.parameters()[0].names[0].value, "x");
        assert!(result.parameters()[0]
            .description
            .value
            .contains("key: value"));
    }

    #[test]
    fn test_case_insensitive_sections() {
        let docstring = r#"Brief summary.

parameters
----------
x : int
    First param.

returns
-------
int
    The result.

NOTES
-----
Some notes here.
"#;
        let result = parse_numpy(docstring).unwrap();
        assert_eq!(result.parameters().len(), 1);
        assert_eq!(result.parameters()[0].names[0].value, "x");
        assert_eq!(result.returns().len(), 1);
        assert!(result.notes().is_some());
        // Original text is preserved in header
        assert_eq!(result.sections[0].header.name.value, "parameters");
        assert_eq!(result.sections[2].header.name.value, "NOTES");
    }
}
