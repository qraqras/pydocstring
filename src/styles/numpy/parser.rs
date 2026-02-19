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

use crate::ast::{build_line_offsets, indent_len, make_range, make_spanned, offset_to_line_col, Spanned, TextRange};
use crate::error::ParseResult;
use crate::styles::numpy::ast::{
    NumPyDeprecation, NumPyDocstring, NumPyException, NumPyParameter, NumPyReturns, NumPySection,
    NumPySectionBody, NumPySectionHeader,
};

// =============================================================================
// Helpers
// =============================================================================

/// Detect the indent level of the first non-empty line starting at `start`.
///
/// This is used within section parsers to determine the "entry indent" —
/// the indentation at which parameter/return entries appear. Lines more
/// indented than this are treated as descriptions.
fn detect_entry_indent(lines: &[&str], start: usize) -> usize {
    for i in start..lines.len() {
        if is_section_header(lines, i) {
            break;
        }
        let line = lines[i];
        if !line.trim().is_empty() {
            return indent_len(line);
        }
    }
    0
}

// =============================================================================
// Section detection
// =============================================================================

/// Check if `lines[index]` is a NumPy-style section header (a non-empty line
/// followed by a line of only dashes).
fn is_section_header(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() {
        return false;
    }
    let current = lines[index].trim();
    let next = lines[index + 1].trim();
    !current.is_empty() && !next.is_empty() && next.chars().all(|c| c == '-')
}

// =============================================================================
// Description collector
// =============================================================================

/// Collect indented description lines starting at `start`, preserving blank
/// lines between paragraphs. Returns the spanned description text and the next
/// line index to process.
fn collect_description(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    entry_indent: usize,
) -> (Spanned<String>, usize) {
    let mut i = start;
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < lines.len() {
        let line = lines[i];
        if is_section_header(lines, i) {
            break;
        }
        // Non-empty line at or below entry indentation signals end of description
        if !line.trim().is_empty() && indent_len(line) <= entry_indent {
            break;
        }
        desc_parts.push(line.trim());
        if !line.trim().is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(i);
            }
            last_content_line = i;
        }
        i += 1;
    }

    // Trim trailing empty entries
    while desc_parts.last().is_some_and(|l| l.is_empty()) {
        desc_parts.pop();
    }
    // Trim leading empty entries
    while desc_parts.first().is_some_and(|l| l.is_empty()) {
        desc_parts.remove(0);
    }

    let text = desc_parts.join("\n");

    if let Some(first) = first_content_line {
        let first_col = indent_len(lines[first]);
        let last_trimmed = lines[last_content_line].trim();
        let last_col = indent_len(lines[last_content_line]) + last_trimmed.len();
        (
            make_spanned(text, first, first_col, last_content_line, last_col, offsets),
            i,
        )
    } else {
        (Spanned::empty_string(), i)
    }
}

// =============================================================================
// Main parser
// =============================================================================

/// Parse a NumPy-style docstring.
pub fn parse_numpy(input: &str) -> ParseResult<NumPyDocstring> {
    let offsets = build_line_offsets(input);
    let lines: Vec<&str> = input.lines().collect();
    let mut docstring = NumPyDocstring::new();
    docstring.source = input.to_string();

    if lines.is_empty() {
        return ParseResult::ok(docstring);
    }

    let mut i = 0;

    // --- Skip leading blank lines ---
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    if i >= lines.len() {
        return ParseResult::ok(docstring);
    }

    // --- Signature / Summary ---
    let first_trimmed = lines[i].trim();
    let first_col = indent_len(lines[i]);

    // Detect function signature: "func(args)" on its own line
    if first_trimmed.contains('(') && first_trimmed.ends_with(')') {
        let next_i = i + 1;
        let is_sig = if next_i < lines.len() {
            let nl = lines[next_i].trim();
            nl.is_empty() || is_section_header(&lines, next_i)
        } else {
            true
        };
        if is_sig {
            docstring.signature = Some(make_spanned(
                first_trimmed.to_string(),
                i,
                first_col,
                i,
                first_col + first_trimmed.len(),
                &offsets,
            ));
            i += 1;
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
        }
    }

    // Parse summary line (may follow signature)
    if i < lines.len() && !is_section_header(&lines, i) {
        let line = lines[i];
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let col = indent_len(line);
            docstring.summary = make_spanned(
                trimmed.to_string(),
                i,
                col,
                i,
                col + trimmed.len(),
                &offsets,
            );
            i += 1;
        }
    }

    // skip blanks
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    // --- Deprecation directive ---
    if i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with(".. deprecated::") {
            let col = indent_len(lines[i]);
            let prefix = ".. deprecated::";
            let after_prefix = &trimmed[prefix.len()..];
            let ws_len = after_prefix.len() - after_prefix.trim_start().len();
            let version_str = after_prefix.trim();
            let version_col = col + prefix.len() + ws_len;

            let version_spanned = make_spanned(
                version_str.to_string(),
                i,
                version_col,
                i,
                version_col + version_str.len(),
                &offsets,
            );

            let dep_start_line = i;
            i += 1;

            // Collect indented body lines
            let (desc_spanned, next_i) = collect_description(&lines, i, &offsets, col);
            i = next_i;

            // Compute deprecation span
            let (dep_end_line, dep_end_col) = if desc_spanned.value.is_empty() {
                (dep_start_line, col + trimmed.len())
            } else {
                offset_to_line_col(desc_spanned.range.end().raw() as usize, &offsets)
            };

            docstring.deprecation = Some(NumPyDeprecation {
                range: make_range(dep_start_line, col, dep_end_line, dep_end_col, &offsets),
                version: version_spanned,
                description: desc_spanned,
            });

            // skip blanks
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
        }
    }

    // --- Extended summary ---
    if i < lines.len() && !is_section_header(&lines, i) {
        let start_line = i;
        let mut desc_lines: Vec<&str> = Vec::new();
        let mut last_non_empty_line = i;

        while i < lines.len() && !is_section_header(&lines, i) {
            let trimmed = lines[i].trim();
            desc_lines.push(trimmed);
            if !trimmed.is_empty() {
                last_non_empty_line = i;
            }
            i += 1;
        }

        // Trim trailing empty lines
        let keep = last_non_empty_line - start_line + 1;
        desc_lines.truncate(keep);

        let joined = desc_lines.join("\n");
        if !joined.trim().is_empty() {
            let first_col = indent_len(lines[start_line]);
            let last_trimmed = lines[last_non_empty_line].trim();
            let last_col = indent_len(lines[last_non_empty_line]) + last_trimmed.len();
            docstring.extended_summary = Some(make_spanned(
                joined,
                start_line,
                first_col,
                last_non_empty_line,
                last_col,
                &offsets,
            ));
        }
    }

    // --- Sections ---
    while i < lines.len() {
        if is_section_header(&lines, i) {
            let section_start = i;
            let header_line = lines[i];
            let header_trimmed = header_line.trim();
            let header_col = indent_len(header_line);

            let underline_line = lines[i + 1];
            let underline_trimmed = underline_line.trim();
            let underline_col = indent_len(underline_line);

            let header = NumPySectionHeader {
                range: make_range(
                    i,
                    header_col,
                    i + 1,
                    underline_col + underline_trimmed.len(),
                    &offsets,
                ),
                name: make_spanned(
                    header_trimmed.to_string(),
                    i,
                    header_col,
                    i,
                    header_col + header_trimmed.len(),
                    &offsets,
                ),
                underline: make_range(
                    i + 1,
                    underline_col,
                    i + 1,
                    underline_col + underline_trimmed.len(),
                    &offsets,
                ),
            };

            i += 2; // skip header + underline

            let normalized = header_trimmed.to_ascii_lowercase();
            let (body, next_i) = match normalized.as_str() {
                "parameters" | "params" => {
                    let (params, ni) = parse_parameters(&lines, i, &offsets).value;
                    (NumPySectionBody::Parameters(params), ni)
                }
                "returns" | "return" => {
                    let (rets, ni) = parse_returns(&lines, i, &offsets).value;
                    (NumPySectionBody::Returns(rets), ni)
                }
                "raises" | "raise" => {
                    let (raises, ni) = parse_raises(&lines, i, &offsets).value;
                    (NumPySectionBody::Raises(raises), ni)
                }
                "yields" | "yield" => {
                    let (yields, ni) = parse_returns(&lines, i, &offsets).value;
                    (NumPySectionBody::Yields(yields), ni)
                }
                "receives" | "receive" => {
                    let (receives, ni) = parse_parameters(&lines, i, &offsets).value;
                    (NumPySectionBody::Receives(receives), ni)
                }
                "other parameters" | "other params" => {
                    let (params, ni) = parse_parameters(&lines, i, &offsets).value;
                    (NumPySectionBody::OtherParameters(params), ni)
                }
                "warns" | "warn" => {
                    let (raises, ni) = parse_raises(&lines, i, &offsets).value;
                    let warns = raises
                        .into_iter()
                        .map(|e| crate::styles::numpy::ast::NumPyWarning {
                            range: e.range,
                            warning_type: e.exception_type,
                            description: e.description,
                        })
                        .collect();
                    (NumPySectionBody::Warns(warns), ni)
                }
                "notes" | "note" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets).value;
                    (NumPySectionBody::Notes(content), ni)
                }
                "examples" | "example" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets).value;
                    (NumPySectionBody::Examples(content), ni)
                }
                "warnings" | "warning" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets).value;
                    (NumPySectionBody::Warnings(content), ni)
                }
                "see also" => {
                    let (items, ni) = parse_see_also(&lines, i, &offsets).value;
                    (NumPySectionBody::SeeAlso(items), ni)
                }
                "references" => {
                    let (refs, ni) = parse_references(&lines, i, &offsets).value;
                    (NumPySectionBody::References(refs), ni)
                }
                "attributes" => {
                    let (params, ni) = parse_parameters(&lines, i, &offsets).value;
                    let attrs = params
                        .into_iter()
                        .map(|p| crate::styles::numpy::ast::NumPyAttribute {
                            range: p.range,
                            name: p
                                .names
                                .into_iter()
                                .next()
                                .unwrap_or_else(Spanned::empty_string),
                            attr_type: p.param_type,
                            description: p.description,
                        })
                        .collect();
                    (NumPySectionBody::Attributes(attrs), ni)
                }
                "methods" => {
                    let (params, ni) = parse_parameters(&lines, i, &offsets).value;
                    let methods = params
                        .into_iter()
                        .map(|p| crate::styles::numpy::ast::NumPyMethod {
                            range: p.range,
                            name: p
                                .names
                                .into_iter()
                                .next()
                                .unwrap_or_else(Spanned::empty_string),
                            description: p.description,
                        })
                        .collect();
                    (NumPySectionBody::Methods(methods), ni)
                }
                _ => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets).value;
                    (NumPySectionBody::Unknown(content), ni)
                }
            };

            // Compute section span (header to last non-empty body line)
            let section_end_line = {
                let mut end = next_i.saturating_sub(1);
                while end > section_start && lines.get(end).is_none_or(|l| l.trim().is_empty()) {
                    end -= 1;
                }
                end
            };
            let section_end_col = lines
                .get(section_end_line)
                .map(|l| indent_len(l) + l.trim().len())
                .unwrap_or(0);

            docstring.sections.push(NumPySection {
                range: make_range(
                    section_start,
                    header_col,
                    section_end_line,
                    section_end_col,
                    &offsets,
                ),
                header,
                body,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    // Docstring span
    let last_line = lines.len().saturating_sub(1);
    let last_col = lines.last().map(|l| l.len()).unwrap_or(0);
    docstring.range = make_range(0, 0, last_line, last_col, &offsets);

    ParseResult::ok(docstring)
}

// =============================================================================
// Parameter parsing
// =============================================================================

/// Parse the Parameters section body.
fn parse_parameters(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Vec<NumPyParameter>, usize)> {
    let mut parameters = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start);

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        // A parameter header is a non-empty line at entry indentation with ` : ` or ending with ` :`
        if !trimmed.is_empty() && indent_len(line) <= entry_indent && is_param_header(trimmed) {
            let col = indent_len(line);
            let entry_start = i;
            let (names, param_type, optional, default_val) =
                parse_name_and_type(trimmed, i, col, offsets);

            i += 1;
            let (desc, next_i) = collect_description(lines, i, offsets, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                offset_to_line_col(desc.range.end().raw() as usize, &offsets)
            };

            parameters.push(NumPyParameter {
                range: make_range(entry_start, col, entry_end_line, entry_end_col, offsets),
                names,
                param_type,
                description: desc,
                optional,
                default: default_val,
            });
            i = next_i;
            continue;
        }

        i += 1;
    }

    ParseResult::ok((parameters, i))
}

/// Check whether `trimmed` looks like a parameter header line.
///
/// NumPy convention: name and type separated by ` : ` (with spaces).
fn is_param_header(trimmed: &str) -> bool {
    trimmed.contains(" : ") || trimmed.ends_with(" :")
}

/// Result of parsing a parameter header: (names, type, optional span, default value).
type ParamHeaderParts = (
    Vec<Spanned<String>>,
    Option<Spanned<String>>,
    Option<TextRange>,
    Option<Spanned<String>>,
);

/// Parse `"name : type, optional"` into components with precise spans.
///
/// `line_idx` is the 0-based line index, `col_base` is the byte column where
/// `text` starts in the raw line.
fn parse_name_and_type(
    text: &str,
    line_idx: usize,
    col_base: usize,
    offsets: &[usize],
) -> ParamHeaderParts {
    // Split on ` : ` (NumPy convention)
    let (name_str, type_str, sep_pos) = if let Some(pos) = text.find(" : ") {
        (&text[..pos], Some(text[pos + 3..].trim()), pos)
    } else if let Some(stripped) = text.strip_suffix(" :") {
        (stripped, None, stripped.len())
    } else {
        // No separator — whole text is the name
        let names = parse_name_list(text, line_idx, col_base, offsets);
        return (names, None, None, None);
    };

    let names = parse_name_list(name_str, line_idx, col_base, offsets);

    let type_str = match type_str {
        Some(t) if !t.is_empty() => t,
        _ => return (names, None, None, None),
    };

    // Column where the type part starts in the line
    let type_col = col_base + sep_pos + 3;
    // Adjust for any leading whitespace we trimmed
    let type_trimmed_offset = text[sep_pos + 3..].len() - text[sep_pos + 3..].trim_start().len();
    let type_col = type_col + type_trimmed_offset;

    // Check for "optional" marker
    let optional = if let Some(opt_pos) = type_str.find("optional") {
        let opt_col = type_col + opt_pos;
        Some(make_range(
            line_idx,
            opt_col,
            line_idx,
            opt_col + "optional".len(),
            offsets,
        ))
    } else {
        None
    };

    // Extract default value (e.g., "default True", "default=True")
    let mut default_val = None;
    let mut clean_type_end = type_str.len();

    if let Some(def_pos) = type_str.find("default") {
        let after_default = type_str[def_pos + 7..].trim_start();
        let after_default = after_default
            .strip_prefix('=')
            .or_else(|| after_default.strip_prefix(':'))
            .unwrap_or(after_default)
            .trim_start();

        let val_end = after_default.find(',').unwrap_or(after_default.len());
        let val_text = after_default[..val_end].trim();

        if !val_text.is_empty() {
            // Find exact position in the original type_str
            let val_offset_in_type = type_str.find(val_text).unwrap_or(def_pos + 7);
            let val_col = type_col + val_offset_in_type;
            default_val = Some(make_spanned(
                val_text.to_string(),
                line_idx,
                val_col,
                line_idx,
                val_col + val_text.len(),
                offsets,
            ));
        }

        clean_type_end = def_pos;
    }

    // Build clean type (without "optional" and "default ...")
    let mut clean = type_str[..clean_type_end].to_string();
    if optional.is_some() {
        clean = clean
            .replace(", optional", "")
            .replace(",optional", "")
            .replace("optional", "");
    }
    let clean = clean.trim().trim_end_matches(',').trim().to_string();

    let param_type = if clean.is_empty() {
        None
    } else {
        // Find the clean type text position in the original type_str
        let type_text_offset = type_str.find(&*clean).unwrap_or(0);
        let tc = type_col + type_text_offset;
        Some(make_spanned(
            clean.clone(),
            line_idx,
            tc,
            line_idx,
            tc + clean.len(),
            offsets,
        ))
    };

    (names, param_type, optional, default_val)
}

/// Parse a comma-separated name list like `"x1, x2"` into spanned names.
fn parse_name_list(
    text: &str,
    line_idx: usize,
    col_base: usize,
    offsets: &[usize],
) -> Vec<Spanned<String>> {
    let mut names = Vec::new();
    let mut byte_pos = 0usize;

    for part in text.split(',') {
        let leading = part.len() - part.trim_start().len();
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            let name_col = col_base + byte_pos + leading;
            names.push(make_spanned(
                trimmed.to_string(),
                line_idx,
                name_col,
                line_idx,
                name_col + trimmed.len(),
                offsets,
            ));
        }
        byte_pos += part.len() + 1; // +1 for the comma
    }

    names
}

// =============================================================================
// Returns parsing
// =============================================================================

/// Parse the Returns / Yields section body.
///
/// Supports both unnamed and named return values:
/// ```text
/// int                       # unnamed, type only
///     Description.
///
/// result : int              # named
///     Description.
/// ```
fn parse_returns(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Vec<NumPyReturns>, usize)> {
    let mut returns = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start);

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            let col = indent_len(line);
            let entry_start = i;

            let (name, return_type) = if trimmed.contains(" : ") {
                // Named return: "name : type"
                let sep = trimmed.find(" : ").unwrap();
                let n = trimmed[..sep].trim();
                let t = trimmed[sep + 3..].trim();
                let name_col = col;
                let type_col = col + sep + 3 + (trimmed[sep + 3..].len() - t.len());
                (
                    Some(make_spanned(
                        n.to_string(),
                        i,
                        name_col,
                        i,
                        name_col + n.len(),
                        offsets,
                    )),
                    Some(make_spanned(
                        t.to_string(),
                        i,
                        type_col,
                        i,
                        type_col + t.len(),
                        offsets,
                    )),
                )
            } else {
                // Unnamed: type only
                (
                    None,
                    Some(make_spanned(
                        trimmed.to_string(),
                        i,
                        col,
                        i,
                        col + trimmed.len(),
                        offsets,
                    )),
                )
            };

            i += 1;
            let (desc, next_i) = collect_description(lines, i, offsets, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                offset_to_line_col(desc.range.end().raw() as usize, &offsets)
            };

            returns.push(NumPyReturns {
                range: make_range(entry_start, col, entry_end_line, entry_end_col, offsets),
                name,
                return_type,
                description: desc,
            });
            i = next_i;
            continue;
        }

        i += 1;
    }

    ParseResult::ok((returns, i))
}

// =============================================================================
// Raises parsing
// =============================================================================

/// Parse the Raises section body.
fn parse_raises(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Vec<NumPyException>, usize)> {
    let mut raises = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start);

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            let col = indent_len(line);
            let entry_start = i;

            let exc_type =
                make_spanned(trimmed.to_string(), i, col, i, col + trimmed.len(), offsets);

            i += 1;
            let (desc, next_i) = collect_description(lines, i, offsets, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                offset_to_line_col(desc.range.end().raw() as usize, &offsets)
            };

            raises.push(NumPyException {
                range: make_range(entry_start, col, entry_end_line, entry_end_col, offsets),
                exception_type: exc_type,
                description: desc,
            });
            i = next_i;
            continue;
        }

        i += 1;
    }

    ParseResult::ok((raises, i))
}

// =============================================================================
// Free-text section content
// =============================================================================

/// Parse a free-text section body (Notes, Examples, Warnings, Unknown, etc.).
///
/// Preserves blank lines between paragraphs.
fn parse_section_content(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Spanned<String>, usize)> {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut i = start;
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }
        let trimmed = lines[i].trim();
        content_lines.push(trimmed);
        if !trimmed.is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(i);
            }
            last_content_line = i;
        }
        i += 1;
    }

    // Trim trailing empty
    while content_lines.last().is_some_and(|l| l.is_empty()) {
        content_lines.pop();
    }
    while content_lines.first().is_some_and(|l| l.is_empty()) {
        content_lines.remove(0);
    }

    let text = content_lines.join("\n");

    let spanned = if let Some(first) = first_content_line {
        let first_col = indent_len(lines[first]);
        let last_trimmed = lines[last_content_line].trim();
        let last_col = indent_len(lines[last_content_line]) + last_trimmed.len();
        make_spanned(text, first, first_col, last_content_line, last_col, offsets)
    } else {
        Spanned::empty_string()
    };

    ParseResult::ok((spanned, i))
}

// =============================================================================
// See Also parsing
// =============================================================================

/// Parse the See Also section body.
///
/// ```text
/// func_a : Description of func_a.
/// func_b, func_c
/// ```
fn parse_see_also(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Vec<crate::styles::numpy::ast::SeeAlsoItem>, usize)> {
    let mut items = Vec::new();
    let mut i = start;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let col = indent_len(line);
        let entry_start = i;

        // Split on " : " for description
        let (names_str, description) = if let Some(pos) = trimmed.find(" : ") {
            let desc_text = trimmed[pos + 3..].trim();
            let desc_col = col + pos + 3 + (trimmed[pos + 3..].len() - desc_text.len());
            (
                &trimmed[..pos],
                Some(make_spanned(
                    desc_text.to_string(),
                    i,
                    desc_col,
                    i,
                    desc_col + desc_text.len(),
                    offsets,
                )),
            )
        } else {
            (trimmed, None)
        };

        let names = parse_name_list(names_str, i, col, offsets);
        let entry_end_col = col + trimmed.len();

        items.push(crate::styles::numpy::ast::SeeAlsoItem {
            range: make_range(entry_start, col, entry_start, entry_end_col, offsets),
            names,
            description,
        });

        i += 1;
    }

    ParseResult::ok((items, i))
}

// =============================================================================
// References parsing
// =============================================================================

/// Parse the References section body.
///
/// Supports RST citation references like `.. [1] Author, Title`.
fn parse_references(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
) -> ParseResult<(Vec<crate::styles::numpy::ast::NumPyReference>, usize)> {
    let mut refs = Vec::new();
    let mut i = start;
    let mut current_number: u32 = 0;
    let mut current_content_lines: Vec<&str> = Vec::new();
    let mut current_start_line: Option<usize> = None;
    let mut current_col = 0usize;

    while i < lines.len() {
        if is_section_header(lines, i) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        // Check for `.. [N]` pattern
        if trimmed.starts_with(".. [") {
            // Flush previous reference
            if let Some(start_l) = current_start_line {
                let content = current_content_lines.join("\n");
                let end_l = if current_content_lines.len() > 1 {
                    start_l + current_content_lines.len() - 1
                } else {
                    start_l
                };
                let end_col = current_col + content.lines().last().unwrap_or("").len();
                refs.push(crate::styles::numpy::ast::NumPyReference {
                    range: make_range(start_l, current_col, end_l, end_col, offsets),
                    number: current_number,
                    content: make_spanned(content, start_l, current_col, end_l, end_col, offsets),
                });
            }

            let col = indent_len(line);
            // Parse `.. [N] content`
            if let Some(bracket_end) = trimmed.find(']') {
                let num_str = &trimmed[4..bracket_end];
                current_number = num_str.parse().unwrap_or(refs.len() as u32 + 1);
                let after_bracket = trimmed[bracket_end + 1..].trim();
                current_content_lines = vec![after_bracket];
                current_start_line = Some(i);
                current_col = col;
            }
            i += 1;
        } else if trimmed.is_empty() {
            // Empty line between references — flush current
            if let Some(start_l) = current_start_line.take() {
                let content = current_content_lines.join("\n");
                let end_l = if current_content_lines.len() > 1 {
                    start_l + current_content_lines.len() - 1
                } else {
                    start_l
                };
                let end_col = current_col + content.lines().last().unwrap_or("").len();
                refs.push(crate::styles::numpy::ast::NumPyReference {
                    range: make_range(start_l, current_col, end_l, end_col, offsets),
                    number: current_number,
                    content: make_spanned(content, start_l, current_col, end_l, end_col, offsets),
                });
                current_content_lines.clear();
            }
            i += 1;
        } else if current_start_line.is_some() {
            // Continuation of current reference
            current_content_lines.push(trimmed);
            i += 1;
        } else {
            // Non-RST reference — treat as plain text content
            current_content_lines.push(trimmed);
            if current_start_line.is_none() {
                current_start_line = Some(i);
                current_number = refs.len() as u32 + 1;
                current_col = indent_len(line);
            }
            i += 1;
        }
    }

    // Flush last reference
    if let Some(start_l) = current_start_line {
        let content = current_content_lines.join("\n");
        let end_l = if current_content_lines.len() > 1 {
            start_l + current_content_lines.len() - 1
        } else {
            start_l
        };
        let end_col = current_col + content.lines().last().unwrap_or("").len();
        refs.push(crate::styles::numpy::ast::NumPyReference {
            range: make_range(start_l, current_col, end_l, end_col, offsets),
            number: current_number,
            content: make_spanned(content, start_l, current_col, end_l, end_col, offsets),
        });
    }

    ParseResult::ok((refs, i))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_entry_indent() {
        // No indentation
        assert_eq!(detect_entry_indent(&["x : int", "    Description."], 0), 0);

        // 4-space indentation
        assert_eq!(
            detect_entry_indent(&["    x : int", "        Description."], 0),
            4
        );

        // Skip leading blank lines
        assert_eq!(
            detect_entry_indent(&["", "    x : int", "        Description."], 0),
            4
        );

        // Empty input
        assert_eq!(detect_entry_indent(&[], 0), 0);

        // Stops at section header
        assert_eq!(detect_entry_indent(&["Returns", "-------"], 0), 0);
    }
}
