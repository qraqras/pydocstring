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
    NumPyMethod, NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader, NumPySectionKind, NumPyWarning, SeeAlsoItem,
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
// Description collector (used only for deprecation directive body)
// =============================================================================

/// Collect indented description lines starting at `cursor.line`.
///
/// Preserves blank lines between paragraphs. Stops at non-empty lines at or
/// below `entry_indent_cols` visual columns, section headers, or EOF.
///
/// On return, `cursor.line` points to the first unconsumed line.
///
/// NOTE: This is used **only** for the deprecation directive body, which needs
/// eager multi-line collection. Section body parsing uses per-line functions.
fn collect_description(cursor: &mut LineCursor, entry_indent_cols: usize) -> Option<TextRange> {
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while !cursor.is_eof() {
        let line = cursor.current_line_text();
        if !line.trim().is_empty() && indent_columns(line) <= entry_indent_cols {
            break;
        }
        if !line.trim().is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(cursor.line);
            }
            last_content_line = cursor.line;
        }
        cursor.advance();
    }

    first_content_line.map(|first| {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content_line);
        let last_col = indent_len(last_line) + last_line.trim().len();
        cursor.make_range(first, first_col, last_content_line, last_col)
    })
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
            let desc_spanned = collect_description(&mut cursor, indent_columns(line));

            // Compute deprecation span
            let (dep_end_line, dep_end_col) = match &desc_spanned {
                None => (dep_start_line, col + trimmed.len()),
                Some(d) => cursor.offset_to_line_col(d.end().raw() as usize),
            };

            docstring.deprecation = Some(NumPyDeprecation {
                range: cursor.make_range(dep_start_line, col, dep_end_line, dep_end_col),
                directive_marker,
                keyword,
                double_colon,
                version: version_spanned,
                description: desc_spanned.unwrap_or_else(TextRange::empty),
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

    // --- Section state ---
    let mut current_header: Option<NumPySectionHeader> = None;
    let mut current_body: Option<NumPySectionBody> = None;
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        // --- Blank lines ---
        if cursor.current_trimmed().is_empty() {
            cursor.advance();
            continue;
        }

        // --- Detect section header ---
        if let Some(header) = try_parse_numpy_header(&cursor) {
            // Flush previous section
            if let Some(prev_header) = current_header.take() {
                flush_section(
                    &cursor,
                    &mut docstring,
                    prev_header,
                    current_body.take().unwrap(),
                );
            }

            // Start new section
            current_body = Some(NumPySectionBody::new(header.kind));
            current_header = Some(header);
            entry_indent = None;
            cursor.line += 2; // skip header + underline
            continue;
        }

        // --- Process line based on current state ---
        if let Some(ref mut body) = current_body {
            #[rustfmt::skip]
            match body {
                NumPySectionBody::Parameters(v) => process_parameter_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::OtherParameters(v) => process_parameter_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Receives(v) => process_parameter_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Returns(v) => process_returns_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Yields(v) => process_returns_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Raises(v) => process_raises_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Warns(v) => process_warning_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Attributes(v) => process_attribute_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Methods(v) => process_method_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::SeeAlso(v) => process_see_also_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::References(v) => process_reference_line(&cursor, v, &mut entry_indent),
                NumPySectionBody::Notes(r) => process_freetext_line(&cursor, r),
                NumPySectionBody::Examples(r) => process_freetext_line(&cursor, r),
                NumPySectionBody::Warnings(r) => process_freetext_line(&cursor, r),
                NumPySectionBody::Unknown(r) => process_freetext_line(&cursor, r),
            };
        } else {
            // Stray line (outside any section in post-section phase)
            docstring.items.push(NumPyDocstringItem::StrayLine(
                cursor.current_trimmed_range(),
            ));
        }

        cursor.advance();
    }

    // Flush final section
    if let Some(header) = current_header.take() {
        flush_section(
            &cursor,
            &mut docstring,
            header,
            current_body.take().unwrap(),
        );
    }

    // Docstring span
    docstring.range = cursor.full_range();

    docstring
}

// =============================================================================
// Section flush
// =============================================================================

/// Flush a completed section into the docstring.
fn flush_section(
    cursor: &LineCursor,
    docstring: &mut NumPyDocstring,
    header: NumPySectionHeader,
    body: NumPySectionBody,
) {
    let header_start = header.range.start().raw() as usize;
    let range = cursor.span_back_from_cursor(header_start);
    docstring
        .items
        .push(NumPyDocstringItem::Section(NumPySection {
            range,
            header,
            body,
        }));
}

// =============================================================================
// Entry header parsing
// =============================================================================

/// Result of parsing a parameter header.
struct ParamHeaderParts {
    names: Vec<TextRange>,
    colon: Option<TextRange>,
    param_type: Option<TextRange>,
    optional: Option<TextRange>,
    default_keyword: Option<TextRange>,
    default_separator: Option<TextRange>,
    default_value: Option<TextRange>,
}

/// Parse `"name : type, optional"` into components with precise spans.
///
/// Tolerant of any whitespace around the colon separator.
/// Single-line only — multi-line type annotations are not supported.
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
    let Some(colon_pos) = find_entry_colon(text) else {
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

    let name_str = text[..colon_pos].trim_end();
    let colon_col = col_base + colon_pos;
    let colon_span = Some(cursor.make_line_range(line_idx, colon_col, 1));
    let names = parse_name_list(name_str, line_idx, col_base, cursor);

    let after_colon = &text[colon_pos + 1..];
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
        };
    }

    let type_abs_start = cursor.substr_offset(after_trimmed);
    let type_text = after_trimmed;

    // Classify segments using bracket-aware comma splitting.
    let mut optional: Option<TextRange> = None;
    let mut default_keyword: Option<TextRange> = None;
    let mut default_separator: Option<TextRange> = None;
    let mut default_value: Option<TextRange> = None;
    let mut type_parts_end: usize = 0;

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
// Per-line section body processors
// =============================================================================

/// Extend a description field with a continuation range.
fn extend_description(description: &mut Option<TextRange>, range: &mut TextRange, cont: TextRange) {
    match description {
        Some(desc) => desc.extend(cont),
        None => *description = Some(cont),
    }
    *range = TextRange::new(range.start(), cont.end());
}

/// Process one content line for a Parameters / OtherParameters / Receives section.
fn process_parameter_line(
    cursor: &LineCursor,
    params: &mut Vec<NumPyParameter>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = params.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();
    let parts = parse_name_and_type(trimmed, cursor.line, col, cursor);

    let entry_range = cursor.current_trimmed_range();
    params.push(NumPyParameter {
        range: entry_range,
        names: parts.names,
        colon: parts.colon,
        r#type: parts.param_type,
        description: None,
        optional: parts.optional,
        default_keyword: parts.default_keyword,
        default_separator: parts.default_separator,
        default_value: parts.default_value,
    });
}

/// Process one content line for a Returns / Yields section.
fn process_returns_line(
    cursor: &LineCursor,
    returns: &mut Vec<NumPyReturns>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = returns.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();

    let (name, colon, return_type) = if let Some(colon_pos) = find_entry_colon(trimmed) {
        let n = trimmed[..colon_pos].trim_end();
        let after_colon = &trimmed[colon_pos + 1..];
        let t = after_colon.trim();
        let ws_after = after_colon.len() - after_colon.trim_start().len();
        let type_col = col + colon_pos + 1 + ws_after;
        (
            Some(cursor.make_line_range(cursor.line, col, n.len())),
            Some(cursor.make_line_range(cursor.line, col + colon_pos, 1)),
            if t.is_empty() {
                None
            } else {
                Some(cursor.make_line_range(cursor.line, type_col, t.len()))
            },
        )
    } else {
        // Unnamed: type only
        (None, None, Some(cursor.current_trimmed_range()))
    };

    returns.push(NumPyReturns {
        range: cursor.current_trimmed_range(),
        name,
        colon,
        return_type,
        description: None,
    });
}

/// Process one content line for a Raises section.
fn process_raises_line(
    cursor: &LineCursor,
    raises: &mut Vec<NumPyException>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = raises.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();

    let (exc_type, colon, first_desc) = if let Some(colon_pos) = find_entry_colon(trimmed) {
        let type_str = trimmed[..colon_pos].trim_end();
        let after_colon = &trimmed[colon_pos + 1..];
        let desc_str = after_colon.trim();
        let ws_after = after_colon.len() - after_colon.trim_start().len();
        let desc_col = col + colon_pos + 1 + ws_after;
        (
            cursor.make_line_range(cursor.line, col, type_str.len()),
            Some(cursor.make_line_range(cursor.line, col + colon_pos, 1)),
            if desc_str.is_empty() {
                None
            } else {
                Some(cursor.make_line_range(cursor.line, desc_col, desc_str.len()))
            },
        )
    } else {
        (cursor.current_trimmed_range(), None, None)
    };

    raises.push(NumPyException {
        range: cursor.current_trimmed_range(),
        r#type: exc_type,
        colon,
        description: first_desc,
    });
}

/// Process one content line for a Warns section.
fn process_warning_line(
    cursor: &LineCursor,
    warnings: &mut Vec<NumPyWarning>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = warnings.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();

    let (warn_type, colon, first_desc) = if let Some(colon_pos) = find_entry_colon(trimmed) {
        let type_str = trimmed[..colon_pos].trim_end();
        let after_colon = &trimmed[colon_pos + 1..];
        let desc_str = after_colon.trim();
        let ws_after = after_colon.len() - after_colon.trim_start().len();
        let desc_col = col + colon_pos + 1 + ws_after;
        (
            cursor.make_line_range(cursor.line, col, type_str.len()),
            Some(cursor.make_line_range(cursor.line, col + colon_pos, 1)),
            if desc_str.is_empty() {
                None
            } else {
                Some(cursor.make_line_range(cursor.line, desc_col, desc_str.len()))
            },
        )
    } else {
        (cursor.current_trimmed_range(), None, None)
    };

    warnings.push(NumPyWarning {
        range: cursor.current_trimmed_range(),
        r#type: warn_type,
        colon,
        description: first_desc,
    });
}

/// Process one content line for an Attributes section.
fn process_attribute_line(
    cursor: &LineCursor,
    attrs: &mut Vec<NumPyAttribute>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = attrs.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();
    let parts = parse_name_and_type(trimmed, cursor.line, col, cursor);

    let name = parts
        .names
        .into_iter()
        .next()
        .unwrap_or_else(TextRange::empty);

    attrs.push(NumPyAttribute {
        range: cursor.current_trimmed_range(),
        name,
        colon: parts.colon,
        r#type: parts.param_type,
        description: None,
    });
}

/// Process one content line for a Methods section.
fn process_method_line(
    cursor: &LineCursor,
    methods: &mut Vec<NumPyMethod>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = methods.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();

    let (name, colon) = if let Some(colon_pos) = find_entry_colon(trimmed) {
        let n = trimmed[..colon_pos].trim_end();
        (
            cursor.make_line_range(cursor.line, col, n.len()),
            Some(cursor.make_line_range(cursor.line, col + colon_pos, 1)),
        )
    } else {
        (cursor.current_trimmed_range(), None)
    };

    methods.push(NumPyMethod {
        range: cursor.current_trimmed_range(),
        name,
        colon,
        description: None,
    });
}

/// Process one content line for a See Also section.
fn process_see_also_line(
    cursor: &LineCursor,
    items: &mut Vec<SeeAlsoItem>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = items.last_mut() {
                extend_description(
                    &mut last.description,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();

    let (names_str, colon, description) = if let Some(colon_pos) = find_entry_colon(trimmed) {
        let after_colon = &trimmed[colon_pos + 1..];
        let desc_text = after_colon.trim();
        let ws_after = after_colon.len() - after_colon.trim_start().len();
        let desc_col = col + colon_pos + 1 + ws_after;
        (
            trimmed[..colon_pos].trim_end(),
            Some(cursor.make_line_range(cursor.line, col + colon_pos, 1)),
            if desc_text.is_empty() {
                None
            } else {
                Some(cursor.make_line_range(cursor.line, desc_col, desc_text.len()))
            },
        )
    } else {
        (trimmed, None, None)
    };

    let names = parse_name_list(names_str, cursor.line, col, cursor);

    items.push(SeeAlsoItem {
        range: cursor.make_line_range(cursor.line, col, trimmed.len()),
        names,
        colon,
        description,
    });
}

/// Process one content line for a References section.
///
/// Handles both RST citation references (`.. [N] content`) and plain text.
fn process_reference_line(
    cursor: &LineCursor,
    refs: &mut Vec<NumPyReference>,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            if let Some(last) = refs.last_mut() {
                extend_description(
                    &mut last.content,
                    &mut last.range,
                    cursor.current_trimmed_range(),
                );
            }
            return;
        }
    }
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }

    let col = cursor.current_indent();
    let trimmed = cursor.current_trimmed();
    let is_directive = trimmed.starts_with("..") && trimmed[2..].trim_start().starts_with('[');

    if is_directive {
        let rel_open = trimmed.find('[').unwrap();
        let abs_open = cursor.substr_offset(trimmed) + rel_open;
        if let Some(abs_close) = find_matching_close(cursor.source(), abs_open) {
            let directive_marker = Some(cursor.make_line_range(cursor.line, col, 2));
            let open_bracket = Some(TextRange::from_offset_len(abs_open, 1));
            let close_bracket = Some(TextRange::from_offset_len(abs_close, 1));
            let num_raw = &cursor.source()[abs_open + 1..abs_close];
            let num_str = num_raw.trim();
            let number = if !num_str.is_empty() {
                let num_abs = cursor.substr_offset(num_str);
                Some(TextRange::from_offset_len(num_abs, num_str.len()))
            } else {
                None
            };
            // Content after `]` on this line
            let line_end_offset =
                cursor.substr_offset(cursor.current_line_text()) + cursor.current_line_text().len();
            let after_on_line =
                &cursor.source()[abs_close + 1..line_end_offset.min(cursor.source().len())];
            let content_str = after_on_line.trim();
            let content = if !content_str.is_empty() {
                Some(TextRange::from_offset_len(
                    cursor.substr_offset(content_str),
                    content_str.len(),
                ))
            } else {
                None
            };

            refs.push(NumPyReference {
                range: cursor.current_trimmed_range(),
                directive_marker,
                open_bracket,
                number,
                close_bracket,
                content,
            });
            return;
        }
    }

    // Plain text reference / non-RST
    refs.push(NumPyReference {
        range: cursor.current_trimmed_range(),
        directive_marker: None,
        open_bracket: None,
        number: None,
        close_bracket: None,
        content: Some(cursor.current_trimmed_range()),
    });
}

/// Process one content line for a free-text section (Notes, Examples, etc.).
///
/// Only called for non-blank lines (blanks are skipped by the main loop).
/// Blank lines between content lines are implicitly included in the
/// resulting range because `extend` spans across them.
fn process_freetext_line(cursor: &LineCursor, content: &mut Option<TextRange>) {
    let range = cursor.current_trimmed_range();
    match content {
        Some(c) => c.extend(range),
        None => *content = Some(range),
    }
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

    /// Check whether `trimmed` looks like a parameter header line.
    /// A parameter header contains a colon (not inside brackets).
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
