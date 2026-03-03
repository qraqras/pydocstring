//! Google style docstring parser.
//!
//! Parses docstrings in Google format:
//! ```text
//! Brief summary.
//!
//! Extended description.
//!
//! Args:
//!     param1 (type): Description of param1.
//!     param2 (type, optional): Description of param2.
//!
//! Returns:
//!     type: Description of return value.
//!
//! Raises:
//!     ValueError: If the input is invalid.
//! ```

use crate::ast::TextRange;
use crate::cursor::{LineCursor, indent_len};
use crate::styles::google::ast::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleDocstringItem, GoogleException,
    GoogleMethod, GoogleReturns, GoogleSection, GoogleSectionBody, GoogleSectionHeader,
    GoogleSectionKind, GoogleSeeAlsoItem, GoogleWarning,
};
use crate::styles::utils::{find_entry_colon, split_comma_parts};

// =============================================================================
// Section detection
// =============================================================================

/// Extract the section name from a trimmed header line.
///
/// Strips the trailing colon (and any whitespace before it) if present.
/// Returns `(name, has_colon)` where `name` is the clean section name.
fn extract_section_name(trimmed: &str) -> (&str, bool) {
    if let Some(stripped) = trimmed.strip_suffix(':') {
        (stripped.trim_end(), true)
    } else {
        (trimmed, false)
    }
}

// =============================================================================
// Optional helpers
// =============================================================================

/// Strip a trailing `optional` marker from a type annotation.
///
/// Uses bracket-aware comma splitting so that commas inside type
/// annotations like `Dict[str, int]` are never mistaken for the
/// separator before `optional`.
///
/// Returns `(clean_type, optional_byte_offset)` where the offset is
/// relative to the start of `type_content` and points to the `o` in
/// `optional`.
fn strip_optional(type_content: &str) -> (&str, Option<usize>) {
    let parts = split_comma_parts(type_content);
    let mut optional_offset = None;
    let mut type_end = 0;

    for &(seg_offset, seg_raw) in &parts {
        let seg = seg_raw.trim();
        if seg == "optional" {
            let ws_lead = seg_raw.len() - seg_raw.trim_start().len();
            optional_offset = Some(seg_offset + ws_lead);
        } else if !seg.is_empty() {
            type_end = seg_offset + seg_raw.trim_end().len();
        }
    }

    if let Some(opt) = optional_offset {
        let clean = type_content[..type_end].trim_end_matches(',').trim_end();
        (clean, Some(opt))
    } else {
        (type_content, None)
    }
}

// =============================================================================
// Entry header parsing
// =============================================================================

/// Type information from a parsed entry header.
///
/// All fields are `TextRange` (byte-offset spans in the source).
struct TypeInfo {
    /// Opening bracket (`(`, `[`, `{`, or `<`).
    open_bracket: TextRange,
    /// Type annotation (without `optional` marker). `None` when brackets are empty.
    r#type: Option<TextRange>,
    /// Closing bracket (`)`, `]`, `}`, or `>`).
    close_bracket: TextRange,
    /// The `optional` marker, if present.
    optional: Option<TextRange>,
}

/// Parsed components of a Google-style entry header.
///
/// All span fields are `TextRange` (byte-offset spans in the source).
struct EntryHeader {
    /// Span of the entire header (from name start to the end of the last
    /// token on the header line — description fragment, colon, or bracket).
    range: TextRange,
    /// Entry name (parameter name, exception type, etc.).
    name: TextRange,
    /// Type annotation info (includes brackets, type, and optional marker).
    type_info: Option<TypeInfo>,
    /// The entry-separating colon (`:`) span, if present.
    colon: Option<TextRange>,
    /// First-line description fragment (may be empty).
    first_description: TextRange,
}

/// Find the matching close bracket within a single string.
///
/// `open_pos` is the byte index of the opening bracket in `s`.
/// Returns `Some(close_pos)` on success, `None` if unmatched.
fn find_matching_close_in_str(s: &str, open_pos: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let open = bytes[open_pos];
    let close = match open {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        _ => return None,
    };
    let mut depth: u32 = 1;
    for (i, &b) in bytes[open_pos + 1..].iter().enumerate() {
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(open_pos + 1 + i);
            }
        }
    }
    None
}

/// Parse a Google-style entry header at `cursor.line`.
///
/// Recognised patterns:
/// - `name (type, optional): description`
/// - `name (type): description`
/// - `name: description`
/// - `*args: description`
/// - `**kwargs (dict): description`
///
/// Bracket matching is line-local: the type annotation (including its
/// closing bracket) must appear on the same line as the opening bracket.
///
/// Does **not** advance the cursor.
fn parse_entry_header(cursor: &LineCursor) -> EntryHeader {
    let line = cursor.current_line_text();
    let trimmed = line.trim();
    let entry_start = cursor.substr_offset(trimmed);

    // --- Pattern 1: `name (type): desc` / `name [type]: desc` / `name {type}: desc` / `name <type>: desc` ---
    // Find the first opening bracket (`(`, `[`, `{`, or `<`) preceded by whitespace.
    let bracket_pos = trimmed.bytes().enumerate().find_map(|(i, b)| {
        if (b == b'(' || b == b'[' || b == b'{' || b == b'<')
            && i > 0
            && trimmed.as_bytes()[i - 1].is_ascii_whitespace()
        {
            Some(i)
        } else {
            None
        }
    });

    if let Some(rel_paren) = bracket_pos {
        // Line-local bracket matching
        if let Some(rel_close) = find_matching_close_in_str(trimmed, rel_paren) {
            let abs_paren = entry_start + rel_paren;
            let abs_close = entry_start + rel_close;

            let name = trimmed[..rel_paren].trim_end();
            let name_span = TextRange::from_offset_len(entry_start, name.len());
            let open_bracket = TextRange::from_offset_len(abs_paren, 1);
            let close_bracket = TextRange::from_offset_len(abs_close, 1);

            // Type content between the brackets (single line)
            let type_raw = &trimmed[rel_paren + 1..rel_close];
            let type_trimmed = type_raw.trim();
            let leading_ws = type_raw.len() - type_raw.trim_start().len();
            let type_start = abs_paren + 1 + leading_ws;

            // Strip optional marker
            let (clean_type, opt_rel) = strip_optional(type_trimmed);
            let opt_span =
                opt_rel.map(|r| TextRange::from_offset_len(type_start + r, "optional".len()));

            let type_span = if !clean_type.is_empty() {
                Some(TextRange::from_offset_len(type_start, clean_type.len()))
            } else {
                None
            };

            let type_info = Some(TypeInfo {
                open_bracket,
                r#type: type_span,
                close_bracket,
                optional: opt_span,
            });

            // Description after closing bracket (same line)
            let after_close = &trimmed[rel_close + 1..];
            let (first_description, colon) = extract_desc_after_colon(after_close, abs_close + 1);

            let range_end = if !first_description.is_empty() {
                first_description.end()
            } else if let Some(ref c) = colon {
                c.end()
            } else {
                close_bracket.end()
            };

            return EntryHeader {
                range: TextRange::new(name_span.start(), range_end),
                name: name_span,
                type_info,
                colon,
                first_description,
            };
        }
    }

    // --- Pattern 2: `name: desc` / `name:desc` / `name:` (no type) ---
    if let Some(colon_rel) = find_entry_colon(trimmed) {
        let name = trimmed[..colon_rel].trim_end();
        let after_colon = &trimmed[colon_rel + 1..];
        let desc = after_colon.trim_start();
        let ws_after = after_colon.len() - desc.len();
        let desc_start = entry_start + colon_rel + 1 + ws_after;
        let colon_span = TextRange::from_offset_len(entry_start + colon_rel, 1);
        let first_description = if desc.is_empty() {
            TextRange::empty()
        } else {
            TextRange::from_offset_len(desc_start, desc.len())
        };
        let range_end = if !first_description.is_empty() {
            first_description.end()
        } else {
            colon_span.end()
        };
        let name_span = TextRange::from_offset_len(entry_start, name.len());
        return EntryHeader {
            range: TextRange::new(name_span.start(), range_end),
            name: name_span,
            type_info: None,
            colon: Some(colon_span),
            first_description,
        };
    }

    // --- Fallback: bare name or plain text ---
    let name_span = TextRange::from_offset_len(entry_start, trimmed.len());
    EntryHeader {
        range: name_span,
        name: name_span,
        type_info: None,
        colon: None,
        first_description: TextRange::empty(),
    }
}

/// Extract description and colon spans after the closing bracket.
///
/// `after_paren` is the portion of text after `)`, and `base_offset` is its
/// absolute byte offset within the source.
///
/// Returns `(description_range, colon_range)`.
fn extract_desc_after_colon(
    after_paren: &str,
    base_offset: usize,
) -> (TextRange, Option<TextRange>) {
    let stripped = after_paren.trim_start();
    if let Some(after_colon) = stripped.strip_prefix(':') {
        let desc = after_colon.trim_start();
        let leading_to_stripped = after_paren.len() - stripped.len();
        let leading_after_colon = after_colon.len() - desc.len();
        let colon_abs = base_offset + leading_to_stripped;
        let desc_start = colon_abs + 1 + leading_after_colon;
        let desc_range = if desc.is_empty() {
            TextRange::empty()
        } else {
            TextRange::from_offset_len(desc_start, desc.len())
        };
        (desc_range, Some(TextRange::from_offset_len(colon_abs, 1)))
    } else {
        (TextRange::empty(), None)
    }
}

// =============================================================================
// Section header parsing
// =============================================================================

/// Try to parse a Google-style section header at `cursor.line`.
///
/// A section header is a line that matches one of:
/// - `Word:` / `Two Words:` — standard form with colon
/// - `Word :` — colon preceded by whitespace
/// - `Word` — colonless form, only for known section names
///
/// For the colon forms, any name that starts with an ASCII letter and
/// contains only alphanumeric characters / whitespace (no embedded
/// colons) is accepted (dispatched as `Unknown` if unrecognised).
/// For the colonless form, only names in [`GoogleSectionKind`] are
/// accepted to avoid treating ordinary text lines as headers.
///
/// Indentation is intentionally **not** checked here so that the parser
/// remains tolerant of irregular formatting.  Indent-level validation is
/// left to a downstream lint pass that can inspect the parsed AST.
///
/// Returns `Some(header)` if the current line is a valid section header,
/// `None` otherwise.  Does **not** advance the cursor.
fn try_parse_section_header(cursor: &LineCursor) -> Option<GoogleSectionHeader> {
    let trimmed = cursor.current_trimmed();
    let (name, has_colon) = extract_section_name(trimmed);

    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }

    let is_header = if has_colon {
        // Standard / space-before-colon form: accept any name without
        // embedded colons or entry-like characters (brackets, asterisks).
        !name.contains(':')
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c.is_ascii_whitespace())
    } else {
        // Colonless form: only known names.
        GoogleSectionKind::is_known(&name.to_ascii_lowercase())
    };

    if !is_header {
        return None;
    }

    let col = cursor.current_indent();
    let header_name = name.trim_end();

    let colon = if has_colon {
        let colon_col = col + trimmed.len() - 1;
        Some(cursor.make_line_range(cursor.line, colon_col, 1))
    } else {
        None
    };

    let normalized = header_name.to_ascii_lowercase();
    let kind = GoogleSectionKind::from_name(&normalized);

    Some(GoogleSectionHeader {
        range: cursor.current_trimmed_range(),
        kind,
        name: cursor.make_line_range(cursor.line, col, header_name.len()),
        colon,
    })
}

// =============================================================================
// Section body helpers
// =============================================================================

/// Push a new entry into `body` from the parsed header at the current line.
fn push_entry(cursor: &LineCursor, body: &mut GoogleSectionBody) {
    let header = parse_entry_header(cursor);
    let entry_col = cursor.current_indent();
    let range_end = if !header.first_description.is_empty() {
        header.first_description.end()
    } else {
        header.range.end()
    };
    let (end_line, end_col_pos) = cursor.offset_to_line_col(range_end.raw() as usize);
    let entry_range = cursor.make_range(cursor.line, entry_col, end_line, end_col_pos);

    let (r#type, optional, open_bracket, close_bracket) = match &header.type_info {
        Some(ti) => (
            ti.r#type,
            ti.optional,
            Some(ti.open_bracket),
            Some(ti.close_bracket),
        ),
        None => (None, None, None, None),
    };

    match body {
        GoogleSectionBody::Args(v)
        | GoogleSectionBody::KeywordArgs(v)
        | GoogleSectionBody::OtherParameters(v)
        | GoogleSectionBody::Receives(v) => {
            v.push(GoogleArg {
                range: entry_range,
                name: header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: header.colon,
                description: header.first_description,
                optional,
            });
        }
        GoogleSectionBody::Raises(v) => {
            v.push(GoogleException {
                range: entry_range,
                r#type: header.name,
                colon: header.colon,
                description: header.first_description,
            });
        }
        GoogleSectionBody::Warns(v) => {
            v.push(GoogleWarning {
                range: entry_range,
                warning_type: header.name,
                colon: header.colon,
                description: header.first_description,
            });
        }
        GoogleSectionBody::Attributes(v) => {
            v.push(GoogleAttribute {
                range: entry_range,
                name: header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: header.colon,
                description: header.first_description,
            });
        }
        GoogleSectionBody::Methods(v) => {
            v.push(GoogleMethod {
                range: entry_range,
                name: header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: header.colon,
                description: header.first_description,
            });
        }
        GoogleSectionBody::SeeAlso(v) => {
            // Split the name span by comma into individual name spans.
            let name_text = header.name.source_text(cursor.source());
            let base = header.name.start().raw() as usize;
            let mut names = Vec::new();
            let mut offset = 0;
            for part in name_text.split(',') {
                let name = part.trim();
                if !name.is_empty() {
                    let lead = part.len() - part.trim_start().len();
                    names.push(TextRange::from_offset_len(base + offset + lead, name.len()));
                }
                offset += part.len() + 1; // +1 for the comma
            }
            v.push(GoogleSeeAlsoItem {
                range: entry_range,
                names,
                colon: header.colon,
                description: if header.first_description.is_empty() {
                    None
                } else {
                    Some(header.first_description)
                },
            });
        }
        _ => unreachable!(),
    }
}

/// Build a [`TextRange`] spanning from the first to the last content line.
fn build_content_range(cursor: &LineCursor, first: Option<usize>, last: usize) -> TextRange {
    if let Some(f) = first {
        let first_line = cursor.line_text(f);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last);
        let last_col = indent_len(last_line) + last_line.trim().len();
        cursor.make_range(f, first_col, last, last_col)
    } else {
        TextRange::empty()
    }
}

// =============================================================================
// Per-line section body processors
// =============================================================================

/// Process one content line for an entry-based section (Args, Raises, etc.).
///
/// Blank lines must be filtered by the caller before invoking this function.
fn process_entry_line(
    cursor: &LineCursor,
    body: &mut GoogleSectionBody,
    entry_indent: &mut Option<usize>,
) {
    let indent_cols = cursor.current_indent_columns();

    // Continuation line for the current entry?
    if let Some(base) = *entry_indent {
        if indent_cols > base {
            body.extend_last_description(cursor.current_trimmed_range());
            return;
        }
    }

    // New entry (or first entry): push immediately
    if entry_indent.is_none() {
        *entry_indent = Some(indent_cols);
    }
    push_entry(cursor, body);
}

/// Process one content line for a Returns / Yields section.
///
/// The first non-blank content line is parsed as `type: description`;
/// subsequent lines extend the description range.
///
/// Blank lines must be filtered by the caller before invoking this function.
fn process_returns_line(cursor: &LineCursor, ret: &mut GoogleReturns) {
    let trimmed_range = cursor.current_trimmed_range();
    if ret.range.is_empty() {
        // First content line — parse type and description
        ret.range = trimmed_range;
        let trimmed = cursor.current_trimmed();
        let col = cursor.current_indent();
        if let Some(colon_pos) = find_entry_colon(trimmed) {
            let type_str = trimmed[..colon_pos].trim_end();
            let after_colon = &trimmed[colon_pos + 1..];
            let desc_str = after_colon.trim_start();
            let ws_after = after_colon.len() - desc_str.len();
            ret.return_type = Some(cursor.make_line_range(cursor.line, col, type_str.len()));
            ret.colon = Some(cursor.make_line_range(cursor.line, col + colon_pos, 1));
            let desc_start = col + colon_pos + 1 + ws_after;
            ret.description = if desc_str.is_empty() {
                TextRange::empty()
            } else {
                cursor.make_line_range(cursor.line, desc_start, desc_str.len())
            };
        } else {
            ret.description = trimmed_range;
        }
    } else {
        // Continuation line — extend description and range
        ret.description.extend(trimmed_range);
        ret.range = TextRange::new(ret.range.start(), trimmed_range.end());
    }
}

/// Process one content line for a free-text section (Notes, Examples, etc.).
///
/// Blank lines must be filtered by the caller before invoking this function.
fn process_freetext_line(cursor: &LineCursor, content: &mut TextRange) {
    content.extend(cursor.current_trimmed_range());
}

/// Flush a completed section into the docstring.
fn flush_section(
    cursor: &LineCursor,
    docstring: &mut GoogleDocstring,
    header: GoogleSectionHeader,
    body: GoogleSectionBody,
) {
    let header_start = header.range.start().raw() as usize;
    let range = cursor.span_back_from_cursor(header_start);
    docstring
        .items
        .push(GoogleDocstringItem::Section(GoogleSection {
            range,
            header,
            body,
        }));
}

// =============================================================================
// Main parser
// =============================================================================

/// Parse a Google-style docstring.
///
/// # Example
///
/// ```rust
/// use pydocstring::google::parse_google;
/// use pydocstring::GoogleSectionBody;
///
/// let input = "Summary.\n\nArgs:\n    x (int): The value.\n\nReturns:\n    int: The result.";
/// let doc = &parse_google(input);
///
/// assert_eq!(doc.summary.as_ref().unwrap().source_text(&doc.source), "Summary.");
///
/// let args: Vec<_> = doc.items.iter().filter_map(|item| match item {
///     pydocstring::GoogleDocstringItem::Section(s) => match &s.body {
///         GoogleSectionBody::Args(v) => Some(v.iter()),
///         _ => None,
///     },
///     _ => None,
/// }).flatten().collect();
/// assert_eq!(args.len(), 1);
/// assert_eq!(args[0].name.source_text(&doc.source), "x");
///
/// let ret = doc.items.iter().find_map(|item| match item {
///     pydocstring::GoogleDocstringItem::Section(s) => match &s.body {
///         GoogleSectionBody::Returns(r) => Some(r),
///         _ => None,
///     },
///     _ => None,
/// }).unwrap();
/// assert_eq!(ret.return_type.as_ref().unwrap().source_text(&doc.source), "int");
/// ```
pub fn parse_google(input: &str) -> GoogleDocstring {
    let mut line_cursor = LineCursor::new(input);
    let mut docstring = GoogleDocstring::new(input);

    line_cursor.skip_blanks();
    if line_cursor.is_eof() {
        return docstring;
    }

    // Phase tracking for pre-section content.
    //   summary_done  – true once a blank line or header terminates summary.
    //   extended_done  – true once a header terminates extended summary.
    let mut summary_done = false;
    let mut extended_done = false;
    let mut summary_first: Option<usize> = None;
    let mut summary_last: usize = 0;
    let mut ext_first: Option<usize> = None;
    let mut ext_last: usize = 0;

    // Current section being parsed.
    let mut current_header: Option<GoogleSectionHeader> = None;
    let mut current_body: Option<GoogleSectionBody> = None;
    let mut entry_indent: Option<usize> = None;

    while !line_cursor.is_eof() {
        // --- Blank lines ---
        if line_cursor.current_trimmed().is_empty() {
            // Blank line after summary content → finalise summary
            if !summary_done && summary_first.is_some() {
                docstring.summary = Some(build_content_range(
                    &line_cursor,
                    summary_first,
                    summary_last,
                ));
                summary_done = true;
            }
            line_cursor.advance();
            continue;
        }

        // --- Detect section header ---
        if let Some(header) = try_parse_section_header(&line_cursor) {
            // Finalise any pending pre-section content
            if !summary_done {
                if summary_first.is_some() {
                    docstring.summary = Some(build_content_range(
                        &line_cursor,
                        summary_first,
                        summary_last,
                    ));
                }
                summary_done = true;
            }
            if !extended_done {
                if ext_first.is_some() {
                    docstring.extended_summary =
                        Some(build_content_range(&line_cursor, ext_first, ext_last));
                }
                extended_done = true;
            }

            // Flush previous section
            if let Some(prev_header) = current_header.take() {
                flush_section(
                    &line_cursor,
                    &mut docstring,
                    prev_header,
                    current_body.take().unwrap(),
                );
            }

            // Start new section
            current_body = Some(GoogleSectionBody::new(header.kind));
            current_header = Some(header);
            entry_indent = None;
            line_cursor.advance(); // skip header line
            continue;
        }

        // --- Process line based on current state ---
        if let Some(ref mut body) = current_body {
            #[rustfmt::skip]
            match body {
                GoogleSectionBody::Args(_)                          => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::KeywordArgs(_)                   => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::OtherParameters(_)               => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Receives(_)                      => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Raises(_)                        => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Warns(_)                         => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Attributes(_)                    => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Methods(_)                       => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::SeeAlso(_)                       => process_entry_line(&line_cursor, body, &mut entry_indent),
                GoogleSectionBody::Returns(ret) => process_returns_line(&line_cursor, ret),
                GoogleSectionBody::Yields(ret)  => process_returns_line(&line_cursor, ret),
                GoogleSectionBody::Notes(r)         => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Examples(r)      => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Todo(r)          => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::References(r)    => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Warnings(r)      => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Attention(r)     => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Caution(r)       => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Danger(r)        => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Error(r)         => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Hint(r)          => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Important(r)     => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Tip(r)           => process_freetext_line(&line_cursor, r),
                GoogleSectionBody::Unknown(r)       => process_freetext_line(&line_cursor, r),
            };
        } else if !summary_done {
            // Summary content line
            if summary_first.is_none() {
                summary_first = Some(line_cursor.line);
            }
            summary_last = line_cursor.line;
        } else if !extended_done {
            // Extended summary content line
            if ext_first.is_none() {
                ext_first = Some(line_cursor.line);
            }
            ext_last = line_cursor.line;
        } else {
            // Stray line (outside any section)
            docstring.items.push(GoogleDocstringItem::StrayLine(
                line_cursor.current_trimmed_range(),
            ));
        }

        line_cursor.advance();
    }

    // Flush final section
    if let Some(header) = current_header.take() {
        flush_section(
            &line_cursor,
            &mut docstring,
            header,
            current_body.take().unwrap(),
        );
    }

    // Finalise at EOF
    if !summary_done
        && summary_first.is_some() {
            docstring.summary = Some(build_content_range(
                &line_cursor,
                summary_first,
                summary_last,
            ));
        }
    if !extended_done
        && ext_first.is_some() {
            docstring.extended_summary =
                Some(build_content_range(&line_cursor, ext_first, ext_last));
        }

    // --- Docstring span ---
    docstring.range = line_cursor.full_range();

    docstring
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- section detection --

    /// Helper: returns true if the given line is detected as a section header.
    fn is_header(text: &str) -> bool {
        let cursor = LineCursor::new(text);
        try_parse_section_header(&cursor).is_some()
    }

    #[test]
    fn test_is_section_header() {
        // Standard colon form (expects pre-trimmed input)
        assert!(is_header("Args:"));
        // "NotASection:" is still detected as an unknown section header (has colon)
        assert!(is_header("NotASection:"));
        assert!(is_header("Returns:"));
        assert!(is_header("Custom:"));
        // Case-insensitive
        assert!(is_header("args:"));
        assert!(is_header("RETURNS:"));
        // Not a section header: contains embedded colon
        assert!(!is_header("key: value:"));
        // Long names with colon are still accepted — length validation
        // is left to a downstream lint pass.
        assert!(is_header(
            "This is a very long line that should not be a section header:"
        ));

        // Space-before-colon form
        assert!(is_header("Args :"));
        assert!(is_header("Returns :"));

        // Colonless form — only known section names
        assert!(is_header("Args"));
        assert!(is_header("Returns"));
        assert!(is_header("args"));
        assert!(is_header("RETURNS"));
        assert!(is_header("See Also"));
        // Unknown names without colon are NOT headers
        assert!(!is_header("NotASection"));
        assert!(!is_header("SomeWord"));
    }

    // -- entry header parsing --

    /// Helper to parse an entry header from a single-line string.
    fn header_from(text: &str) -> EntryHeader {
        let cursor = LineCursor::new(text);
        parse_entry_header(&cursor)
    }

    #[test]
    fn test_parse_entry_header_with_type() {
        let src = "name (int): Description";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "name");
        assert!(header.type_info.is_some());
        let ti = header.type_info.unwrap();
        assert_eq!(ti.r#type.unwrap().source_text(src), "int");
        assert_eq!(header.first_description.source_text(src), "Description");
    }

    #[test]
    fn test_parse_entry_header_optional() {
        let src = "name (int, optional): Description";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "name");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.r#type.unwrap().source_text(src), "int");
        assert!(ti.optional.is_some());
        assert_eq!(ti.optional.unwrap().source_text(src), "optional");
    }

    #[test]
    fn test_parse_entry_header_no_type() {
        let src = "name: Description";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "name");
        assert!(header.type_info.is_none());
        assert_eq!(header.first_description.source_text(src), "Description");
    }

    #[test]
    fn test_parse_entry_header_complex_type() {
        let src = "data (Dict[str, List[int]]): Values";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "data");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.r#type.unwrap().source_text(src), "Dict[str, List[int]]");
        assert_eq!(header.first_description.source_text(src), "Values");
    }

    #[test]
    fn test_parse_entry_header_colon_only() {
        let src = "x:";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "x");
        assert!(header.type_info.is_none());
        assert!(header.first_description.is_empty());
    }

    #[test]
    fn test_parse_entry_header_varargs() {
        let src1 = "*args: Positional arguments";
        let header = header_from(src1);
        assert_eq!(header.name.source_text(src1), "*args");
        assert_eq!(
            header.first_description.source_text(src1),
            "Positional arguments"
        );

        let src2 = "**kwargs (dict): Keyword arguments";
        let header = header_from(src2);
        assert_eq!(header.name.source_text(src2), "**kwargs");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.r#type.unwrap().source_text(src2), "dict");
    }

    #[test]
    fn test_parse_entry_header_no_space_after_colon() {
        let src = "name:Description";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "name");
        assert!(header.type_info.is_none());
        assert_eq!(header.first_description.source_text(src), "Description");
    }

    #[test]
    fn test_parse_entry_header_extra_spaces_after_colon() {
        let src = "name:   Description";
        let header = header_from(src);
        assert_eq!(header.name.source_text(src), "name");
        assert!(header.type_info.is_none());
        assert_eq!(header.first_description.source_text(src), "Description");
    }

    // -- strip_optional --

    #[test]
    fn test_strip_optional_basic() {
        assert_eq!(strip_optional("int, optional"), ("int", Some(5)));
        assert_eq!(strip_optional("int"), ("int", None));
        assert_eq!(
            strip_optional("Dict[str, int], optional"),
            ("Dict[str, int]", Some(16))
        );
        assert_eq!(strip_optional("optional"), ("", Some(0)));
        // Varying whitespace after comma
        assert_eq!(strip_optional("int,optional"), ("int", Some(4)));
        assert_eq!(strip_optional("int,  optional"), ("int", Some(6)));
        assert_eq!(strip_optional("int, optional  "), ("int", Some(5)));
    }
}
