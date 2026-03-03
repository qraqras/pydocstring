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

use crate::ast::TextRange;
use crate::cursor::{LineCursor, indent_columns, indent_len};
use crate::styles::numpy::ast::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyDocstringItem, NumPyException,
    NumPyMethod, NumPyParameter, NumPyReturns, NumPySection, NumPySectionBody, NumPySectionHeader,
    NumPySectionKind, NumPyWarning,
};
use crate::styles::utils::{find_entry_colon, find_matching_close, split_comma_parts};

// =============================================================================
// Section detection
// =============================================================================

/// Check if a trimmed line is a NumPy-style section underline (only dashes).
fn is_underline(trimmed: &str) -> bool {
    !trimmed.is_empty() && trimmed.bytes().all(|b| b == b'-')
}

/// Try to parse a NumPy-style section header at `cursor.line`.
///
/// A section header is a non-empty line immediately followed by a
/// line consisting only of dashes.  Does **not** advance the cursor.
fn try_parse_numpy_header(cursor: &LineCursor) -> Option<NumPySectionHeader> {
    let header_trimmed = cursor.current_trimmed();
    if header_trimmed.is_empty() {
        return None;
    }
    if cursor.line + 1 >= cursor.total_lines() {
        return None;
    }
    let underline_line = cursor.line_text(cursor.line + 1);
    let underline_trimmed = underline_line.trim();
    if !is_underline(underline_trimmed) {
        return None;
    }

    let header_col = cursor.current_indent();
    let underline_col = indent_len(underline_line);
    let normalized = header_trimmed.to_ascii_lowercase();
    let kind = NumPySectionKind::from_name(&normalized);

    Some(NumPySectionHeader {
        range: cursor.make_range(
            cursor.line,
            header_col,
            cursor.line + 1,
            underline_col + underline_trimmed.len(),
        ),
        kind,
        name: cursor.make_line_range(cursor.line, header_col, header_trimmed.len()),
        underline: cursor.make_line_range(cursor.line + 1, underline_col, underline_trimmed.len()),
    })
}

// =============================================================================
// Description collector
// =============================================================================

/// Collect indented description lines starting at `cursor.line`.
///
/// Preserves blank lines between paragraphs. Stops at non-empty lines at or
/// below `entry_indent`, section headers, or EOF.
///
/// On return, `cursor.line` points to the first unconsumed line.
fn collect_description(cursor: &mut LineCursor, entry_indent: usize) -> TextRange {
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while !cursor.is_eof() {
        let line = cursor.current_line_text();
        // Non-empty line at or below entry indentation signals end of description
        if !line.trim().is_empty() && indent_columns(line) <= entry_indent {
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

    if let Some(first) = first_content_line {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content_line);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        cursor.make_range(first, first_col, last_content_line, last_col)
    } else {
        TextRange::empty()
    }
}

// =============================================================================
// Main parser
// =============================================================================

/// Parse a NumPy-style docstring.
pub fn parse_numpy(input: &str) -> NumPyDocstring {
    let mut cursor = LineCursor::new(input);
    let mut docstring = NumPyDocstring::new(input);

    if cursor.total_lines() == 0 {
        return docstring;
    }

    // --- Skip leading blank lines ---
    cursor.skip_blanks();
    if cursor.is_eof() {
        return docstring;
    }

    // --- Summary (all lines until blank line or section header) ---
    if try_parse_numpy_header(&cursor).is_none() {
        let trimmed = cursor.current_trimmed();
        if !trimmed.is_empty() {
            let start_line = cursor.line;
            let start_col = cursor.current_indent();
            let mut last_line = start_line;

            while !cursor.is_eof() {
                if try_parse_numpy_header(&cursor).is_some() {
                    break;
                }
                let t = cursor.current_trimmed();
                if t.is_empty() {
                    break;
                }
                last_line = cursor.line;
                cursor.advance();
            }

            let last_text = cursor.line_text(last_line);
            let last_col = indent_len(last_text) + last_text.trim().len();
            let range = cursor.make_range(start_line, start_col, last_line, last_col);
            if !range.is_empty() {
                docstring.summary = Some(range);
            }
        }
    }

    // skip blanks
    cursor.skip_blanks();

    // --- Deprecation directive ---
    if !cursor.is_eof() && try_parse_numpy_header(&cursor).is_none() {
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
            let directive_marker = Some(cursor.make_line_range(cursor.line, col, 2));
            // `deprecated` at col+3..col+13
            let kw_col = col + 3;
            let keyword = Some(cursor.make_line_range(cursor.line, kw_col, 10));
            // `::` at col+13..col+15
            let dc_col = col + 13;
            let double_colon = Some(cursor.make_line_range(cursor.line, dc_col, 2));

            let version_spanned =
                cursor.make_line_range(cursor.line, version_col, version_str.len());

            let dep_start_line = cursor.line;
            cursor.advance();

            // Collect indented body lines
            let desc_spanned = collect_description(&mut cursor, col);

            // Compute deprecation span
            let (dep_end_line, dep_end_col) = if desc_spanned.is_empty() {
                (dep_start_line, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc_spanned.end().raw() as usize)
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
            cursor.skip_blanks();
        }
    }

    // --- Extended summary ---
    if !cursor.is_eof() && try_parse_numpy_header(&cursor).is_none() {
        let start_line = cursor.line;
        let mut desc_lines: Vec<&str> = Vec::new();
        let mut last_non_empty_line = cursor.line;

        while !cursor.is_eof() {
            if try_parse_numpy_header(&cursor).is_some() {
                break;
            }
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
            docstring.extended_summary =
                Some(cursor.make_range(start_line, first_col, last_non_empty_line, last_col));
        }
    }

    // --- Sections ---
    while !cursor.is_eof() {
        // Skip blank lines between sections
        if cursor.current_trimmed().is_empty() {
            cursor.advance();
            continue;
        }

        // Detect section header
        let Some(header) = try_parse_numpy_header(&cursor) else {
            // Stray line (not a section header)
            docstring.items.push(NumPyDocstringItem::StrayLine(
                cursor.current_trimmed_range(),
            ));
            cursor.advance();
            continue;
        };

        let header_indent = cursor.current_indent_columns();
        let section_kind = header.kind;

        cursor.line += 2; // skip header + underline

        // Parse section body (branching by kind)
        #[rustfmt::skip]
        let body = match section_kind {
            NumPySectionKind::Parameters      => NumPySectionBody::Parameters     (parse_parameters     (&mut cursor, header_indent)),
            NumPySectionKind::Returns         => NumPySectionBody::Returns        (parse_returns        (&mut cursor, header_indent)),
            NumPySectionKind::Raises          => NumPySectionBody::Raises         (parse_raises         (&mut cursor, header_indent)),
            NumPySectionKind::Yields          => NumPySectionBody::Yields         (parse_returns        (&mut cursor, header_indent)),
            NumPySectionKind::Receives        => NumPySectionBody::Receives       (parse_parameters     (&mut cursor, header_indent)),
            NumPySectionKind::OtherParameters => NumPySectionBody::OtherParameters(parse_parameters     (&mut cursor, header_indent)),
            NumPySectionKind::Warns           => NumPySectionBody::Warns          (parse_warns          (&mut cursor, header_indent)),
            NumPySectionKind::Notes           => NumPySectionBody::Notes          (parse_section_content(&mut cursor)),
            NumPySectionKind::Examples        => NumPySectionBody::Examples       (parse_section_content(&mut cursor)),
            NumPySectionKind::Warnings        => NumPySectionBody::Warnings       (parse_section_content(&mut cursor)),
            NumPySectionKind::SeeAlso         => NumPySectionBody::SeeAlso        (parse_see_also       (&mut cursor)),
            NumPySectionKind::References      => NumPySectionBody::References     (parse_references     (&mut cursor)),
            NumPySectionKind::Attributes      => NumPySectionBody::Attributes     (parse_attributes     (&mut cursor, header_indent)),
            NumPySectionKind::Methods         => NumPySectionBody::Methods        (parse_methods        (&mut cursor, header_indent)),
            NumPySectionKind::Unknown         => NumPySectionBody::Unknown        (parse_section_content(&mut cursor)),
        };

        let section_range = cursor.span_back_from_cursor(header.range.start().raw() as usize);

        docstring
            .items
            .push(NumPyDocstringItem::Section(NumPySection {
                range: section_range,
                header,
                body,
            }));
    }

    // Docstring span
    docstring.range = cursor.full_range();

    docstring
}

// =============================================================================
// Warns parsing
// =============================================================================

/// Parse the Warns section body.
///
/// Reuses `parse_raises` and converts each `NumPyException` to `NumPyWarning`.
fn parse_warns(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyWarning> {
    parse_raises(cursor, entry_indent)
        .into_iter()
        .map(|e| NumPyWarning {
            range: e.range,
            r#type: e.r#type,
            colon: e.colon,
            description: e.description,
        })
        .collect()
}

// =============================================================================
// Attributes parsing
// =============================================================================

/// Parse the Attributes section body.
///
/// Reuses `parse_parameters` and converts each `NumPyParameter` to `NumPyAttribute`.
fn parse_attributes(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyAttribute> {
    parse_parameters(cursor, entry_indent)
        .into_iter()
        .map(|p| NumPyAttribute {
            range: p.range,
            name: p.names.into_iter().next().unwrap_or_else(TextRange::empty),
            colon: p.colon,
            r#type: p.r#type,
            description: p.description,
        })
        .collect()
}

// =============================================================================
// Methods parsing
// =============================================================================

/// Parse the Methods section body.
///
/// Reuses `parse_parameters` and converts each `NumPyParameter` to `NumPyMethod`.
fn parse_methods(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyMethod> {
    parse_parameters(cursor, entry_indent)
        .into_iter()
        .map(|p| NumPyMethod {
            range: p.range,
            name: p.names.into_iter().next().unwrap_or_else(TextRange::empty),
            colon: p.colon,
            description: p.description,
        })
        .collect()
}

// =============================================================================
// Parameter parsing
// =============================================================================

/// Parse the Parameters section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_parameters(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyParameter> {
    let mut parameters = Vec::new();

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        // A parameter header is a non-empty line at or below entry indentation.
        // Lines with a colon are split into name/type; lines without a colon
        // are parsed best-effort as a bare name (colon = None).
        if !trimmed.is_empty() && indent_columns(line) <= entry_indent {
            let col = cursor.current_indent();
            let entry_start = cursor.line;
            let parts = parse_name_and_type(trimmed, cursor.line, col, cursor);

            // Advance past all header lines (may span multiple for multi-line types)
            cursor.line = parts.header_end_line + 1;
            let desc = collect_description(cursor, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.is_empty() {
                if parts.header_end_line > entry_start {
                    // Multi-line header: compute end from last header line
                    let last_line = cursor.line_text(parts.header_end_line);
                    let last_trimmed = last_line.trim();
                    (
                        parts.header_end_line,
                        indent_len(last_line) + last_trimmed.len(),
                    )
                } else {
                    (entry_start, col + trimmed.len())
                }
            } else {
                cursor.offset_to_line_col(desc.end().raw() as usize)
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
    names: Vec<TextRange>,
    colon: Option<TextRange>,
    param_type: Option<TextRange>,
    optional: Option<TextRange>,
    default_keyword: Option<TextRange>,
    default_separator: Option<TextRange>,
    default_value: Option<TextRange>,
    /// Line index where the header ends (may differ from start for multi-line types).
    header_end_line: usize,
}

/// Parse `"name : type, optional"` into components with precise spans.
///
/// Tolerant of any whitespace around the colon separator.
/// Supports multi-line type annotations with brackets spanning multiple lines
/// (e.g., `Dict[str,\n    int]`).
///
/// `line_idx` is the 0-based line index, `col_base` is the byte column where
/// `text` starts in the raw line.
fn parse_name_and_type(
    text: &str,
    line_idx: usize,
    col_base: usize,
    cursor: &LineCursor,
) -> ParamHeaderParts {
    // Find the first colon not inside brackets
    let (name_str, colon_span, colon_rel) = if let Some(colon_pos) = find_entry_colon(text) {
        let before = text[..colon_pos].trim_end();
        let colon_col = col_base + colon_pos;
        let colon = Some(cursor.make_line_range(line_idx, colon_col, 1));
        (before, colon, Some(colon_pos))
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
            header_end_line: line_idx,
        };
    };

    let names = parse_name_list(name_str, line_idx, col_base, cursor);

    let colon_rel = colon_rel.unwrap();
    let after_colon = &text[colon_rel + 1..];
    let after_trimmed = after_colon.trim();

    if after_trimmed.is_empty() {
        return ParamHeaderParts {
            names,
            colon: colon_span,
            param_type: None,
            optional: None,
            default_keyword: None,
            default_separator: None,
            default_value: None,
            header_end_line: line_idx,
        };
    }

    // Determine the full type text, potentially spanning multiple lines.
    // `after_trimmed` is a subslice of cursor.source(), so we can use
    // substr_offset to get its absolute byte position.
    let type_abs_start = cursor.substr_offset(after_trimmed);

    // Check if brackets are balanced on the current line.
    let opens: usize = after_trimmed
        .bytes()
        .filter(|&b| matches!(b, b'(' | b'[' | b'{' | b'<'))
        .count();
    let closes: usize = after_trimmed
        .bytes()
        .filter(|&b| matches!(b, b')' | b']' | b'}' | b'>'))
        .count();

    let (type_text, header_end_line) = if opens > closes {
        // Unclosed bracket — find the first opening bracket and its match
        let first_open_rel = after_trimmed
            .bytes()
            .position(|b| matches!(b, b'(' | b'[' | b'{' | b'<'))
            .unwrap();
        let abs_open = type_abs_start + first_open_rel;
        if let Some(abs_close) = find_matching_close(cursor.source(), abs_open) {
            let close_line_idx = cursor.offset_to_line_col(abs_close).0;
            // Include everything from type start through end of close bracket's line
            let close_line_text = cursor.line_text(close_line_idx);
            let close_line_end =
                cursor.substr_offset(close_line_text) + close_line_text.trim_end().len();
            let full = &cursor.source()[type_abs_start..close_line_end];
            (full, close_line_idx)
        } else {
            // No matching close found — treat as single-line
            (after_trimmed, line_idx)
        }
    } else {
        (after_trimmed, line_idx)
    };

    // Now classify segments within `type_text` using bracket-aware comma splitting.
    let mut optional: Option<TextRange> = None;
    let mut default_keyword: Option<TextRange> = None;
    let mut default_separator: Option<TextRange> = None;
    let mut default_value: Option<TextRange> = None;
    let mut type_parts_end: usize = 0; // byte end offset of last type part in type_text

    for (seg_offset, seg_raw) in split_comma_parts(type_text) {
        let seg = seg_raw.trim();
        if seg.is_empty() {
            continue;
        }

        if seg == "optional" {
            let seg_abs =
                type_abs_start + seg_offset + (seg_raw.len() - seg_raw.trim_start().len());
            optional = Some(TextRange::from_offset_len(seg_abs, "optional".len()));
        } else if let Some(stripped) = seg.strip_prefix("default") {
            let ws_lead = seg_raw.len() - seg_raw.trim_start().len();
            let kw_abs = type_abs_start + seg_offset + ws_lead;
            default_keyword = Some(TextRange::from_offset_len(kw_abs, "default".len()));

            let after_kw = stripped.trim_start();
            if let Some(rest) = after_kw.strip_prefix('=') {
                let sep_pos = seg.find('=').unwrap();
                let sep_abs = kw_abs + sep_pos;
                default_separator = Some(TextRange::from_offset_len(sep_abs, 1));
                let val = rest.trim_start();
                if !val.is_empty() {
                    let val_abs = cursor.substr_offset(val);
                    default_value = Some(TextRange::from_offset_len(val_abs, val.len()));
                }
            } else if let Some(rest) = after_kw.strip_prefix(':') {
                let sep_pos = seg.rfind(':').unwrap();
                let sep_abs = kw_abs + sep_pos;
                default_separator = Some(TextRange::from_offset_len(sep_abs, 1));
                let val = rest.trim_start();
                if !val.is_empty() {
                    let val_abs = cursor.substr_offset(val);
                    default_value = Some(TextRange::from_offset_len(val_abs, val.len()));
                }
            } else {
                let val = after_kw.trim_start();
                if !val.is_empty() {
                    let val_abs = cursor.substr_offset(val);
                    default_value = Some(TextRange::from_offset_len(val_abs, val.len()));
                }
            }
        } else {
            // Real type segment
            type_parts_end = seg_offset + seg_raw.trim_end().len();
        }
    }

    let param_type = if type_parts_end == 0 {
        None
    } else {
        let clean = type_text[..type_parts_end].trim_end_matches(',').trim_end();
        Some(TextRange::from_offset_len(type_abs_start, clean.len()))
    };

    ParamHeaderParts {
        names,
        colon: colon_span,
        param_type,
        optional,
        default_keyword,
        default_separator,
        default_value,
        header_end_line,
    }
}

/// Parse a comma-separated name list like `"x1, x2"` into spanned names.
fn parse_name_list(
    text: &str,
    line_idx: usize,
    col_base: usize,
    cursor: &LineCursor,
) -> Vec<TextRange> {
    let mut names = Vec::new();
    let mut byte_pos = 0usize;

    for part in text.split(',') {
        let leading = part.len() - part.trim_start().len();
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            let name_col = col_base + byte_pos + leading;
            names.push(cursor.make_line_range(line_idx, name_col, trimmed.len()));
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
fn parse_returns(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyReturns> {
    let mut returns = Vec::new();

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_columns(line) <= entry_indent {
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
                    Some(cursor.make_line_range(cursor.line, name_col, n.len())),
                    Some(cursor.make_line_range(cursor.line, colon_col, 1)),
                    Some(cursor.make_line_range(cursor.line, type_col, t.len())),
                )
            } else {
                // Unnamed: type only
                (None, None, Some(cursor.current_trimmed_range()))
            };

            cursor.advance();
            let desc = collect_description(cursor, entry_indent);

            let (entry_end_line, entry_end_col) = if desc.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc.end().raw() as usize)
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
/// Supports both bare exception types and `ExcType : description` format.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_raises(cursor: &mut LineCursor, entry_indent: usize) -> Vec<NumPyException> {
    let mut raises = Vec::new();

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if !trimmed.is_empty() && indent_columns(line) <= entry_indent {
            let col = cursor.current_indent();
            let entry_start = cursor.line;

            let (exc_type, colon, first_desc) = if let Some(colon_pos) = find_entry_colon(trimmed) {
                let type_str = trimmed[..colon_pos].trim_end();
                let after_colon = &trimmed[colon_pos + 1..];
                let desc_str = after_colon.trim();
                let ws_after = after_colon.len() - after_colon.trim_start().len();
                let colon_col = col + colon_pos;
                let desc_col = col + colon_pos + 1 + ws_after;

                let et = cursor.make_line_range(cursor.line, col, type_str.len());
                let c = Some(cursor.make_line_range(cursor.line, colon_col, 1));
                let fd = if desc_str.is_empty() {
                    TextRange::empty()
                } else {
                    cursor.make_line_range(cursor.line, desc_col, desc_str.len())
                };
                (et, c, fd)
            } else {
                // Bare type, no colon
                let et = cursor.current_trimmed_range();
                (et, None, TextRange::empty())
            };

            cursor.advance();
            let cont_desc = collect_description(cursor, entry_indent);

            // Merge first-line description with continuation
            let desc = if first_desc.is_empty() {
                cont_desc
            } else if cont_desc.is_empty() {
                first_desc
            } else {
                TextRange::new(first_desc.start(), cont_desc.end())
            };

            let (entry_end_line, entry_end_col) = if desc.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(desc.end().raw() as usize)
            };

            raises.push(NumPyException {
                range: cursor.make_range(entry_start, col, entry_end_line, entry_end_col),
                r#type: exc_type,
                colon,
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
fn parse_section_content(cursor: &mut LineCursor) -> TextRange {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
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

    if let Some(first) = first_content_line {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content_line);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        cursor.make_range(first, first_col, last_content_line, last_col)
    } else {
        TextRange::empty()
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
fn parse_see_also(cursor: &mut LineCursor) -> Vec<crate::styles::numpy::ast::SeeAlsoItem> {
    let mut items = Vec::new();

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
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
                Some(cursor.make_line_range(cursor.line, colon_col, 1)),
                Some(cursor.make_line_range(cursor.line, desc_col, desc_text.len())),
            )
        } else {
            (trimmed, None, None)
        };

        let names = parse_name_list(names_str, cursor.line, col, cursor);

        items.push(crate::styles::numpy::ast::SeeAlsoItem {
            range: cursor.make_line_range(entry_start, col, trimmed.len()),
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
fn parse_references(cursor: &mut LineCursor) -> Vec<crate::styles::numpy::ast::NumPyReference> {
    let mut refs = Vec::new();
    let mut current_number: TextRange = TextRange::empty();
    let mut current_directive_marker: Option<TextRange> = None;
    let mut current_open_bracket: Option<TextRange> = None;
    let mut current_close_bracket: Option<TextRange> = None;
    let mut current_content_lines: Vec<&str> = Vec::new();
    let mut current_start_line: Option<usize> = None;
    let mut current_col = 0usize;

    while !cursor.is_eof() {
        if try_parse_numpy_header(cursor).is_some() {
            break;
        }
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
                    directive_marker: current_directive_marker,
                    open_bracket: current_open_bracket,
                    number: current_number,
                    close_bracket: current_close_bracket,
                    content: cursor.make_range(start_l, current_col, end_l, end_col),
                });
            }

            let col = cursor.current_indent();
            // Find actual positions of `[` and `]` — use bracket-aware matching
            let rel_open = trimmed.find('[').unwrap();
            let abs_open = cursor.substr_offset(trimmed) + rel_open;
            if let Some(abs_close) = find_matching_close(cursor.source(), abs_open) {
                // `..` directive marker
                current_directive_marker =
                    Some(cursor.make_range(cursor.line, col, cursor.line, col + 2));
                // `[`
                current_open_bracket = Some(TextRange::from_offset_len(abs_open, 1));
                // `]`
                current_close_bracket = Some(TextRange::from_offset_len(abs_close, 1));
                // Number inside brackets, trimmed of whitespace
                let num_raw = &cursor.source()[abs_open + 1..abs_close];
                let num_str = num_raw.trim();
                if !num_str.is_empty() {
                    let num_abs = cursor.substr_offset(num_str);
                    current_number = TextRange::from_offset_len(num_abs, num_str.len());
                } else {
                    current_number = TextRange::empty();
                }
                let close_line_text = cursor.line_text(cursor.offset_to_line_col(abs_close).0);
                let close_line_end = cursor.substr_offset(close_line_text) + close_line_text.len();
                let after_bracket = cursor.source()[abs_close + 1..close_line_end].trim();
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
                    directive_marker: current_directive_marker,
                    open_bracket: current_open_bracket,
                    number: current_number,
                    close_bracket: current_close_bracket,
                    content: cursor.make_range(start_l, current_col, end_l, end_col),
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
                let num_col = cursor.current_indent();
                current_number = cursor.make_range(cursor.line, num_col, cursor.line, num_col);
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
            content: cursor.make_range(start_l, current_col, end_l, end_col),
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
    fn test_try_parse_numpy_header() {
        let c1 = LineCursor::new("Parameters\n----------");
        assert!(try_parse_numpy_header(&c1).is_some());
        assert_eq!(
            try_parse_numpy_header(&c1).unwrap().kind,
            NumPySectionKind::Parameters
        );

        // No section
        let c2 = LineCursor::new("just text\nmore text");
        assert!(try_parse_numpy_header(&c2).is_none());

        // Empty line before underline — not a header
        let c3 = LineCursor::new("\n----------");
        assert!(try_parse_numpy_header(&c3).is_none());

        // Single line — no room for underline
        let c4 = LineCursor::new("Only one line");
        assert!(try_parse_numpy_header(&c4).is_none());

        // Header at non-zero line
        let mut c5 = LineCursor::new("Parameters\n----------\nx : int\nReturns\n-------");
        assert!(try_parse_numpy_header(&c5).is_some());
        c5.line = 3;
        assert!(try_parse_numpy_header(&c5).is_some());
        assert_eq!(
            try_parse_numpy_header(&c5).unwrap().kind,
            NumPySectionKind::Returns
        );
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

    // -- parse_name_and_type --

    #[test]
    fn test_parse_name_and_type_basic() {
        let src = "x : int";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.names[0].source_text(src), "x");
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert!(p.optional.is_none());
        assert!(p.default_keyword.is_none());
        assert!(p.default_value.is_none());
    }

    #[test]
    fn test_parse_name_and_type_optional() {
        let src = "x : int, optional";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.names[0].source_text(src), "x");
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_optional_no_space() {
        let src = "x : int,optional";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_default_space() {
        let src = "x : int, default True";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert_eq!(
            p.default_keyword.as_ref().unwrap().source_text(src),
            "default"
        );
        assert!(p.default_separator.is_none()); // space-separated, no = or :
        assert_eq!(p.default_value.unwrap().source_text(src), "True");
    }

    #[test]
    fn test_parse_name_and_type_default_equals() {
        let src = "x : int, default=True";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert_eq!(
            p.default_keyword.as_ref().unwrap().source_text(src),
            "default"
        );
        assert_eq!(p.default_separator.as_ref().unwrap().source_text(src), "=");
        assert_eq!(p.default_value.unwrap().source_text(src), "True");
    }

    #[test]
    fn test_parse_name_and_type_default_colon() {
        let src = "x : int, default: True";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert_eq!(
            p.default_keyword.as_ref().unwrap().source_text(src),
            "default"
        );
        assert_eq!(p.default_separator.as_ref().unwrap().source_text(src), ":");
        assert_eq!(p.default_value.unwrap().source_text(src), "True");
    }

    #[test]
    fn test_parse_name_and_type_default_bare() {
        // "default" alone with no value
        let src = "x : int, default";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.param_type.unwrap().source_text(src), "int");
        assert_eq!(
            p.default_keyword.as_ref().unwrap().source_text(src),
            "default"
        );
        assert!(p.default_separator.is_none());
        assert!(p.default_value.is_none());
    }

    #[test]
    fn test_parse_name_and_type_complex() {
        let src = "x : Dict[str, int], optional";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert!(p.colon.is_some());
        assert_eq!(p.param_type.unwrap().source_text(src), "Dict[str, int]");
        assert!(p.optional.is_some());
    }

    #[test]
    fn test_parse_name_and_type_no_colon() {
        let src = "x";
        let cursor = LineCursor::new(src);
        let p = parse_name_and_type(src, 0, 0, &cursor);
        assert_eq!(p.names[0].source_text(src), "x");
        assert!(p.colon.is_none());
        assert!(p.param_type.is_none());
        assert!(p.optional.is_none());
        assert!(p.default_keyword.is_none());
        assert!(p.default_value.is_none());
    }
}
