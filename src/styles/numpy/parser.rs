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

use crate::ast::Spanned;
use crate::cursor::{Cursor, indent_len};
use crate::styles::numpy::ast::{
    NumPyDeprecation, NumPyDocstring, NumPyDocstringItem, NumPyException, NumPyParameter,
    NumPyReturns, NumPySection, NumPySectionBody, NumPySectionHeader, NumPySectionKind,
};

// =============================================================================
// Entry colon detection
// =============================================================================

/// Find the byte offset of the first entry-separating colon in `text`.
///
/// Skips colons inside balanced brackets (`()`, `[]`, `{}`, `<>`) so that
/// type annotations like `Dict[str, int]` never trigger a false split.
fn find_entry_colon(text: &str) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth = depth.saturating_sub(1),
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split `text` by top-level commas (respecting `()`, `[]`, `{}`, and `<>` depth).
///
/// Returns an iterator of `(byte_offset, segment)` pairs where
/// `byte_offset` is the start position of each segment within `text`.
fn split_comma_parts(text: &str) -> Vec<(usize, &str)> {
    let mut parts = Vec::new();
    let mut depth: u32 = 0;
    let mut start = 0;

    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push((start, &text[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push((start, &text[start..]));
    parts
}

// =============================================================================
// Section detection
// =============================================================================

/// Check if a trimmed line is a NumPy-style section underline (only dashes).
fn is_underline(trimmed: &str) -> bool {
    !trimmed.is_empty() && trimmed.bytes().all(|b| b == b'-')
}

/// Find the line index where the next section header starts at or after `start`,
/// or `total_lines` if there are no more sections.
///
/// Uses the "pending line" pattern: each line is read once, and when a dash
/// line is encountered the previous non-empty line is identified as the header.
fn find_next_section_start(cursor: &Cursor, start: usize) -> usize {
    let mut prev_non_empty = false;
    let mut prev_idx = start;
    for i in start..cursor.total_lines() {
        let trimmed = cursor.line_text(i).trim();
        if prev_non_empty && is_underline(trimmed) {
            return prev_idx;
        }
        prev_non_empty = !trimmed.is_empty();
        if prev_non_empty {
            prev_idx = i;
        }
    }
    cursor.total_lines()
}

// =============================================================================
// Description collector
// =============================================================================

/// Collect indented description lines starting at `cursor.line`, up to `end`.
///
/// Preserves blank lines between paragraphs. Stops at non-empty lines at or
/// below `entry_indent`.
///
/// On return, `cursor.line` points to the first unconsumed line.
fn collect_description(cursor: &mut Cursor, end: usize, entry_indent: usize) -> Spanned<String> {
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while cursor.line < end {
        let line = cursor.current_line_text();
        // Non-empty line at or below entry indentation signals end of description
        if !line.trim().is_empty() && indent_len(line) <= entry_indent {
            break;
        }
        desc_parts.push(line.trim());
        if !line.trim().is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(cursor.line);
            }
            last_content_line = cursor.line;
        }
        cursor.advance();
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
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content_line);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        cursor.make_spanned(text, first, first_col, last_content_line, last_col)
    } else {
        Spanned::empty_string()
    }
}

// =============================================================================
// Main parser
// =============================================================================

/// Parse a NumPy-style docstring.
pub fn parse_numpy(input: &str) -> NumPyDocstring {
    let mut cursor = Cursor::new(input);
    let first_section = find_next_section_start(&cursor, 0);
    let mut docstring = NumPyDocstring::new();
    docstring.source = input.to_string();

    if cursor.total_lines() == 0 {
        return docstring;
    }

    // --- Skip leading blank lines ---
    cursor.skip_blank_lines();
    if cursor.is_eof() {
        return docstring;
    }

    // --- Summary ---
    if cursor.line < first_section {
        let trimmed = cursor.current_trimmed();
        if !trimmed.is_empty() {
            let col = cursor.current_indent();
            docstring.summary = cursor.make_spanned(
                trimmed.to_string(),
                cursor.line,
                col,
                cursor.line,
                col + trimmed.len(),
            );
            cursor.advance();
        }
    }

    // skip blanks
    cursor.skip_blank_lines();

    // --- Deprecation directive ---
    if !cursor.is_eof() {
        let line = cursor.current_line_text();
        let trimmed = line.trim();
        if trimmed.starts_with(".. deprecated::") {
            let col = cursor.current_indent();
            let prefix = ".. deprecated::";
            let after_prefix = &trimmed[prefix.len()..];
            let ws_len = after_prefix.len() - after_prefix.trim_start().len();
            let version_str = after_prefix.trim();
            let version_col = col + prefix.len() + ws_len;

            // `..` at col..col+2
            let directive_marker =
                Some(cursor.make_spanned("..".to_string(), cursor.line, col, cursor.line, col + 2));
            // `deprecated` at col+3..col+13
            let kw_col = col + 3;
            let keyword = Some(cursor.make_spanned(
                "deprecated".to_string(),
                cursor.line,
                kw_col,
                cursor.line,
                kw_col + 10,
            ));
            // `::` at col+13..col+15
            let dc_col = col + 13;
            let double_colon = Some(cursor.make_spanned(
                "::".to_string(),
                cursor.line,
                dc_col,
                cursor.line,
                dc_col + 2,
            ));

            let version_spanned = cursor.make_spanned(
                version_str.to_string(),
                cursor.line,
                version_col,
                cursor.line,
                version_col + version_str.len(),
            );

            let dep_start_line = cursor.line;
            cursor.advance();

            // Collect indented body lines
            let desc_spanned = collect_description(&mut cursor, first_section, col);

            // Compute deprecation span
            let (dep_end_line, dep_end_col) = if desc_spanned.value.is_empty() {
                (dep_start_line, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc_spanned.range.end().raw() as usize)
            };

            docstring.deprecation = Some(NumPyDeprecation {
                range: cursor.make_range(dep_start_line, col, dep_end_line, dep_end_col),
                directive_marker,
                keyword,
                double_colon,
                version: version_spanned,
                description: desc_spanned,
            });

            // skip blanks
            cursor.skip_blank_lines();
        }
    }

    // --- Extended summary ---
    if cursor.line < first_section {
        let start_line = cursor.line;
        let mut desc_lines: Vec<&str> = Vec::new();
        let mut last_non_empty_line = cursor.line;

        while cursor.line < first_section {
            let trimmed = cursor.current_trimmed();
            desc_lines.push(trimmed);
            if !trimmed.is_empty() {
                last_non_empty_line = cursor.line;
            }
            cursor.advance();
        }

        // Trim trailing empty lines
        let keep = last_non_empty_line - start_line + 1;
        desc_lines.truncate(keep);

        let joined = desc_lines.join("\n");
        if !joined.trim().is_empty() {
            let first_line = cursor.line_text(start_line);
            let first_col = indent_len(first_line);
            let last_line = cursor.line_text(last_non_empty_line);
            let last_trimmed = last_line.trim();
            let last_col = indent_len(last_line) + last_trimmed.len();
            docstring.extended_summary = Some(cursor.make_spanned(
                joined,
                start_line,
                first_col,
                last_non_empty_line,
                last_col,
            ));
        }
    }

    // --- Sections ---
    cursor.line = first_section;
    while !cursor.is_eof() {
        // Verify this line is actually a section header (non-empty + next is underline)
        let header_trimmed = cursor.current_trimmed();
        if header_trimmed.is_empty()
            || cursor.line + 1 >= cursor.total_lines()
            || !is_underline(cursor.line_text(cursor.line + 1).trim())
        {
            // Non-blank lines that are not section headers are stray lines.
            if !header_trimmed.is_empty() {
                let col = cursor.current_indent();
                let spanned = cursor.make_spanned(
                    header_trimmed.to_string(),
                    cursor.line,
                    col,
                    cursor.line,
                    col + header_trimmed.len(),
                );
                docstring.items.push(NumPyDocstringItem::StrayLine(spanned));
            }
            cursor.advance();
            continue;
        }

        let section_start = cursor.line;
        let header_col = cursor.current_indent();

        let underline_line = cursor.line_text(cursor.line + 1);
        let underline_trimmed = underline_line.trim();
        let underline_col = indent_len(underline_line);

        let header = NumPySectionHeader {
            range: cursor.make_range(
                cursor.line,
                header_col,
                cursor.line + 1,
                underline_col + underline_trimmed.len(),
            ),
            name: cursor.make_spanned(
                header_trimmed.to_string(),
                cursor.line,
                header_col,
                cursor.line,
                header_col + header_trimmed.len(),
            ),
            underline: cursor.make_spanned(
                underline_trimmed.to_string(),
                cursor.line + 1,
                underline_col,
                cursor.line + 1,
                underline_col + underline_trimmed.len(),
            ),
        };

        cursor.line += 2; // skip header + underline

        let section_end = find_next_section_start(&cursor, cursor.line);
        let normalized = header_trimmed.to_ascii_lowercase();
        let section_kind = NumPySectionKind::from_name(&normalized);
        let body = match section_kind {
            Some(NumPySectionKind::Parameters) => {
                let params = parse_parameters(&mut cursor, section_end, header_col);
                NumPySectionBody::Parameters(params)
            }
            Some(NumPySectionKind::Returns) => {
                let rets = parse_returns(&mut cursor, section_end, header_col);
                NumPySectionBody::Returns(rets)
            }
            Some(NumPySectionKind::Raises) => {
                let raises = parse_raises(&mut cursor, section_end, header_col);
                NumPySectionBody::Raises(raises)
            }
            Some(NumPySectionKind::Yields) => {
                let yields = parse_returns(&mut cursor, section_end, header_col);
                NumPySectionBody::Yields(yields)
            }
            Some(NumPySectionKind::Receives) => {
                let receives = parse_parameters(&mut cursor, section_end, header_col);
                NumPySectionBody::Receives(receives)
            }
            Some(NumPySectionKind::OtherParameters) => {
                let params = parse_parameters(&mut cursor, section_end, header_col);
                NumPySectionBody::OtherParameters(params)
            }
            Some(NumPySectionKind::Warns) => {
                let raises = parse_raises(&mut cursor, section_end, header_col);
                let warns = raises
                    .into_iter()
                    .map(|e| crate::styles::numpy::ast::NumPyWarning {
                        range: e.range,
                        r#type: e.r#type,
                        description: e.description,
                    })
                    .collect();
                NumPySectionBody::Warns(warns)
            }
            Some(NumPySectionKind::Notes) => {
                let content = parse_section_content(&mut cursor, section_end);
                NumPySectionBody::Notes(content)
            }
            Some(NumPySectionKind::Examples) => {
                let content = parse_section_content(&mut cursor, section_end);
                NumPySectionBody::Examples(content)
            }
            Some(NumPySectionKind::Warnings) => {
                let content = parse_section_content(&mut cursor, section_end);
                NumPySectionBody::Warnings(content)
            }
            Some(NumPySectionKind::SeeAlso) => {
                let items = parse_see_also(&mut cursor, section_end);
                NumPySectionBody::SeeAlso(items)
            }
            Some(NumPySectionKind::References) => {
                let refs = parse_references(&mut cursor, section_end);
                NumPySectionBody::References(refs)
            }
            Some(NumPySectionKind::Attributes) => {
                let params = parse_parameters(&mut cursor, section_end, header_col);
                let attrs = params
                    .into_iter()
                    .map(|p| crate::styles::numpy::ast::NumPyAttribute {
                        range: p.range,
                        name: p
                            .names
                            .into_iter()
                            .next()
                            .unwrap_or_else(Spanned::empty_string),
                        colon: p.colon,
                        r#type: p.r#type,
                        description: p.description,
                    })
                    .collect();
                NumPySectionBody::Attributes(attrs)
            }
            Some(NumPySectionKind::Methods) => {
                let params = parse_parameters(&mut cursor, section_end, header_col);
                let methods = params
                    .into_iter()
                    .map(|p| crate::styles::numpy::ast::NumPyMethod {
                        range: p.range,
                        name: p
                            .names
                            .into_iter()
                            .next()
                            .unwrap_or_else(Spanned::empty_string),
                        colon: p.colon,
                        description: p.description,
                    })
                    .collect();
                NumPySectionBody::Methods(methods)
            }
            None => {
                let content = parse_section_content(&mut cursor, section_end);
                NumPySectionBody::Unknown(content)
            }
        };

        // Compute section span (header to last non-empty body line)
        let section_end_line = {
            let mut end = cursor.line.saturating_sub(1);
            while end > section_start {
                if !cursor.line_text(end).trim().is_empty() {
                    break;
                }
                end -= 1;
            }
            end
        };
        let section_end_col = {
            let end_line_text = cursor.line_text(section_end_line);
            indent_len(end_line_text) + end_line_text.trim().len()
        };

        docstring
            .items
            .push(NumPyDocstringItem::Section(NumPySection {
                range: cursor.make_range(
                    section_start,
                    header_col,
                    section_end_line,
                    section_end_col,
                ),
                header,
                body,
            }));
    }

    // Docstring span
    let last_line_idx = cursor.total_lines().saturating_sub(1);
    let last_col = cursor.line_text(last_line_idx).len();
    docstring.range = cursor.make_range(0, 0, last_line_idx, last_col);

    docstring
}

// =============================================================================
// Parameter parsing
// =============================================================================

/// Parse the Parameters section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_parameters(cursor: &mut Cursor, end: usize, entry_indent: usize) -> Vec<NumPyParameter> {
    let mut parameters = Vec::new();

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        // A parameter header is a non-empty line at or below entry indentation.
        // Lines with a colon are split into name/type; lines without a colon
        // are parsed best-effort as a bare name (colon = None).
        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            let col = cursor.current_indent();
            let entry_start = cursor.line;
            let parts = parse_name_and_type(trimmed, cursor.line, col, cursor);

            cursor.advance();
            let desc = collect_description(cursor, end, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc.range.end().raw() as usize)
            };

            parameters.push(NumPyParameter {
                range: cursor.make_range(entry_start, col, entry_end_line, entry_end_col),
                names: parts.names,
                colon: parts.colon,
                r#type: parts.param_type,
                description: desc,
                optional: parts.optional,
                default_keyword: parts.default_keyword,
                default_separator: parts.default_separator,
                default_value: parts.default_value,
            });
            continue;
        }

        cursor.advance();
    }

    parameters
}

/// Result of parsing a parameter header.
struct ParamHeaderParts {
    names: Vec<Spanned<String>>,
    colon: Option<Spanned<String>>,
    param_type: Option<Spanned<String>>,
    optional: Option<Spanned<String>>,
    default_keyword: Option<Spanned<String>>,
    default_separator: Option<Spanned<String>>,
    default_value: Option<Spanned<String>>,
}

/// Parse `"name : type, optional"` into components with precise spans.
///
/// Tolerant of any whitespace around the colon separator.
///
/// `line_idx` is the 0-based line index, `col_base` is the byte column where
/// `text` starts in the raw line.
fn parse_name_and_type(
    text: &str,
    line_idx: usize,
    col_base: usize,
    cursor: &Cursor,
) -> ParamHeaderParts {
    // Find the first colon not inside brackets
    let (name_str, type_str, colon_span, colon_rel) =
        if let Some(colon_pos) = find_entry_colon(text) {
            let before = text[..colon_pos].trim_end();
            let after = &text[colon_pos + 1..];
            let after_trimmed = after.trim();
            let colon_col = col_base + colon_pos;
            let colon = Some(cursor.make_spanned(
                ":".to_string(),
                line_idx,
                colon_col,
                line_idx,
                colon_col + 1,
            ));
            if after_trimmed.is_empty() {
                (before, None, colon, colon_pos)
            } else {
                (before, Some(after_trimmed), colon, colon_pos)
            }
        } else {
            // No separator — whole text is the name
            let names = parse_name_list(text, line_idx, col_base, cursor);
            return ParamHeaderParts {
                names,
                colon: None,
                param_type: None,
                optional: None,
                default_keyword: None,
                default_separator: None,
                default_value: None,
            };
        };

    let names = parse_name_list(name_str, line_idx, col_base, cursor);

    let type_str = match type_str {
        Some(t) if !t.is_empty() => t,
        _ => {
            return ParamHeaderParts {
                names,
                colon: colon_span,
                param_type: None,
                optional: None,
                default_keyword: None,
                default_separator: None,
                default_value: None,
            };
        }
    };

    // Column where the type part starts in the line.
    let after_colon = &text[colon_rel + 1..];
    let ws_after = after_colon.len() - after_colon.trim_start().len();
    let type_col = col_base + colon_rel + 1 + ws_after;

    // Split the type annotation into bracket-aware, comma-separated segments
    // and classify each one.
    let mut optional: Option<Spanned<String>> = None;
    let mut default_keyword: Option<Spanned<String>> = None;
    let mut default_separator: Option<Spanned<String>> = None;
    let mut default_value: Option<Spanned<String>> = None;
    let mut type_parts: Vec<&str> = Vec::new();
    let mut type_parts_end: usize = 0; // byte end offset of last type part in type_str

    for (seg_offset, seg_raw) in split_comma_parts(type_str) {
        let seg = seg_raw.trim();
        if seg.is_empty() {
            continue;
        }

        if seg == "optional" {
            // Record the "optional" span
            let ws_lead = seg_raw.len() - seg_raw.trim_start().len();
            let opt_col = type_col + seg_offset + ws_lead;
            optional = Some(cursor.make_spanned(
                "optional".to_string(),
                line_idx,
                opt_col,
                line_idx,
                opt_col + "optional".len(),
            ));
        } else if seg.starts_with("default") {
            // Record the "default" keyword span
            let ws_lead = seg_raw.len() - seg_raw.trim_start().len();
            let kw_col = type_col + seg_offset + ws_lead;
            default_keyword = Some(cursor.make_spanned(
                "default".to_string(),
                line_idx,
                kw_col,
                line_idx,
                kw_col + "default".len(),
            ));

            // Check for separator (`=` or `:`) and value
            let after_kw = seg["default".len()..].trim_start();
            if let Some(rest) = after_kw.strip_prefix('=') {
                let sep_pos = seg.find('=').unwrap();
                let sep_col = kw_col + sep_pos;
                default_separator = Some(cursor.make_spanned(
                    "=".to_string(),
                    line_idx,
                    sep_col,
                    line_idx,
                    sep_col + 1,
                ));
                let val = rest.trim_start();
                if !val.is_empty() {
                    let val_offset = type_str[seg_offset..].find(val).unwrap_or(0) + seg_offset;
                    let val_col = type_col + val_offset;
                    default_value = Some(cursor.make_spanned(
                        val.to_string(),
                        line_idx,
                        val_col,
                        line_idx,
                        val_col + val.len(),
                    ));
                }
            } else if let Some(rest) = after_kw.strip_prefix(':') {
                let sep_pos = seg.rfind(':').unwrap();
                let sep_col = kw_col + sep_pos;
                default_separator = Some(cursor.make_spanned(
                    ":".to_string(),
                    line_idx,
                    sep_col,
                    line_idx,
                    sep_col + 1,
                ));
                let val = rest.trim_start();
                if !val.is_empty() {
                    let val_offset = type_str[seg_offset..].find(val).unwrap_or(0) + seg_offset;
                    let val_col = type_col + val_offset;
                    default_value = Some(cursor.make_spanned(
                        val.to_string(),
                        line_idx,
                        val_col,
                        line_idx,
                        val_col + val.len(),
                    ));
                }
            } else {
                // No separator — value follows whitespace (e.g., "default True")
                let val = after_kw.trim_start();
                if !val.is_empty() {
                    let val_offset = type_str[seg_offset..].find(val).unwrap_or(0) + seg_offset;
                    let val_col = type_col + val_offset;
                    default_value = Some(cursor.make_spanned(
                        val.to_string(),
                        line_idx,
                        val_col,
                        line_idx,
                        val_col + val.len(),
                    ));
                }
            }
        } else {
            // This is a real type segment
            type_parts.push(seg);
            type_parts_end = seg_offset + seg_raw.trim_end().len();
        }
    }

    let param_type = if type_parts.is_empty() {
        None
    } else {
        // Reconstruct the clean type and locate it in source
        let clean = &type_str[..type_parts_end].trim_end_matches(',').trim_end();
        let tc = type_col;
        Some(cursor.make_spanned(clean.to_string(), line_idx, tc, line_idx, tc + clean.len()))
    };

    ParamHeaderParts {
        names,
        colon: colon_span,
        param_type,
        optional,
        default_keyword,
        default_separator,
        default_value,
    }
}

/// Parse a comma-separated name list like `"x1, x2"` into spanned names.
fn parse_name_list(
    text: &str,
    line_idx: usize,
    col_base: usize,
    cursor: &Cursor,
) -> Vec<Spanned<String>> {
    let mut names = Vec::new();
    let mut byte_pos = 0usize;

    for part in text.split(',') {
        let leading = part.len() - part.trim_start().len();
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            let name_col = col_base + byte_pos + leading;
            names.push(cursor.make_spanned(
                trimmed.to_string(),
                line_idx,
                name_col,
                line_idx,
                name_col + trimmed.len(),
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
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_returns(cursor: &mut Cursor, end: usize, entry_indent: usize) -> Vec<NumPyReturns> {
    let mut returns = Vec::new();

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            let col = cursor.current_indent();
            let entry_start = cursor.line;

            let (name, colon, return_type) = if let Some(colon_pos) = find_entry_colon(trimmed) {
                // Named return: "name : type" (tolerant of whitespace)
                let n = trimmed[..colon_pos].trim_end();
                let after_colon = &trimmed[colon_pos + 1..];
                let t = after_colon.trim();
                let name_col = col;
                let ws_after = after_colon.len() - after_colon.trim_start().len();
                let type_col = col + colon_pos + 1 + ws_after;
                let colon_col = col + colon_pos;
                (
                    Some(cursor.make_spanned(
                        n.to_string(),
                        cursor.line,
                        name_col,
                        cursor.line,
                        name_col + n.len(),
                    )),
                    Some(cursor.make_spanned(
                        ":".to_string(),
                        cursor.line,
                        colon_col,
                        cursor.line,
                        colon_col + 1,
                    )),
                    Some(cursor.make_spanned(
                        t.to_string(),
                        cursor.line,
                        type_col,
                        cursor.line,
                        type_col + t.len(),
                    )),
                )
            } else {
                // Unnamed: type only
                (
                    None,
                    None,
                    Some(cursor.make_spanned(
                        trimmed.to_string(),
                        cursor.line,
                        col,
                        cursor.line,
                        col + trimmed.len(),
                    )),
                )
            };

            cursor.advance();
            let desc = collect_description(cursor, end, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc.range.end().raw() as usize)
            };

            returns.push(NumPyReturns {
                range: cursor.make_range(entry_start, col, entry_end_line, entry_end_col),
                name,
                colon,
                return_type,
                description: desc,
            });
            continue;
        }

        cursor.advance();
    }

    returns
}

// =============================================================================
// Raises parsing
// =============================================================================

/// Parse the Raises section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_raises(cursor: &mut Cursor, end: usize, entry_indent: usize) -> Vec<NumPyException> {
    let mut raises = Vec::new();

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            let col = cursor.current_indent();
            let entry_start = cursor.line;

            let exc_type = cursor.make_spanned(
                trimmed.to_string(),
                cursor.line,
                col,
                cursor.line,
                col + trimmed.len(),
            );

            cursor.advance();
            let desc = collect_description(cursor, end, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc.range.end().raw() as usize)
            };

            raises.push(NumPyException {
                range: cursor.make_range(entry_start, col, entry_end_line, entry_end_col),
                r#type: exc_type,
                description: desc,
            });
            continue;
        }

        cursor.advance();
    }

    raises
}

// =============================================================================
// Free-text section content
// =============================================================================

/// Parse a free-text section body (Notes, Examples, Warnings, Unknown, etc.).
///
/// Preserves blank lines between paragraphs.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_section_content(cursor: &mut Cursor, end: usize) -> Spanned<String> {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();
        content_lines.push(trimmed);
        if !trimmed.is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(cursor.line);
            }
            last_content_line = cursor.line;
        }
        cursor.advance();
    }

    // Trim trailing empty
    while content_lines.last().is_some_and(|l| l.is_empty()) {
        content_lines.pop();
    }
    while content_lines.first().is_some_and(|l| l.is_empty()) {
        content_lines.remove(0);
    }

    let text = content_lines.join("\n");

    if let Some(first) = first_content_line {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content_line);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        cursor.make_spanned(text, first, first_col, last_content_line, last_col)
    } else {
        Spanned::empty_string()
    }
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
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_see_also(cursor: &mut Cursor, end: usize) -> Vec<crate::styles::numpy::ast::SeeAlsoItem> {
    let mut items = Vec::new();

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let col = cursor.current_indent();
        let entry_start = cursor.line;

        // Split on first colon for description (tolerant of whitespace)
        let (names_str, colon, description) = if let Some(colon_pos) = find_entry_colon(trimmed) {
            let after_colon = &trimmed[colon_pos + 1..];
            let desc_text = after_colon.trim();
            let ws_after = after_colon.len() - after_colon.trim_start().len();
            let desc_col = col + colon_pos + 1 + ws_after;
            let colon_col = col + colon_pos;
            (
                trimmed[..colon_pos].trim_end(),
                Some(cursor.make_spanned(
                    ":".to_string(),
                    cursor.line,
                    colon_col,
                    cursor.line,
                    colon_col + 1,
                )),
                Some(cursor.make_spanned(
                    desc_text.to_string(),
                    cursor.line,
                    desc_col,
                    cursor.line,
                    desc_col + desc_text.len(),
                )),
            )
        } else {
            (trimmed, None, None)
        };

        let names = parse_name_list(names_str, cursor.line, col, cursor);
        let entry_end_col = col + trimmed.len();

        items.push(crate::styles::numpy::ast::SeeAlsoItem {
            range: cursor.make_range(entry_start, col, entry_start, entry_end_col),
            names,
            colon,
            description,
        });

        cursor.advance();
    }

    items
}

// =============================================================================
// References parsing
// =============================================================================

/// Parse the References section body.
///
/// Supports RST citation references like `.. [1] Author, Title`.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_references(
    cursor: &mut Cursor,
    end: usize,
) -> Vec<crate::styles::numpy::ast::NumPyReference> {
    use crate::ast::Spanned;

    let mut refs = Vec::new();
    let mut current_number: Spanned<String> = Spanned::dummy(String::new());
    let mut current_directive_marker: Option<Spanned<String>> = None;
    let mut current_open_bracket: Option<Spanned<String>> = None;
    let mut current_close_bracket: Option<Spanned<String>> = None;
    let mut current_content_lines: Vec<&str> = Vec::new();
    let mut current_start_line: Option<usize> = None;
    let mut current_col = 0usize;

    while cursor.line < end {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        // Check for `.. [N]` pattern — tolerate extra whitespace between `..` and `[`
        let is_directive_ref =
            trimmed.starts_with("..") && trimmed[2..].trim_start().starts_with('[');
        if is_directive_ref {
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
                    range: cursor.make_range(start_l, current_col, end_l, end_col),
                    directive_marker: current_directive_marker.clone(),
                    open_bracket: current_open_bracket.clone(),
                    number: current_number.clone(),
                    close_bracket: current_close_bracket.clone(),
                    content: cursor.make_spanned(content, start_l, current_col, end_l, end_col),
                });
            }

            let col = cursor.current_indent();
            // Find actual positions of `[` and `]` within `trimmed`
            let rel_open = trimmed.find('[').unwrap();
            if let Some(rel_close) = trimmed.find(']') {
                // `..` directive marker
                current_directive_marker = Some(cursor.make_spanned(
                    "..".to_string(),
                    cursor.line,
                    col,
                    cursor.line,
                    col + 2,
                ));
                // `[`
                let open_col = col + rel_open;
                current_open_bracket = Some(cursor.make_spanned(
                    "[".to_string(),
                    cursor.line,
                    open_col,
                    cursor.line,
                    open_col + 1,
                ));
                // `]`
                let close_col = col + rel_close;
                current_close_bracket = Some(cursor.make_spanned(
                    "]".to_string(),
                    cursor.line,
                    close_col,
                    cursor.line,
                    close_col + 1,
                ));
                // Number inside brackets, trimmed of whitespace
                let num_raw = &trimmed[rel_open + 1..rel_close];
                let num_str = num_raw.trim();
                let num_ws_lead = num_raw.len() - num_raw.trim_start().len();
                let num_col = col + rel_open + 1 + num_ws_lead;
                current_number = Spanned::new(
                    num_str.to_string(),
                    cursor.make_range(cursor.line, num_col, cursor.line, num_col + num_str.len()),
                );
                let after_bracket = trimmed[rel_close + 1..].trim();
                current_content_lines = vec![after_bracket];
                current_start_line = Some(cursor.line);
                current_col = col;
            }
            cursor.advance();
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
                    range: cursor.make_range(start_l, current_col, end_l, end_col),
                    directive_marker: current_directive_marker.clone(),
                    open_bracket: current_open_bracket.clone(),
                    number: current_number.clone(),
                    close_bracket: current_close_bracket.clone(),
                    content: cursor.make_spanned(content, start_l, current_col, end_l, end_col),
                });
                current_content_lines.clear();
                current_directive_marker = None;
                current_open_bracket = None;
                current_close_bracket = None;
            }
            cursor.advance();
        } else if current_start_line.is_some() {
            // Continuation of current reference
            current_content_lines.push(trimmed);
            cursor.advance();
        } else {
            // Non-RST reference — treat as plain text content
            current_content_lines.push(trimmed);
            if current_start_line.is_none() {
                current_start_line = Some(cursor.line);
                let fallback_num = (refs.len() + 1).to_string();
                let num_col = cursor.current_indent();
                current_number = Spanned::new(
                    fallback_num,
                    cursor.make_range(cursor.line, num_col, cursor.line, num_col),
                );
                current_col = num_col;
                current_directive_marker = None;
                current_open_bracket = None;
                current_close_bracket = None;
            }
            cursor.advance();
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
            range: cursor.make_range(start_l, current_col, end_l, end_col),
            directive_marker: current_directive_marker,
            open_bracket: current_open_bracket,
            number: current_number,
            close_bracket: current_close_bracket,
            content: cursor.make_spanned(content, start_l, current_col, end_l, end_col),
        });
    }

    refs
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_underline() {
        assert!(is_underline("----------"));
        assert!(is_underline("---"));
        assert!(!is_underline(""));
        assert!(!is_underline("not dashes"));
        assert!(!is_underline("---x---"));
    }

    #[test]
    fn test_find_next_section_start() {
        let c1 = Cursor::new("Parameters\n----------");
        assert_eq!(find_next_section_start(&c1, 0), 0);

        // No section
        let c2 = Cursor::new("just text\nmore text");
        assert_eq!(find_next_section_start(&c2, 0), c2.total_lines());

        // Empty line before underline — not a header
        let c3 = Cursor::new("\n----------");
        assert_eq!(find_next_section_start(&c3, 0), c3.total_lines());

        // Single line — no room for underline
        let c4 = Cursor::new("Only one line");
        assert_eq!(find_next_section_start(&c4, 0), c4.total_lines());

        // Start after first section finds second
        let c5 = Cursor::new("Parameters\n----------\nx : int\nReturns\n-------");
        assert_eq!(find_next_section_start(&c5, 0), 0);
        assert_eq!(find_next_section_start(&c5, 2), 3);
    }

    // -- entry colon detection --

    #[test]
    fn test_find_entry_colon() {
        assert_eq!(find_entry_colon("name : int"), Some(5));
        assert_eq!(find_entry_colon("name: int"), Some(4));
        assert_eq!(find_entry_colon("name:int"), Some(4));
        assert_eq!(find_entry_colon("name:"), Some(4));
        assert_eq!(find_entry_colon("name"), None);
        // Colon inside brackets is skipped
        assert_eq!(find_entry_colon("Dict[k: v] : int"), Some(11));
        assert_eq!(find_entry_colon("Dict[k: v]"), None);
    }

    // -- param header detection --

    /// Check whether `trimmed` looks like a parameter header line.\n    /// A parameter header contains a colon (not inside brackets).
    fn is_param_header(trimmed: &str) -> bool {
        find_entry_colon(trimmed).is_some()
    }

    #[test]
    fn test_is_param_header() {
        assert!(is_param_header("x : int"));
        assert!(is_param_header("x: int"));
        assert!(is_param_header("x:int"));
        assert!(is_param_header("x:"));
        assert!(!is_param_header("just a name"));
    }

    // -- comma splitting --

    #[test]
    fn test_split_comma_parts() {
        let parts: Vec<_> = split_comma_parts("int, optional")
            .iter()
            .map(|(_, s)| s.trim())
            .collect();
        assert_eq!(parts, vec!["int", "optional"]);

        let parts: Vec<_> = split_comma_parts("Dict[str, int], optional")
            .iter()
            .map(|(_, s)| s.trim())
            .collect();
        assert_eq!(parts, vec!["Dict[str, int]", "optional"]);

        let parts: Vec<_> = split_comma_parts("int,optional,default True")
            .iter()
            .map(|(_, s)| s.trim())
            .collect();
        assert_eq!(parts, vec!["int", "optional", "default True"]);

        // Offsets are correct
        let parts = split_comma_parts("int, optional");
        assert_eq!(parts[0].0, 0); // "int" starts at 0
        assert_eq!(parts[1].0, 4); // " optional" starts at 4
    }

    // -- parse_name_and_type --

    #[test]
    fn test_parse_name_and_type_basic() {
        let cursor = Cursor::new("x : int");
        let p = parse_name_and_type("x : int", 0, 0, &cursor);
        assert_eq!(p.names[0].value, "x");
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().value, "int");
        assert!(p.optional.is_none());
        assert!(p.default_keyword.is_none());
        assert!(p.default_value.is_none());
    }

    #[test]
    fn test_parse_name_and_type_optional() {
        let cursor = Cursor::new("x : int, optional");
        let p = parse_name_and_type("x : int, optional", 0, 0, &cursor);
        assert_eq!(p.names[0].value, "x");
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().value, "int");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_optional_no_space() {
        let cursor = Cursor::new("x : int,optional");
        let p = parse_name_and_type("x : int,optional", 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().value, "int");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_default_space() {
        let cursor = Cursor::new("x : int, default True");
        let p = parse_name_and_type("x : int, default True", 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().value, "int");
        assert_eq!(p.default_keyword.as_ref().unwrap().value, "default");
        assert!(p.default_separator.is_none()); // space-separated, no = or :
        assert_eq!(p.default_value.unwrap().value, "True");
    }

    #[test]
    fn test_parse_name_and_type_default_equals() {
        let cursor = Cursor::new("x : int, default=True");
        let p = parse_name_and_type("x : int, default=True", 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().value, "int");
        assert_eq!(p.default_keyword.as_ref().unwrap().value, "default");
        assert_eq!(p.default_separator.as_ref().unwrap().value, "=");
        assert_eq!(p.default_value.unwrap().value, "True");
    }

    #[test]
    fn test_parse_name_and_type_default_colon() {
        let cursor = Cursor::new("x : int, default: True");
        let p = parse_name_and_type("x : int, default: True", 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().value, "int");
        assert_eq!(p.default_keyword.as_ref().unwrap().value, "default");
        assert_eq!(p.default_separator.as_ref().unwrap().value, ":");
        assert_eq!(p.default_value.unwrap().value, "True");
    }

    #[test]
    fn test_parse_name_and_type_default_bare() {
        // "default" alone with no value
        let cursor = Cursor::new("x : int, default");
        let p = parse_name_and_type("x : int, default", 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().value, "int");
        assert_eq!(p.default_keyword.as_ref().unwrap().value, "default");
        assert!(p.default_separator.is_none());
        assert!(p.default_value.is_none());
    }

    #[test]
    fn test_parse_name_and_type_complex() {
        let cursor = Cursor::new("x : Dict[str, int], optional");
        let p = parse_name_and_type("x : Dict[str, int], optional", 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().value, "Dict[str, int]");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_no_colon() {
        let cursor = Cursor::new("x");
        let p = parse_name_and_type("x", 0, 0, &cursor);
        assert_eq!(p.names[0].value, "x");
        assert!(p.colon.is_none());
        assert!(p.param_type.is_none());
        assert!(p.optional.is_none());
        assert!(p.default_keyword.is_none());
        assert!(p.default_value.is_none());
    }
}
