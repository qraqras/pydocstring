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
use crate::cursor::{LineCursor, indent_columns, indent_len};
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

/// Check if a line is a Google-style section header.
///
/// A section header is a line that matches one of:
/// - `Word:` / `Two Words:` — standard form with colon
/// - `Word :` — colon preceded by whitespace
/// - `Word` — colonless form, only for known section names
///
/// For the colon forms, any short name (≤ 40 chars, starts with alpha, no
/// embedded colons) is accepted (dispatched as Unknown if unrecognised).
/// For the colonless form, only names in [`KNOWN_SECTIONS`] are accepted
/// to avoid treating ordinary text lines as headers.
///
/// The caller must pass a line with leading / trailing whitespace
/// already stripped.  Indentation is intentionally **not** checked
/// here so that the parser remains tolerant of irregular formatting.
/// Indent-level validation is left to a downstream lint pass that can
/// inspect the parsed AST.
fn is_section_header(trimmed: &str) -> bool {
    let (name, has_colon) = extract_section_name(trimmed);

    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return false;
    }

    if has_colon {
        // Standard / space-before-colon form: accept any short name without
        // embedded colons or entry-like characters (brackets, asterisks).
        name.len() <= 40
            && !name.contains(':')
            && name.chars().all(|c| c.is_alphanumeric() || c == ' ')
    } else {
        // Colonless form: only known names.
        GoogleSectionKind::is_known(&name.to_ascii_lowercase())
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
// Text range collector
// =============================================================================

/// Collect consecutive lines into a [`TextRange`], stopping at section headers
/// and EOF.
///
/// When `stop_at_blank` is `true`, also stops at blank lines (used for
/// single-paragraph collection such as Summary).  Leading and trailing
/// blank lines within the collected range are excluded from the span.
///
/// On return, `cursor.line` points to the first unconsumed line.
fn collect_text_range(cursor: &mut LineCursor, stop_at_blank: bool) -> TextRange {
    let mut first_content: Option<usize> = None;
    let mut last_content = cursor.line;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) || (stop_at_blank && trimmed.is_empty()) {
            break;
        }
        if !trimmed.is_empty() {
            if first_content.is_none() {
                first_content = Some(cursor.line);
            }
            last_content = cursor.line;
        }
        cursor.advance();
    }

    if let Some(first) = first_content {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content);
        let last_col = indent_len(last_line) + last_line.trim().len();
        cursor.make_range(first, first_col, last_content, last_col)
    } else {
        TextRange::empty()
    }
}

// =============================================================================
// Description collector
// =============================================================================

/// Collect indented description continuation lines starting at `cursor.line`.
///
/// Stops at:
/// - Section headers (detected by pattern)
/// - Non-empty lines at or below `entry_indent` (i.e. a new entry)
/// - End of input
///
/// On return, `cursor.line` points to the first unconsumed line.
fn collect_description(cursor: &mut LineCursor, entry_indent: usize) -> TextRange {
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = cursor.line;

    while !cursor.is_eof() {
        let line = cursor.current_line_text();
        let trimmed = line.trim();

        if is_section_header(trimmed) {
            break;
        }

        // Non-empty line at or below entry indent ⇒ new entry
        if !trimmed.is_empty() && indent_columns(line) <= entry_indent {
            break;
        }

        desc_parts.push(trimmed);
        if !trimmed.is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(cursor.line);
            }
            last_content_line = cursor.line;
        }
        cursor.advance();
    }

    // Trim leading / trailing empty entries
    while desc_parts.last().is_some_and(|l| l.is_empty()) {
        desc_parts.pop();
    }
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

/// Merge a first-line description `TextRange` with a continuation `TextRange`.
///
/// Either or both may be empty. When both are non-empty, returns a range
/// spanning from the start of `first` to the end of `cont`.
fn merge_descriptions(first: TextRange, cont: TextRange) -> TextRange {
    if first.is_empty() {
        return cont;
    }
    if cont.is_empty() {
        return first;
    }
    TextRange::new(first.start(), cont.end())
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

/// Parse a Google-style entry header at `cursor.line`.
///
/// Recognised patterns:
/// - `name (type, optional): description`
/// - `name (type): description`
/// - `name: description`
/// - `*args: description`
/// - `**kwargs (dict): description`
///
/// Bracket matching works on the full source, so type annotations that span
/// multiple lines (e.g. `Dict[str,\n  int]`) are handled correctly.
///
/// Does **not** advance the cursor — the caller must derive the end line
/// from `header.range` and advance past it.
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
        let abs_paren = entry_start + rel_paren;

        // find_matching_close on full source — crosses line boundaries
        if let Some(abs_close) = cursor.find_matching_close(abs_paren) {
            let name = trimmed[..rel_paren].trim_end();
            let name_span = TextRange::from_offset_len(entry_start, name.len());
            let open_bracket = TextRange::from_offset_len(abs_paren, 1); // single-byte ASCII bracket
            let close_bracket = TextRange::from_offset_len(abs_close, 1); // single-byte ASCII bracket

            // Type content between the brackets (may span multiple lines)
            let type_raw = &cursor.source()[abs_paren + 1..abs_close];
            let type_trimmed = type_raw.trim();
            let type_start = if !type_trimmed.is_empty() {
                cursor.substr_offset(type_trimmed)
            } else {
                abs_paren + 1
            };

            // Strip optional marker
            let (clean_type, opt_rel) = strip_optional(type_trimmed);
            let opt_span =
                opt_rel.map(|r| TextRange::from_offset_len(type_start + r, "optional".len()));

            let type_span = if !clean_type.is_empty() {
                let ts = cursor.substr_offset(clean_type);
                Some(TextRange::from_offset_len(ts, clean_type.len()))
            } else {
                None
            };

            let type_info = Some(TypeInfo {
                open_bracket,
                r#type: type_span,
                close_bracket,
                optional: opt_span,
            });

            // Find which line the close bracket is on
            let close_line = cursor.offset_to_line_col(abs_close).0;

            // Description after closing bracket on the same line
            let close_line_str = cursor.line_text(close_line);
            let close_line_end = cursor.substr_offset(close_line_str) + close_line_str.len();
            let after_close = &cursor.source()[abs_close + 1..close_line_end];
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
/// Returns `Some(header)` if the current line is a valid section header,
/// `None` otherwise.  Does **not** advance the cursor.
fn try_parse_section_header(cursor: &LineCursor) -> Option<GoogleSectionHeader> {
    let trimmed = cursor.current_trimmed();
    if !is_section_header(trimmed) {
        return None;
    }

    let col = cursor.current_indent();

    let (raw_name, has_colon) = extract_section_name(trimmed);
    let header_name = raw_name.trim_end();

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

    // --- Summary ---
    if try_parse_section_header(&line_cursor).is_none() {
        let range = collect_text_range(&mut line_cursor, true);
        if !range.is_empty() {
            docstring.summary = Some(range);
        }
        line_cursor.skip_blanks();
        if line_cursor.is_eof() {
            docstring.range = line_cursor.full_range();
            return docstring;
        }
    }

    // --- Extended Summary ---
    if try_parse_section_header(&line_cursor).is_none() {
        let range = collect_text_range(&mut line_cursor, false);
        if !range.is_empty() {
            docstring.extended_summary = Some(range);
        }
        line_cursor.skip_blanks();
        if line_cursor.is_eof() {
            docstring.range = line_cursor.full_range();
            return docstring;
        }
    }

    // --- Sections ---
    while !line_cursor.is_eof() {
        line_cursor.skip_blanks();
        if line_cursor.is_eof() {
            break;
        }

        if let Some(header) = try_parse_section_header(&line_cursor) {
            line_cursor.advance(); // skip header line
            #[rustfmt::skip]
            let body = match header.kind {
                GoogleSectionKind::Args            => GoogleSectionBody::Args           (parse_args              (&mut line_cursor)),
                GoogleSectionKind::KeywordArgs     => GoogleSectionBody::KeywordArgs    (parse_args              (&mut line_cursor)),
                GoogleSectionKind::OtherParameters => GoogleSectionBody::OtherParameters(parse_args              (&mut line_cursor)),
                GoogleSectionKind::Receives        => GoogleSectionBody::Receives       (parse_args              (&mut line_cursor)),
                GoogleSectionKind::Returns         => GoogleSectionBody::Returns        (parse_returns_section   (&mut line_cursor)),
                GoogleSectionKind::Yields          => GoogleSectionBody::Yields         (parse_returns_section   (&mut line_cursor)),
                GoogleSectionKind::Raises          => GoogleSectionBody::Raises         (parse_raises_section    (&mut line_cursor)),
                GoogleSectionKind::Warns           => GoogleSectionBody::Warns          (parse_warns_section     (&mut line_cursor)),
                GoogleSectionKind::Attributes      => GoogleSectionBody::Attributes     (parse_attributes_section(&mut line_cursor)),
                GoogleSectionKind::Methods         => GoogleSectionBody::Methods        (parse_methods_section   (&mut line_cursor)),
                GoogleSectionKind::SeeAlso         => GoogleSectionBody::SeeAlso        (parse_see_also_section  (&mut line_cursor)),
                GoogleSectionKind::Notes           => GoogleSectionBody::Notes          (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Examples        => GoogleSectionBody::Examples       (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Todo            => GoogleSectionBody::Todo           (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::References      => GoogleSectionBody::References     (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Warnings        => GoogleSectionBody::Warnings       (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Attention       => GoogleSectionBody::Attention      (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Caution         => GoogleSectionBody::Caution        (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Danger          => GoogleSectionBody::Danger         (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Error           => GoogleSectionBody::Error          (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Hint            => GoogleSectionBody::Hint           (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Important       => GoogleSectionBody::Important      (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Tip             => GoogleSectionBody::Tip            (parse_section_content   (&mut line_cursor)),
                GoogleSectionKind::Unknown         => GoogleSectionBody::Unknown        (parse_section_content   (&mut line_cursor)),
            };

            let range = line_cursor.span_back_from_cursor(header.range.start().raw() as usize);

            docstring
                .items
                .push(GoogleDocstringItem::Section(GoogleSection {
                    range,
                    header,
                    body,
                }));
        } else {
            let spanned = line_cursor.current_trimmed_range();
            docstring
                .items
                .push(GoogleDocstringItem::StrayLine(spanned));
            line_cursor.advance();
        }
    }

    // --- Docstring span ---
    docstring.range = line_cursor.full_range();

    docstring
}

// =============================================================================
// Args parsing
// =============================================================================

/// Parse the Args / Arguments section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_args(cursor: &mut LineCursor) -> Vec<GoogleArg> {
    let mut args = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        // Entry line at entry indent level
        if indent_cols <= ei {
            let col = indent;
            let entry_start_line = cursor.line;

            let header = parse_entry_header(cursor);

            // Extract type info fields (already TextRange)
            let (arg_type, optional, open_bracket, close_bracket) = match &header.type_info {
                Some(ti) => (
                    ti.r#type,
                    ti.optional,
                    Some(ti.open_bracket),
                    Some(ti.close_bracket),
                ),
                None => (None, None, None, None),
            };

            // Description: first-line fragment + continuation lines
            let header_end_line = cursor
                .offset_to_line_col(header.range.end().raw() as usize)
                .0;
            cursor.line = header_end_line + 1;
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(header.first_description, cont_desc);

            let range_end = if full_desc.is_empty() {
                header.range.end()
            } else {
                full_desc.end()
            };
            let (end_line, end_col) = cursor.offset_to_line_col(range_end.raw() as usize);

            args.push(GoogleArg {
                range: cursor.make_range(entry_start_line, col, end_line, end_col),
                name: header.name,
                open_bracket,
                r#type: arg_type,
                close_bracket,
                colon: header.colon,
                description: full_desc,
                optional,
            });
        } else {
            cursor.advance();
        }
    }

    args
}

// =============================================================================
// Returns / Yields parsing
// =============================================================================

/// Parse the Returns / Yields section body as a single entry.
///
/// Only the first content line is checked for a `type: description` pattern.
/// All subsequent lines in the section are treated as continuation of the
/// description, regardless of indentation level (as long as they remain
/// within the section).
///
/// Supports both typed and untyped entries:
/// ```text
/// int: The result.          # typed
/// The result description.   # untyped
/// ```
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_returns_section(cursor: &mut LineCursor) -> GoogleReturns {
    // Skip leading blank lines within the section
    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            return GoogleReturns {
                range: TextRange::empty(),
                return_type: None,
                colon: None,
                description: TextRange::empty(),
            };
        }
        if !trimmed.is_empty() {
            break;
        }
        cursor.advance();
    }

    if cursor.is_eof() {
        return GoogleReturns {
            range: TextRange::empty(),
            return_type: None,
            colon: None,
            description: TextRange::empty(),
        };
    }

    let trimmed = cursor.current_trimmed();
    let col = cursor.current_indent();
    let entry_start = cursor.line;

    // Try `type: description` / `type:description` / `type:` pattern
    // only on the first content line.
    let (return_type, colon, first_desc_range) = if let Some(colon_pos) = find_entry_colon(trimmed)
    {
        let type_str = trimmed[..colon_pos].trim_end();
        let after_colon = &trimmed[colon_pos + 1..];
        let desc_str = after_colon.trim_start();
        let ws_after = after_colon.len() - desc_str.len();
        let type_col = col;
        let rt = Some(cursor.make_line_range(cursor.line, type_col, type_str.len()));
        let colon_spanned = Some(cursor.make_line_range(cursor.line, col + colon_pos, 1));
        let desc_start = col + colon_pos + 1 + ws_after;
        let desc_range = if desc_str.is_empty() {
            TextRange::empty()
        } else {
            cursor.make_line_range(cursor.line, desc_start, desc_str.len())
        };
        (rt, colon_spanned, desc_range)
    } else {
        // No type — just description
        let desc_range = if trimmed.is_empty() {
            TextRange::empty()
        } else {
            cursor.current_trimmed_range()
        };
        (None, None, desc_range)
    };

    cursor.advance();

    // Collect all remaining indented lines as continuation description.
    let cont_desc = parse_section_content(cursor);
    let full_desc = merge_descriptions(first_desc_range, cont_desc);

    let (end_line, end_col) = if full_desc.is_empty() {
        (entry_start, col + trimmed.len())
    } else {
        cursor.offset_to_line_col(full_desc.end().raw() as usize)
    };

    GoogleReturns {
        range: cursor.make_range(entry_start, col, end_line, end_col),
        return_type,
        colon,
        description: full_desc,
    }
}

// =============================================================================
// Raises parsing
// =============================================================================

/// Parse the Raises section body.
///
/// Format: `ExceptionType: description`
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_raises_section(cursor: &mut LineCursor) -> Vec<GoogleException> {
    let mut raises = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        if indent_cols <= ei {
            let col = indent;
            let entry_start = cursor.line;

            let (exc_type_str, first_desc_range, colon_offset) =
                if let Some(colon_pos) = find_entry_colon(trimmed) {
                    let et = trimmed[..colon_pos].trim_end();
                    let after_colon = &trimmed[colon_pos + 1..];
                    let desc = after_colon.trim_start();
                    let ws_after = after_colon.len() - desc.len();
                    let desc_start = col + colon_pos + 1 + ws_after;
                    let dr = if desc.is_empty() {
                        TextRange::empty()
                    } else {
                        cursor.make_line_range(cursor.line, desc_start, desc.len())
                    };
                    (et, dr, Some(col + colon_pos))
                } else {
                    (trimmed, TextRange::empty(), None)
                };

            let exc_type = cursor.make_line_range(cursor.line, col, exc_type_str.len());

            cursor.advance();
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(first_desc_range, cont_desc);

            let (end_line, end_col) = if full_desc.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(full_desc.end().raw() as usize)
            };

            let colon =
                colon_offset.map(|colon_col| cursor.make_line_range(entry_start, colon_col, 1));

            raises.push(GoogleException {
                range: cursor.make_range(entry_start, col, end_line, end_col),
                r#type: exc_type,
                colon,
                description: full_desc,
            });
        } else {
            cursor.advance();
        }
    }

    raises
}

// =============================================================================
// Warns parsing
// =============================================================================

/// Parse the Warns section body.
///
/// Format: `WarningType: description`
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_warns_section(cursor: &mut LineCursor) -> Vec<GoogleWarning> {
    let mut warns = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        if indent_cols <= ei {
            let col = indent;
            let entry_start = cursor.line;

            let (warn_type_str, first_desc_range, colon_offset) =
                if let Some(colon_pos) = find_entry_colon(trimmed) {
                    let wt = trimmed[..colon_pos].trim_end();
                    let after_colon = &trimmed[colon_pos + 1..];
                    let desc = after_colon.trim_start();
                    let ws_after = after_colon.len() - desc.len();
                    let desc_start = col + colon_pos + 1 + ws_after;
                    let dr = if desc.is_empty() {
                        TextRange::empty()
                    } else {
                        cursor.make_line_range(cursor.line, desc_start, desc.len())
                    };
                    (wt, dr, Some(col + colon_pos))
                } else {
                    (trimmed, TextRange::empty(), None)
                };

            let warning_type = cursor.make_line_range(cursor.line, col, warn_type_str.len());

            cursor.advance();
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(first_desc_range, cont_desc);

            let (end_line, end_col) = if full_desc.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                cursor.offset_to_line_col(full_desc.end().raw() as usize)
            };

            let colon =
                colon_offset.map(|colon_col| cursor.make_line_range(entry_start, colon_col, 1));

            warns.push(GoogleWarning {
                range: cursor.make_range(entry_start, col, end_line, end_col),
                warning_type,
                colon,
                description: full_desc,
            });
        } else {
            cursor.advance();
        }
    }

    warns
}

// =============================================================================
// Attributes parsing
// =============================================================================

/// Parse the Attributes section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_attributes_section(cursor: &mut LineCursor) -> Vec<GoogleAttribute> {
    let mut attrs = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        if indent_cols <= ei {
            let col = indent;
            let entry_start_line = cursor.line;

            let header = parse_entry_header(cursor);

            let (attr_type, open_bracket, close_bracket) = match &header.type_info {
                Some(ti) => (ti.r#type, Some(ti.open_bracket), Some(ti.close_bracket)),
                None => (None, None, None),
            };

            let header_end_line = cursor
                .offset_to_line_col(header.range.end().raw() as usize)
                .0;
            cursor.line = header_end_line + 1;
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(header.first_description, cont_desc);

            let range_end = if full_desc.is_empty() {
                header.range.end()
            } else {
                full_desc.end()
            };
            let (end_line, end_col) = cursor.offset_to_line_col(range_end.raw() as usize);

            attrs.push(GoogleAttribute {
                range: cursor.make_range(entry_start_line, col, end_line, end_col),
                name: header.name,
                open_bracket,
                r#type: attr_type,
                close_bracket,
                colon: header.colon,
                description: full_desc,
            });
        } else {
            cursor.advance();
        }
    }

    attrs
}

// =============================================================================
// Methods parsing
// =============================================================================

/// Parse the Methods section body.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_methods_section(cursor: &mut LineCursor) -> Vec<GoogleMethod> {
    let mut methods = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        if indent_cols <= ei {
            let col = indent;
            let entry_start_line = cursor.line;

            let header = parse_entry_header(cursor);

            let (method_type, open_bracket, close_bracket) = match &header.type_info {
                Some(ti) => (ti.r#type, Some(ti.open_bracket), Some(ti.close_bracket)),
                None => (None, None, None),
            };

            let header_end_line = cursor
                .offset_to_line_col(header.range.end().raw() as usize)
                .0;
            cursor.line = header_end_line + 1;
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(header.first_description, cont_desc);

            let range_end = if full_desc.is_empty() {
                header.range.end()
            } else {
                full_desc.end()
            };
            let (end_line, end_col) = cursor.offset_to_line_col(range_end.raw() as usize);

            methods.push(GoogleMethod {
                range: cursor.make_range(entry_start_line, col, end_line, end_col),
                name: header.name,
                open_bracket,
                r#type: method_type,
                close_bracket,
                colon: header.colon,
                description: full_desc,
            });
        } else {
            cursor.advance();
        }
    }

    methods
}

// =============================================================================
// Free-text section parsing
// =============================================================================

/// Parse a free-text section body (Notes, Examples, References, Warnings, …).
///
/// Collects all indented lines until the next section header.
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_section_content(cursor: &mut LineCursor) -> TextRange {
    collect_text_range(cursor, false)
}

// =============================================================================
// See Also parsing
// =============================================================================

/// Parse the See Also section body.
///
/// Supports the following patterns (following Napoleon):
/// ```text
/// See Also:
///     func_a: Description of func_a.
///     func_b, func_c
///     :meth:`func_d`: Description.
/// ```
///
/// On return, `cursor.line` points to the first line after the section.
fn parse_see_also_section(cursor: &mut LineCursor) -> Vec<GoogleSeeAlsoItem> {
    let mut items = Vec::new();
    let mut entry_indent: Option<usize> = None;

    while !cursor.is_eof() {
        let trimmed = cursor.current_trimmed();
        if is_section_header(trimmed) {
            break;
        }

        if trimmed.is_empty() {
            cursor.advance();
            continue;
        }

        let indent = cursor.current_indent();
        let indent_cols = cursor.current_indent_columns();

        let ei = *entry_indent.get_or_insert(indent_cols);

        if indent_cols <= ei {
            let col = indent;
            let entry_start = cursor.line;

            // Split on first colon for description (tolerant of any whitespace)
            let (names_part, first_desc_range, colon_offset) =
                if let Some(colon_pos) = find_entry_colon(trimmed) {
                    let n = trimmed[..colon_pos].trim_end();
                    let after_colon = &trimmed[colon_pos + 1..];
                    let desc = after_colon.trim_start();
                    let ws_after = after_colon.len() - desc.len();
                    let desc_start = col + colon_pos + 1 + ws_after;
                    let dr = if desc.is_empty() {
                        TextRange::empty()
                    } else {
                        cursor.make_line_range(cursor.line, desc_start, desc.len())
                    };
                    (n, dr, Some(col + colon_pos))
                } else {
                    (trimmed, TextRange::empty(), None)
                };

            // Parse comma-separated names
            let mut names = Vec::new();
            let mut name_offset = col;
            for part in names_part.split(',') {
                let name = part.trim();
                if !name.is_empty() {
                    // Find the actual position of this name within the line
                    let name_start = name_offset + (part.len() - part.trim_start().len());
                    names.push(cursor.make_line_range(cursor.line, name_start, name.len()));
                }
                name_offset += part.len() + 1; // +1 for the comma
            }

            cursor.advance();
            let cont_desc = collect_description(cursor, ei);
            let full_desc = merge_descriptions(first_desc_range, cont_desc);

            let description = if full_desc.is_empty() {
                None
            } else {
                Some(full_desc)
            };

            let (end_line, end_col) = if let Some(ref d) = description {
                cursor.offset_to_line_col(d.end().raw() as usize)
            } else {
                (entry_start, col + trimmed.len())
            };

            let colon =
                colon_offset.map(|colon_col| cursor.make_line_range(entry_start, colon_col, 1));

            items.push(GoogleSeeAlsoItem {
                range: cursor.make_range(entry_start, col, end_line, end_col),
                names,
                colon,
                description,
            });
        } else {
            cursor.advance();
        }
    }

    items
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- section detection --

    #[test]
    fn test_is_section_header() {
        // Standard colon form (expects pre-trimmed input)
        assert!(is_section_header("Args:"));
        // "NotASection:" is still detected as an unknown section header (has colon)
        assert!(is_section_header("NotASection:"));
        assert!(is_section_header("Returns:"));
        assert!(is_section_header("Custom:"));
        // Case-insensitive
        assert!(is_section_header("args:"));
        assert!(is_section_header("RETURNS:"));
        // Not a section header: contains embedded colon
        assert!(!is_section_header("key: value:"));
        // Not a section header: too long
        assert!(!is_section_header(
            "This is a very long line that should not be a section header:"
        ));

        // Space-before-colon form
        assert!(is_section_header("Args :"));
        assert!(is_section_header("Returns :"));

        // Colonless form — only known section names
        assert!(is_section_header("Args"));
        assert!(is_section_header("Returns"));
        assert!(is_section_header("args"));
        assert!(is_section_header("RETURNS"));
        assert!(is_section_header("See Also"));
        // Unknown names without colon are NOT headers
        assert!(!is_section_header("NotASection"));
        assert!(!is_section_header("SomeWord"));
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
    fn test_parse_entry_header_multiline_type() {
        let input = "x (Dict[str,\n        int]): The value.";
        let cursor = LineCursor::new(input);
        let header = parse_entry_header(&cursor);
        assert_eq!(header.name.source_text(input), "x");
        let ti = header.type_info.unwrap();
        assert_eq!(
            ti.r#type.unwrap().source_text(input),
            "Dict[str,\n        int]"
        );
        assert_eq!(header.first_description.source_text(input), "The value.");
        assert_eq!(
            header.range.source_text(input),
            "x (Dict[str,\n        int]): The value."
        );
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
