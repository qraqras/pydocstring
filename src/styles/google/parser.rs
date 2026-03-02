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

use crate::ast::{TextRange, TextSize};
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
///
/// Returns `(range, next_header)` where `next_header` is `Some` if
/// collection was terminated by a section header.  In that case the
/// cursor is positioned on the header line.
fn collect_text_range(
    cursor: &mut LineCursor,
    stop_at_blank: bool,
) -> (TextRange, Option<GoogleSectionHeader>) {
    let mut first_content: Option<usize> = None;
    let mut last_content = cursor.line;
    let mut found_header = None;

    while !cursor.is_eof() {
        if let Some(header) = try_parse_section_header(cursor) {
            found_header = Some(header);
            break;
        }
        let trimmed = cursor.current_trimmed();
        if stop_at_blank && trimmed.is_empty() {
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

    let range = if let Some(first) = first_content {
        let first_line = cursor.line_text(first);
        let first_col = indent_len(first_line);
        let last_line = cursor.line_text(last_content);
        let last_col = indent_len(last_line) + last_line.trim().len();
        cursor.make_range(first, first_col, last_content, last_col)
    } else {
        TextRange::empty()
    };
    (range, found_header)
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

/// A partially-parsed entry whose continuation lines are still being
/// accumulated.  Flushed when the next entry, section header, or EOF
/// is reached.
struct PendingEntry {
    header: EntryHeader,
    entry_start_line: usize,
    entry_col: usize,
    cont_first: Option<usize>,
    cont_last: usize,
}

/// Create an empty [`GoogleSectionBody`] for an entry-based section.
#[rustfmt::skip]
fn empty_entry_body(kind: GoogleSectionKind) -> GoogleSectionBody {
    match kind {
        GoogleSectionKind::Args            => GoogleSectionBody::Args(Vec::new()),
        GoogleSectionKind::KeywordArgs     => GoogleSectionBody::KeywordArgs(Vec::new()),
        GoogleSectionKind::OtherParameters => GoogleSectionBody::OtherParameters(Vec::new()),
        GoogleSectionKind::Receives        => GoogleSectionBody::Receives(Vec::new()),
        GoogleSectionKind::Raises          => GoogleSectionBody::Raises(Vec::new()),
        GoogleSectionKind::Warns           => GoogleSectionBody::Warns(Vec::new()),
        GoogleSectionKind::Attributes      => GoogleSectionBody::Attributes(Vec::new()),
        GoogleSectionKind::Methods         => GoogleSectionBody::Methods(Vec::new()),
        GoogleSectionKind::SeeAlso         => GoogleSectionBody::SeeAlso(Vec::new()),
        _ => unreachable!(),
    }
}

/// Finalise a pending entry and push it into `body`.
fn flush_pending_entry(cursor: &LineCursor, body: &mut GoogleSectionBody, pe: PendingEntry) {
    let cont_range = build_content_range(cursor, pe.cont_first, pe.cont_last);
    let full_desc = merge_descriptions(pe.header.first_description, cont_range);
    let range_end = if full_desc.is_empty() {
        pe.header.range.end()
    } else {
        full_desc.end()
    };
    let (end_line, end_col) = cursor.offset_to_line_col(range_end.raw() as usize);
    let entry_range = cursor.make_range(pe.entry_start_line, pe.entry_col, end_line, end_col);

    let (r#type, optional, open_bracket, close_bracket) = match &pe.header.type_info {
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
                name: pe.header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: pe.header.colon,
                description: full_desc,
                optional,
            });
        }
        GoogleSectionBody::Raises(v) => {
            v.push(GoogleException {
                range: entry_range,
                r#type: pe.header.name,
                colon: pe.header.colon,
                description: full_desc,
            });
        }
        GoogleSectionBody::Warns(v) => {
            v.push(GoogleWarning {
                range: entry_range,
                warning_type: pe.header.name,
                colon: pe.header.colon,
                description: full_desc,
            });
        }
        GoogleSectionBody::Attributes(v) => {
            v.push(GoogleAttribute {
                range: entry_range,
                name: pe.header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: pe.header.colon,
                description: full_desc,
            });
        }
        GoogleSectionBody::Methods(v) => {
            v.push(GoogleMethod {
                range: entry_range,
                name: pe.header.name,
                open_bracket,
                r#type,
                close_bracket,
                colon: pe.header.colon,
                description: full_desc,
            });
        }
        GoogleSectionBody::SeeAlso(v) => {
            // Split the name span by comma into individual name spans.
            let name_text = pe.header.name.source_text(cursor.source());
            let base = pe.header.name.start().raw() as usize;
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
                colon: pe.header.colon,
                description: if full_desc.is_empty() {
                    None
                } else {
                    Some(full_desc)
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

/// Wrap a freetext `TextRange` in the appropriate [`GoogleSectionBody`] variant.
#[rustfmt::skip]
fn wrap_freetext(kind: GoogleSectionKind, range: TextRange) -> GoogleSectionBody {
    match kind {
        GoogleSectionKind::Notes     => GoogleSectionBody::Notes(range),
        GoogleSectionKind::Examples  => GoogleSectionBody::Examples(range),
        GoogleSectionKind::Todo      => GoogleSectionBody::Todo(range),
        GoogleSectionKind::References => GoogleSectionBody::References(range),
        GoogleSectionKind::Warnings  => GoogleSectionBody::Warnings(range),
        GoogleSectionKind::Attention => GoogleSectionBody::Attention(range),
        GoogleSectionKind::Caution   => GoogleSectionBody::Caution(range),
        GoogleSectionKind::Danger    => GoogleSectionBody::Danger(range),
        GoogleSectionKind::Error     => GoogleSectionBody::Error(range),
        GoogleSectionKind::Hint      => GoogleSectionBody::Hint(range),
        GoogleSectionKind::Important => GoogleSectionBody::Important(range),
        GoogleSectionKind::Tip       => GoogleSectionBody::Tip(range),
        GoogleSectionKind::Unknown   => GoogleSectionBody::Unknown(range),
        _ => unreachable!(),
    }
}

// =============================================================================
// Section body parsers
// =============================================================================

/// Consume body lines for an entry-based section (Args, Raises, etc.).
///
/// Reads lines from `cursor` until a section header or EOF is reached.
/// If a header is found it is stored in `*next_header` (not consumed).
fn parse_entry_body(
    cursor: &mut LineCursor,
    kind: GoogleSectionKind,
    next_header: &mut Option<GoogleSectionHeader>,
) -> GoogleSectionBody {
    let mut body = empty_entry_body(kind);
    let mut entry_indent: Option<usize> = None;
    let mut pending: Option<PendingEntry> = None;

    while !cursor.is_eof() {
        if let Some(h) = try_parse_section_header(cursor) {
            *next_header = Some(h);
            break;
        }
        if cursor.current_trimmed().is_empty() {
            cursor.advance();
            continue;
        }
        let indent_cols = cursor.current_indent_columns();
        let ei = *entry_indent.get_or_insert(indent_cols);
        if indent_cols <= ei {
            // New entry: flush any pending, parse header
            if let Some(pe) = pending.take() {
                flush_pending_entry(cursor, &mut body, pe);
            }
            let header = parse_entry_header(cursor);
            pending = Some(PendingEntry {
                entry_start_line: cursor.line,
                entry_col: cursor.current_indent(),
                cont_first: None,
                cont_last: cursor.line,
                header,
            });
        } else if let Some(pe) = &mut pending {
            // Continuation line for current entry
            if pe.cont_first.is_none() {
                pe.cont_first = Some(cursor.line);
            }
            pe.cont_last = cursor.line;
        }
        cursor.advance();
    }
    if let Some(pe) = pending {
        flush_pending_entry(cursor, &mut body, pe);
    }
    body
}

/// Consume body lines for a Returns / Yields section.
///
/// The first non-blank line is parsed as `type: description`; remaining
/// lines are treated as continuation.
fn parse_returns_body(
    cursor: &mut LineCursor,
    kind: GoogleSectionKind,
    next_header: &mut Option<GoogleSectionHeader>,
) -> GoogleSectionBody {
    let mut return_type: Option<TextRange> = None;
    let mut colon: Option<TextRange> = None;
    let mut first_desc = TextRange::empty();
    let mut header_end = TextSize::from(0u32);
    let mut entry_start: Option<usize> = None;
    let mut entry_col: usize = 0;
    let mut cont_first: Option<usize> = None;
    let mut cont_last: usize = 0;

    while !cursor.is_eof() {
        if let Some(h) = try_parse_section_header(cursor) {
            *next_header = Some(h);
            break;
        }
        if cursor.current_trimmed().is_empty() {
            cursor.advance();
            continue;
        }
        if entry_start.is_none() {
            // First content line — parse type and description
            entry_start = Some(cursor.line);
            entry_col = cursor.current_indent();
            cont_last = cursor.line;

            let trimmed = cursor.current_trimmed();
            let col = cursor.current_indent();
            if let Some(colon_pos) = find_entry_colon(trimmed) {
                let type_str = trimmed[..colon_pos].trim_end();
                let after_colon = &trimmed[colon_pos + 1..];
                let desc_str = after_colon.trim_start();
                let ws_after = after_colon.len() - desc_str.len();
                return_type = Some(cursor.make_line_range(cursor.line, col, type_str.len()));
                colon = Some(cursor.make_line_range(cursor.line, col + colon_pos, 1));
                let desc_start = col + colon_pos + 1 + ws_after;
                first_desc = if desc_str.is_empty() {
                    TextRange::empty()
                } else {
                    cursor.make_line_range(cursor.line, desc_start, desc_str.len())
                };
            } else {
                first_desc = cursor.current_trimmed_range();
            }
            header_end = TextSize::from(cursor.substr_offset(trimmed) + trimmed.len());
        } else {
            // Continuation line
            if cont_first.is_none() {
                cont_first = Some(cursor.line);
            }
            cont_last = cursor.line;
        }
        cursor.advance();
    }

    let ret = if let Some(es) = entry_start {
        let cont_range = build_content_range(cursor, cont_first, cont_last);
        let full_desc = merge_descriptions(first_desc, cont_range);
        let range_end = if full_desc.is_empty() {
            header_end
        } else {
            full_desc.end()
        };
        let (end_line, end_col) = cursor.offset_to_line_col(range_end.raw() as usize);
        GoogleReturns {
            range: cursor.make_range(es, entry_col, end_line, end_col),
            return_type,
            colon,
            description: full_desc,
        }
    } else {
        GoogleReturns {
            range: TextRange::empty(),
            return_type: None,
            colon: None,
            description: TextRange::empty(),
        }
    };
    if kind == GoogleSectionKind::Yields {
        GoogleSectionBody::Yields(ret)
    } else {
        GoogleSectionBody::Returns(ret)
    }
}

/// Consume body lines for a freetext section (Notes, Examples, etc.).
fn parse_freetext_body(
    cursor: &mut LineCursor,
    kind: GoogleSectionKind,
    next_header: &mut Option<GoogleSectionHeader>,
) -> GoogleSectionBody {
    let mut first_content: Option<usize> = None;
    let mut last_content: usize = 0;

    while !cursor.is_eof() {
        if let Some(h) = try_parse_section_header(cursor) {
            *next_header = Some(h);
            break;
        }
        if cursor.current_trimmed().is_empty() {
            cursor.advance();
            continue;
        }
        if first_content.is_none() {
            first_content = Some(cursor.line);
        }
        last_content = cursor.line;
        cursor.advance();
    }

    wrap_freetext(
        kind,
        build_content_range(cursor, first_content, last_content),
    )
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

    // Cached section header found while parsing the previous region.
    // Avoids redundant `try_parse_section_header` calls at boundaries.
    let mut next_header: Option<GoogleSectionHeader>;

    // --- Summary ---
    // `collect_text_range` returns the header it bumped into (if any).
    {
        let (range, nh) = collect_text_range(&mut line_cursor, true);
        if !range.is_empty() {
            docstring.summary = Some(range);
        }
        next_header = nh;
        if next_header.is_none() {
            line_cursor.skip_blanks();
            if line_cursor.is_eof() {
                docstring.range = line_cursor.full_range();
                return docstring;
            }
        }
    }

    // --- Extended Summary ---
    if next_header.is_none() {
        let (range, nh) = collect_text_range(&mut line_cursor, false);
        if !range.is_empty() {
            docstring.extended_summary = Some(range);
        }
        next_header = nh;
        if next_header.is_none() {
            line_cursor.skip_blanks();
            if line_cursor.is_eof() {
                docstring.range = line_cursor.full_range();
                return docstring;
            }
        }
    }

    // --- Sections ---
    while !line_cursor.is_eof() {
        // Skip blank lines between sections
        if line_cursor.current_trimmed().is_empty() {
            line_cursor.advance();
            continue;
        }

        // Detect section header (use cached or detect)
        let header = next_header
            .take()
            .or_else(|| try_parse_section_header(&line_cursor));

        let Some(header) = header else {
            // Stray line (outside any section)
            docstring.items.push(GoogleDocstringItem::StrayLine(
                line_cursor.current_trimmed_range(),
            ));
            line_cursor.advance();
            continue;
        };

        line_cursor.advance(); // skip header line
        let header_start = header.range.start().raw() as usize;
        let kind = header.kind;

        // Parse section body (branching by kind)
        #[rustfmt::skip]
        let body = match kind {
            GoogleSectionKind::Args            => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::KeywordArgs     => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::OtherParameters => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Receives        => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Raises          => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Warns           => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Attributes      => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Methods         => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::SeeAlso         => parse_entry_body   (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Returns         => parse_returns_body (&mut line_cursor, kind, &mut next_header),
            GoogleSectionKind::Yields          => parse_returns_body (&mut line_cursor, kind, &mut next_header),
            _                                  => parse_freetext_body(&mut line_cursor, kind, &mut next_header),
        };

        let range = line_cursor.span_back_from_cursor(header_start);
        docstring
            .items
            .push(GoogleDocstringItem::Section(GoogleSection {
                range,
                header,
                body,
            }));
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
