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

use crate::ast::{
    Spanned, TextRange, TextSize, build_line_offsets, indent_len, line_text, make_range,
    make_spanned, num_lines, offset_to_line_col, substr_offset,
};
use crate::styles::google::ast::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleException, GoogleMethod, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader, GoogleSectionKind, GoogleSeeAlsoItem,
    GoogleWarning,
};

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

/// Check if a line is a Google-style section header at the given base indentation.
///
/// A section header is a line at `base_indent` that matches one of:
/// - `Word:` / `Two Words:` — standard form with colon
/// - `Word :` — colon preceded by whitespace
/// - `Word` — colonless form, only for known section names
///
/// For the colon forms, any short name (≤ 40 chars, starts with alpha, no
/// embedded colons) is accepted (dispatched as Unknown if unrecognised).
/// For the colonless form, only names in [`KNOWN_SECTIONS`] are accepted
/// to avoid treating ordinary text lines as headers.
fn is_section_header(line: &str, base_indent: usize) -> bool {
    let indent = indent_len(line);
    if indent != base_indent {
        return false;
    }
    let trimmed = line.trim();
    let (name, has_colon) = extract_section_name(trimmed);

    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return false;
    }

    if has_colon {
        // Standard / space-before-colon form: accept any short name without
        // embedded colons.
        name.len() <= 40 && !name.contains(':')
    } else {
        // Colonless form: only known names.
        GoogleSectionKind::is_known(&name.to_ascii_lowercase())
    }
}

// =============================================================================
// Bracket / optional helpers
// =============================================================================

/// Find the matching closing bracket for an opening bracket at `open_pos`.
///
/// Handles nested `()`, `[]`, `{}`.
fn find_matching_close(text: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in text[open_pos..].char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_pos + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Strip a trailing `optional` marker from a type annotation.
///
/// Finds the last `,` and checks whether everything after it (ignoring
/// whitespace) is `optional`.  Works regardless of how many spaces
/// appear between the comma and the word.
///
/// Returns `(clean_type, optional_byte_offset)` where the offset is
/// relative to the start of `type_content` and points to the `o` in
/// `optional`.
fn strip_optional(type_content: &str) -> (&str, Option<usize>) {
    if let Some(comma_pos) = type_content.rfind(',') {
        let after_comma = &type_content[comma_pos + 1..];
        if after_comma.trim() == "optional" {
            let ws_len = after_comma.len() - after_comma.trim_start().len();
            return (
                type_content[..comma_pos].trim(),
                Some(comma_pos + 1 + ws_len),
            );
        }
    }
    if type_content.trim() == "optional" {
        return ("", Some(type_content.find("optional").unwrap_or(0)));
    }
    (type_content, None)
}

// =============================================================================
// Description collector
// =============================================================================

/// Collect indented description continuation lines starting at `start`.
///
/// Stops at:
/// - Section headers at `base_indent`
/// - Non-empty lines at or below `entry_indent` (i.e. a new entry)
/// - End of input
fn collect_description(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    entry_indent: usize,
    base_indent: usize,
) -> (Spanned<String>, usize) {
    let mut i = start;
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();

        // Non-empty line at or below entry indent ⇒ new entry
        if !trimmed.is_empty() && indent_len(line) <= entry_indent {
            break;
        }

        desc_parts.push(trimmed);
        if !trimmed.is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(i);
            }
            last_content_line = i;
        }
        i += 1;
    }

    // Trim leading / trailing empty entries
    while desc_parts.last().is_some_and(|l| l.is_empty()) {
        desc_parts.pop();
    }
    while desc_parts.first().is_some_and(|l| l.is_empty()) {
        desc_parts.remove(0);
    }

    let text = desc_parts.join("\n");

    if let Some(first) = first_content_line {
        let first_line = line_text(source, first, offsets);
        let first_col = indent_len(first_line);
        let last_line = line_text(source, last_content_line, offsets);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        (
            make_spanned(text, first, first_col, last_content_line, last_col, offsets),
            i,
        )
    } else {
        (Spanned::empty_string(), i)
    }
}

/// Merge a first-line description fragment with a continuation `Spanned<String>`.
fn build_full_description(
    first_line: &str,
    first_col: usize,
    first_line_idx: usize,
    cont: &Spanned<String>,
    offsets: &[usize],
) -> Spanned<String> {
    if first_line.is_empty() {
        if cont.value.is_empty() {
            return Spanned::empty_string();
        }
        return cont.clone();
    }
    if cont.value.is_empty() {
        return make_spanned(
            first_line.to_string(),
            first_line_idx,
            first_col,
            first_line_idx,
            first_col + first_line.len(),
            offsets,
        );
    }
    let combined = format!("{}\n{}", first_line, cont.value);
    let (end_line, end_col) = offset_to_line_col(cont.range.end().raw() as usize, offsets);
    make_spanned(
        combined,
        first_line_idx,
        first_col,
        end_line,
        end_col,
        offsets,
    )
}

// =============================================================================
// Entry header parsing
// =============================================================================

/// Type information from a parsed entry header.
struct TypeInfo<'a> {
    /// Clean type string (without optional marker).
    clean_type: &'a str,
    /// Absolute byte offset of the clean type in source.
    type_start: usize,
    /// Absolute byte offset of `optional` in source, if present.
    optional_start: Option<usize>,
}

/// Parsed components of a Google-style entry header.
///
/// All byte offsets are absolute offsets within the source string.
struct EntryHeader<'a> {
    /// Entry name (parameter name, exception type, etc.).
    name: &'a str,
    /// Absolute byte offset of name in source.
    name_start: usize,
    /// Type annotation info.
    type_info: Option<TypeInfo<'a>>,
    /// First-line description text.
    desc_text: &'a str,
    /// Absolute byte offset of description text in source.
    desc_start: usize,
    /// Line index where the header ends (e.g. line of the closing paren).
    /// Description continuation starts on `end_line + 1`.
    end_line: usize,
}

/// Parse a Google-style entry header starting at `line_idx`.
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
fn parse_entry_header<'a>(
    source: &'a str,
    line_idx: usize,
    offsets: &[usize],
    #[allow(unused_variables)] total_lines: usize,
) -> EntryHeader<'a> {
    let line = line_text(source, line_idx, offsets);
    let trimmed = line.trim();
    let entry_start = substr_offset(source, trimmed);

    // --- Pattern 1: `name (type): desc` ---
    // Find the first `(` preceded by whitespace.
    if let Some(rel_paren) = trimmed
        .find('(')
        .filter(|&p| p > 0 && trimmed.as_bytes()[p - 1].is_ascii_whitespace())
    {
        let abs_paren = entry_start + rel_paren;
        // find_matching_close on full source — crosses line boundaries
        if let Some(abs_close) = find_matching_close(source, abs_paren) {
            let name = trimmed[..rel_paren].trim_end();
            let name_start = entry_start;

            // Type content between the parentheses (may span multiple lines)
            let type_raw = &source[abs_paren + 1..abs_close];
            let type_trimmed = type_raw.trim();
            let type_start = if !type_trimmed.is_empty() {
                substr_offset(source, type_trimmed)
            } else {
                abs_paren + 1
            };

            // Strip optional marker
            let (clean_type, opt_rel) = strip_optional(type_trimmed);
            let opt_start = opt_rel.map(|r| type_start + r);

            let type_info = if clean_type.is_empty() && opt_start.is_some() {
                // Only `optional` inside parens — no type, but we still record optional
                Some(TypeInfo {
                    clean_type: "",
                    type_start,
                    optional_start: opt_start,
                })
            } else if !clean_type.is_empty() {
                Some(TypeInfo {
                    clean_type,
                    type_start: substr_offset(source, clean_type),
                    optional_start: opt_start,
                })
            } else {
                None
            };

            // Find which line the close paren is on
            let close_line = offset_to_line_col(abs_close, offsets).0;

            // Description after `)` on the same line as the close paren
            let close_line_str = line_text(source, close_line, offsets);
            let close_line_end =
                substr_offset(source, close_line_str) + close_line_str.len();
            let after_close = &source[abs_close + 1..close_line_end];
            let (desc_text, desc_start) =
                extract_desc_after_colon(after_close, abs_close + 1);

            return EntryHeader {
                name,
                name_start,
                type_info,
                desc_text,
                desc_start,
                end_line: close_line,
            };
        }
    }

    // --- Pattern 2: `name: description` (no type) ---
    if let Some(colon_rel) = trimmed.find(": ") {
        let name = &trimmed[..colon_rel];
        return EntryHeader {
            name,
            name_start: entry_start,
            type_info: None,
            desc_text: &trimmed[colon_rel + 2..],
            desc_start: entry_start + colon_rel + 2,
            end_line: line_idx,
        };
    }

    // --- Pattern 3: `name:` with description on the next line ---
    if let Some(name) = trimmed.strip_suffix(':') {
        return EntryHeader {
            name,
            name_start: entry_start,
            type_info: None,
            desc_text: "",
            desc_start: entry_start + trimmed.len(),
            end_line: line_idx,
        };
    }

    // --- Fallback: bare name or plain text ---
    EntryHeader {
        name: trimmed,
        name_start: entry_start,
        type_info: None,
        desc_text: "",
        desc_start: entry_start + trimmed.len(),
        end_line: line_idx,
    }
}

/// Extract description text after a colon following the closing paren.
///
/// `after_paren` is the portion of text after `)`, and `base_offset` is its
/// byte offset within the source.
fn extract_desc_after_colon(after_paren: &str, base_offset: usize) -> (&str, usize) {
    let stripped = after_paren.trim_start();
    if let Some(after_colon) = stripped.strip_prefix(':') {
        let desc = after_colon.trim_start();
        let leading_to_stripped = after_paren.len() - stripped.len();
        let leading_after_colon = after_colon.len() - desc.len();
        let offset = base_offset + leading_to_stripped + 1 + leading_after_colon;
        (desc, offset)
    } else {
        ("", base_offset + after_paren.len())
    }
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
/// assert_eq!(doc.summary.value, "Summary.");
///
/// let args: Vec<_> = doc.sections.iter().filter_map(|s| match &s.body {
///     GoogleSectionBody::Args(v) => Some(v.iter()),
///     _ => None,
/// }).flatten().collect();
/// assert_eq!(args.len(), 1);
/// assert_eq!(args[0].name.value, "x");
///
/// let returns: Vec<_> = doc.sections.iter().filter_map(|s| match &s.body {
///     GoogleSectionBody::Returns(v) => Some(v.iter()),
///     _ => None,
/// }).flatten().collect();
/// assert_eq!(returns.len(), 1);
/// ```
pub fn parse_google(input: &str) -> GoogleDocstring {
    let offsets = build_line_offsets(input);
    let total_lines = num_lines(input, &offsets);
    let mut docstring = GoogleDocstring::new();
    docstring.source = input.to_string();

    if total_lines == 0 {
        return docstring;
    }

    let mut i = 0;

    // --- Skip leading blank lines ---
    while i < total_lines && line_text(input, i, &offsets).trim().is_empty() {
        i += 1;
    }
    if i >= total_lines {
        return docstring;
    }

    // Detect base indentation from the first non-empty line
    let base_indent = indent_len(line_text(input, i, &offsets));

    // --- Summary ---
    if !is_section_header(line_text(input, i, &offsets), base_indent) {
        let line = line_text(input, i, &offsets);
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
    while i < total_lines && line_text(input, i, &offsets).trim().is_empty() {
        i += 1;
    }

    // --- Extended description ---
    if i < total_lines && !is_section_header(line_text(input, i, &offsets), base_indent) {
        let start_line = i;
        let mut desc_lines: Vec<&str> = Vec::new();
        let mut last_non_empty = i;

        while i < total_lines && !is_section_header(line_text(input, i, &offsets), base_indent) {
            let trimmed = line_text(input, i, &offsets).trim();
            desc_lines.push(trimmed);
            if !trimmed.is_empty() {
                last_non_empty = i;
            }
            i += 1;
        }

        let keep = last_non_empty - start_line + 1;
        desc_lines.truncate(keep);

        let joined = desc_lines.join("\n");
        if !joined.trim().is_empty() {
            let first_line = line_text(input, start_line, &offsets);
            let first_col = indent_len(first_line);
            let last_line = line_text(input, last_non_empty, &offsets);
            let last_trimmed = last_line.trim();
            let last_col = indent_len(last_line) + last_trimmed.len();
            docstring.extended_summary = Some(make_spanned(
                joined,
                start_line,
                first_col,
                last_non_empty,
                last_col,
                &offsets,
            ));
        }
    }

    // --- Sections ---
    while i < total_lines {
        let cur_line = line_text(input, i, &offsets);
        if cur_line.trim().is_empty() {
            i += 1;
            continue;
        }

        if is_section_header(cur_line, base_indent) {
            let section_start = i;
            let header_line = cur_line;
            let header_trimmed = header_line.trim();
            let header_col = indent_len(header_line);

            // Extract the section name and whether a colon is present.
            // Handles "Args:", "Args :", and colonless "Args".
            let (raw_name, has_colon) = extract_section_name(header_trimmed);
            let header_name = raw_name.trim_end();

            let colon = if has_colon {
                // Colon is always the last character of the trimmed line
                let colon_col = header_col + header_trimmed.len() - 1;
                Some(make_spanned(
                    ":".to_string(),
                    i,
                    colon_col,
                    i,
                    colon_col + 1,
                    &offsets,
                ))
            } else {
                None
            };

            let header = GoogleSectionHeader {
                range: make_range(
                    i,
                    header_col,
                    i,
                    header_col + header_trimmed.len(),
                    &offsets,
                ),
                name: make_spanned(
                    header_name.to_string(),
                    i,
                    header_col,
                    i,
                    header_col + header_name.len(),
                    &offsets,
                ),
                colon,
            };

            i += 1; // skip header line

            let normalized = header_name.to_ascii_lowercase();
            let section_kind = GoogleSectionKind::from_name(&normalized);
            let (body, next_i) = match section_kind {
                // ----- Parameter-like sections -----
                Some(GoogleSectionKind::Args) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Args(args), ni)
                }
                Some(GoogleSectionKind::KeywordArgs) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::KeywordArgs(args), ni)
                }
                Some(GoogleSectionKind::OtherParameters) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::OtherParameters(args), ni)
                }
                Some(GoogleSectionKind::Receives) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Receives(args), ni)
                }
                // ----- Return/yield sections -----
                Some(GoogleSectionKind::Returns) => {
                    let (returns, ni) =
                        parse_returns_section(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Returns(returns), ni)
                }
                Some(GoogleSectionKind::Yields) => {
                    let (yields, ni) =
                        parse_returns_section(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Yields(yields), ni)
                }
                // ----- Exception/warning sections -----
                Some(GoogleSectionKind::Raises) => {
                    let (raises, ni) =
                        parse_raises_section(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Raises(raises), ni)
                }
                Some(GoogleSectionKind::Warns) => {
                    let (raises, ni) =
                        parse_raises_section(input, i, &offsets, total_lines, base_indent);
                    let warns = raises
                        .into_iter()
                        .map(|e| GoogleWarning {
                            range: e.range,
                            warning_type: e.r#type,
                            description: e.description,
                        })
                        .collect();
                    (GoogleSectionBody::Warns(warns), ni)
                }
                // ----- Structured sections -----
                Some(GoogleSectionKind::Attributes) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    let attrs = args
                        .into_iter()
                        .map(|a| GoogleAttribute {
                            range: a.range,
                            name: a.name,
                            r#type: a.r#type,
                            description: a.description,
                        })
                        .collect();
                    (GoogleSectionBody::Attributes(attrs), ni)
                }
                Some(GoogleSectionKind::Methods) => {
                    let (args, ni) = parse_args(input, i, &offsets, total_lines, base_indent);
                    let methods = args
                        .into_iter()
                        .map(|a| GoogleMethod {
                            range: a.range,
                            name: a.name,
                            description: a.description,
                        })
                        .collect();
                    (GoogleSectionBody::Methods(methods), ni)
                }
                Some(GoogleSectionKind::SeeAlso) => {
                    let (items, ni) =
                        parse_see_also_section(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::SeeAlso(items), ni)
                }
                // ----- Free-text / admonition sections -----
                Some(GoogleSectionKind::Notes) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Notes(content), ni)
                }
                Some(GoogleSectionKind::Examples) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Examples(content), ni)
                }
                Some(GoogleSectionKind::Todo) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Todo(content), ni)
                }
                Some(GoogleSectionKind::References) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::References(content), ni)
                }
                Some(GoogleSectionKind::Warnings) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Warnings(content), ni)
                }
                Some(GoogleSectionKind::Attention) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Attention(content), ni)
                }
                Some(GoogleSectionKind::Caution) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Caution(content), ni)
                }
                Some(GoogleSectionKind::Danger) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Danger(content), ni)
                }
                Some(GoogleSectionKind::Error) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Error(content), ni)
                }
                Some(GoogleSectionKind::Hint) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Hint(content), ni)
                }
                Some(GoogleSectionKind::Important) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Important(content), ni)
                }
                Some(GoogleSectionKind::Tip) => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Tip(content), ni)
                }
                None => {
                    let (content, ni) =
                        parse_section_content(input, i, &offsets, total_lines, base_indent);
                    (GoogleSectionBody::Unknown(content), ni)
                }
            };

            // Compute section span
            let section_end_line = {
                let mut end = next_i.saturating_sub(1);
                while end > section_start {
                    let end_line = line_text(input, end, &offsets);
                    if !end_line.trim().is_empty() {
                        break;
                    }
                    end -= 1;
                }
                end
            };
            let section_end_col = {
                let end_line = line_text(input, section_end_line, &offsets);
                indent_len(end_line) + end_line.trim().len()
            };

            docstring.sections.push(GoogleSection {
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

    // --- Docstring span ---
    let last_line_idx = total_lines.saturating_sub(1);
    let last_col = line_text(input, last_line_idx, &offsets).len();
    docstring.range = make_range(0, 0, last_line_idx, last_col, &offsets);

    docstring
}

// =============================================================================
// Args parsing
// =============================================================================

/// Parse the Args / Arguments section body.
fn parse_args(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    base_indent: usize,
) -> (Vec<GoogleArg>, usize) {
    let mut args = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        let ei = *entry_indent.get_or_insert(indent);

        // Entry line at entry indent level
        if indent <= ei {
            let col = indent;
            let entry_start_line = i;

            let header = parse_entry_header(source, i, offsets, total_lines);

            // Name
            let name_spanned = Spanned::new(
                header.name.to_string(),
                TextRange::new(
                    TextSize::new(header.name_start as u32),
                    TextSize::new((header.name_start + header.name.len()) as u32),
                ),
            );

            // Type and optional
            let (arg_type, optional) = match &header.type_info {
                Some(ti) => {
                    let arg_t = if !ti.clean_type.is_empty() {
                        Some(Spanned::new(
                            ti.clean_type.to_string(),
                            TextRange::new(
                                TextSize::new(ti.type_start as u32),
                                TextSize::new((ti.type_start + ti.clean_type.len()) as u32),
                            ),
                        ))
                    } else {
                        None
                    };
                    let opt = ti.optional_start.map(|os| {
                        Spanned::new(
                            "optional".to_string(),
                            TextRange::new(
                                TextSize::new(os as u32),
                                TextSize::new((os + "optional".len()) as u32),
                            ),
                        )
                    });
                    (arg_t, opt)
                }
                None => (None, None),
            };

            // Description: inline fragment + continuation lines
            let first_desc = header.desc_text;
            let desc_first_line = header.end_line;
            let desc_first_col = if !first_desc.is_empty() {
                offset_to_line_col(header.desc_start, offsets).1
            } else {
                0 // irrelevant when first_desc is empty
            };

            i = header.end_line + 1;
            let (cont_desc, next_i) =
                collect_description(source, i, offsets, total_lines, ei, base_indent);
            let full_desc = build_full_description(
                first_desc,
                desc_first_col,
                desc_first_line,
                &cont_desc,
                offsets,
            );

            let (end_line, end_col) = if full_desc.value.is_empty() {
                let last_header_line = line_text(source, header.end_line, offsets);
                (
                    header.end_line,
                    indent_len(last_header_line) + last_header_line.trim().len(),
                )
            } else {
                offset_to_line_col(full_desc.range.end().raw() as usize, offsets)
            };

            args.push(GoogleArg {
                range: make_range(entry_start_line, col, end_line, end_col, offsets),
                name: name_spanned,
                r#type: arg_type,
                description: full_desc,
                optional,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    (args, i)
}

// =============================================================================
// Returns / Yields parsing
// =============================================================================

/// Parse the Returns / Yields section body.
///
/// Supports both typed and untyped entries:
/// ```text
/// int: The result.          # typed
/// The result description.   # untyped
/// ```
fn parse_returns_section(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    base_indent: usize,
) -> (Vec<GoogleReturns>, usize) {
    let mut returns = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        let ei = *entry_indent.get_or_insert(indent);

        if indent <= ei {
            let col = indent;
            let entry_start = i;

            // Try `type: description` pattern
            let (return_type, first_desc, desc_col) = if let Some(colon_pos) = trimmed.find(": ") {
                let type_str = &trimmed[..colon_pos];
                let desc_str = &trimmed[colon_pos + 2..];
                let type_col = col;
                let rt = Some(make_spanned(
                    type_str.to_string(),
                    i,
                    type_col,
                    i,
                    type_col + type_str.len(),
                    offsets,
                ));
                (rt, desc_str, col + colon_pos + 2)
            } else if let Some(type_str) = trimmed.strip_suffix(':') {
                // Type only, description on next line
                let type_col = col;
                let rt = Some(make_spanned(
                    type_str.to_string(),
                    i,
                    type_col,
                    i,
                    type_col + type_str.len(),
                    offsets,
                ));
                (rt, "", col + trimmed.len())
            } else {
                // No type — just description
                (None, trimmed, col)
            };

            i += 1;
            let (cont_desc, next_i) =
                collect_description(source, i, offsets, total_lines, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                offset_to_line_col(full_desc.range.end().raw() as usize, offsets)
            };

            returns.push(GoogleReturns {
                range: make_range(entry_start, col, end_line, end_col, offsets),
                return_type,
                description: full_desc,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    (returns, i)
}

// =============================================================================
// Raises parsing
// =============================================================================

/// Parse the Raises section body.
///
/// Format: `ExceptionType: description`
fn parse_raises_section(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    base_indent: usize,
) -> (Vec<GoogleException>, usize) {
    let mut raises = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        let ei = *entry_indent.get_or_insert(indent);

        if indent <= ei {
            let col = indent;
            let entry_start = i;

            let (exc_type_str, first_desc, desc_col) =
                if let Some(colon_pos) = trimmed.find(": ") {
                    let et = &trimmed[..colon_pos];
                    let desc = &trimmed[colon_pos + 2..];
                    (et, desc, col + colon_pos + 2)
                } else if let Some(prefix) = trimmed.strip_suffix(':') {
                    (prefix, "", col + trimmed.len())
                } else {
                    (trimmed, "", col + trimmed.len())
                };

            let exc_type = make_spanned(
                exc_type_str.to_string(),
                i,
                col,
                i,
                col + exc_type_str.len(),
                offsets,
            );

            i += 1;
            let (cont_desc, next_i) =
                collect_description(source, i, offsets, total_lines, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                offset_to_line_col(full_desc.range.end().raw() as usize, offsets)
            };

            raises.push(GoogleException {
                range: make_range(entry_start, col, end_line, end_col, offsets),
                r#type: exc_type,
                description: full_desc,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    (raises, i)
}

// =============================================================================
// Free-text section parsing
// =============================================================================

/// Parse a free-text section body (Notes, Examples, References, Warnings, …).
///
/// Collects all indented lines until the next section header.
fn parse_section_content(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    base_indent: usize,
) -> (Spanned<String>, usize) {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut i = start;
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();
        // Non-empty line at or below base indent ⇒ outside the section
        if !trimmed.is_empty() && indent_len(line) <= base_indent {
            break;
        }

        content_lines.push(trimmed);
        if !trimmed.is_empty() {
            if first_content_line.is_none() {
                first_content_line = Some(i);
            }
            last_content_line = i;
        }
        i += 1;
    }

    // Trim leading / trailing empty
    while content_lines.last().is_some_and(|l| l.is_empty()) {
        content_lines.pop();
    }
    while content_lines.first().is_some_and(|l| l.is_empty()) {
        content_lines.remove(0);
    }

    let text = content_lines.join("\n");

    let spanned = if let Some(first) = first_content_line {
        let first_line = line_text(source, first, offsets);
        let first_col = indent_len(first_line);
        let last_line = line_text(source, last_content_line, offsets);
        let last_trimmed = last_line.trim();
        let last_col = indent_len(last_line) + last_trimmed.len();
        make_spanned(text, first, first_col, last_content_line, last_col, offsets)
    } else {
        Spanned::empty_string()
    };

    (spanned, i)
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
fn parse_see_also_section(
    source: &str,
    start: usize,
    offsets: &[usize],
    total_lines: usize,
    base_indent: usize,
) -> (Vec<GoogleSeeAlsoItem>, usize) {
    let mut items = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

    while i < total_lines {
        let line = line_text(source, i, offsets);
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        let ei = *entry_indent.get_or_insert(indent);

        if indent <= ei {
            let col = indent;
            let entry_start = i;

            // Split on ` : ` or `: ` for description
            let (names_part, first_desc, desc_col) = if let Some(colon_pos) = trimmed.find(": ") {
                let n = &trimmed[..colon_pos];
                let desc = &trimmed[colon_pos + 2..];
                (n, desc, col + colon_pos + 2)
            } else if let Some(prefix) = trimmed.strip_suffix(':') {
                (prefix, "", col + trimmed.len())
            } else {
                (trimmed, "", col + trimmed.len())
            };

            // Parse comma-separated names
            let mut names = Vec::new();
            let mut name_offset = col;
            for part in names_part.split(',') {
                let name = part.trim();
                if !name.is_empty() {
                    // Find the actual position of this name within the line
                    let name_start = name_offset + (part.len() - part.trim_start().len());
                    names.push(make_spanned(
                        name.to_string(),
                        i,
                        name_start,
                        i,
                        name_start + name.len(),
                        offsets,
                    ));
                }
                name_offset += part.len() + 1; // +1 for the comma
            }

            i += 1;
            let (cont_desc, next_i) =
                collect_description(source, i, offsets, total_lines, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let description = if full_desc.value.is_empty() {
                None
            } else {
                Some(full_desc)
            };

            let (end_line, end_col) = if let Some(ref d) = description {
                offset_to_line_col(d.range.end().raw() as usize, offsets)
            } else {
                (entry_start, col + trimmed.len())
            };

            items.push(GoogleSeeAlsoItem {
                range: make_range(entry_start, col, end_line, end_col, offsets),
                names,
                description,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    (items, i)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- test-local helpers --

    fn args(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
        doc.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Args(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn returns(doc: &GoogleDocstring) -> Vec<&GoogleReturns> {
        doc.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Returns(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn raises(doc: &GoogleDocstring) -> Vec<&GoogleException> {
        doc.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    // -- helpers --

    #[test]
    fn test_is_section_header() {
        // Standard colon form
        assert!(is_section_header("Args:", 0));
        assert!(is_section_header("    Args:", 4));
        assert!(!is_section_header("    Args:", 0));
        // "NotASection:" is still detected as an unknown section header (has colon)
        assert!(is_section_header("NotASection:", 0));
        assert!(is_section_header("Returns:", 0));
        // Case-insensitive
        assert!(is_section_header("args:", 0));
        assert!(is_section_header("RETURNS:", 0));
        // Not a section header: contains embedded colon
        assert!(!is_section_header("key: value:", 0));
        // Not a section header: too long
        assert!(!is_section_header(
            "This is a very long line that should not be a section header:",
            0
        ));
        // Not a section header: starts with space but indent doesn't match
        assert!(!is_section_header("  Custom:", 4));

        // Space-before-colon form
        assert!(is_section_header("Args :", 0));
        assert!(is_section_header("Returns :", 0));
        assert!(is_section_header("    Args :", 4));

        // Colonless form — only known section names
        assert!(is_section_header("Args", 0));
        assert!(is_section_header("Returns", 0));
        assert!(is_section_header("    Args", 4));
        assert!(is_section_header("args", 0));
        assert!(is_section_header("RETURNS", 0));
        assert!(is_section_header("See Also", 0));
        // Unknown names without colon are NOT headers
        assert!(!is_section_header("NotASection", 0));
        assert!(!is_section_header("SomeWord", 0));
    }

    // -- entry header parsing --

    /// Helper to parse an entry header from a single-line string.
    fn header_from(text: &str) -> EntryHeader<'_> {
        let offsets = crate::ast::build_line_offsets(text);
        let total = num_lines(text, &offsets);
        parse_entry_header(text, 0, &offsets, total)
    }

    #[test]
    fn test_parse_entry_header_with_type() {
        let header = header_from("name (int): Description");
        assert_eq!(header.name, "name");
        assert!(header.type_info.is_some());
        let ti = header.type_info.unwrap();
        assert_eq!(ti.clean_type, "int");
        assert_eq!(header.desc_text, "Description");
    }

    #[test]
    fn test_parse_entry_header_optional() {
        let header = header_from("name (int, optional): Description");
        assert_eq!(header.name, "name");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.clean_type, "int");
        assert!(ti.optional_start.is_some());
    }

    #[test]
    fn test_parse_entry_header_no_type() {
        let header = header_from("name: Description");
        assert_eq!(header.name, "name");
        assert!(header.type_info.is_none());
        assert_eq!(header.desc_text, "Description");
    }

    #[test]
    fn test_parse_entry_header_complex_type() {
        let header = header_from("data (Dict[str, List[int]]): Values");
        assert_eq!(header.name, "data");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.clean_type, "Dict[str, List[int]]");
        assert_eq!(header.desc_text, "Values");
    }

    #[test]
    fn test_parse_entry_header_colon_only() {
        let header = header_from("x:");
        assert_eq!(header.name, "x");
        assert!(header.type_info.is_none());
        assert_eq!(header.desc_text, "");
    }

    #[test]
    fn test_parse_entry_header_varargs() {
        let header = header_from("*args: Positional arguments");
        assert_eq!(header.name, "*args");
        assert_eq!(header.desc_text, "Positional arguments");

        let header = header_from("**kwargs (dict): Keyword arguments");
        assert_eq!(header.name, "**kwargs");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.clean_type, "dict");
    }

    #[test]
    fn test_parse_entry_header_multiline_type() {
        let input = "x (Dict[str,\n        int]): The value.";
        let offsets = crate::ast::build_line_offsets(input);
        let total = num_lines(input, &offsets);
        let header = parse_entry_header(input, 0, &offsets, total);
        assert_eq!(header.name, "x");
        let ti = header.type_info.unwrap();
        assert_eq!(ti.clean_type, "Dict[str,\n        int]");
        assert_eq!(header.desc_text, "The value.");
        assert_eq!(header.end_line, 1);
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

    // -- full parser --

    #[test]
    fn test_parse_simple_summary() {
        let result = parse_google("Brief description.");
        assert_eq!(result.summary.value, "Brief description.");
    }

    #[test]
    fn test_parse_empty() {
        let result = parse_google("");
        assert_eq!(result.summary.value, "");
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_google("   \n   \n");
        assert_eq!(result.summary.value, "");
    }

    #[test]
    fn test_parse_summary_with_description() {
        let input = "Brief summary.\n\nExtended description.\nMore text.";
        let result = parse_google(input);
        assert_eq!(result.summary.value, "Brief summary.");
        assert_eq!(
            result.extended_summary.as_ref().unwrap().value,
            "Extended description.\nMore text."
        );
    }

    #[test]
    fn test_parse_args() {
        let input = "Summary.\n\nArgs:\n    x (int): The value.\n    y (str): The name.";
        let result = parse_google(input);
        let a = args(&result);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].name.value, "x");
        assert_eq!(a[0].r#type.as_ref().unwrap().value, "int");
        assert_eq!(a[0].description.value, "The value.");
        assert_eq!(a[1].name.value, "y");
    }

    #[test]
    fn test_parse_args_multiline_desc() {
        let input = "Summary.\n\nArgs:\n    x (int): First line.\n        Second line.";
        let result = parse_google(input);
        assert_eq!(
            args(&result)[0].description.value,
            "First line.\nSecond line."
        );
    }

    #[test]
    fn test_parse_args_multiline_type() {
        let input = "Summary.\n\nArgs:\n    x (Dict[str,\n            int]): The value.";
        let result = parse_google(input);
        let a = args(&result);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].name.value, "x");
        assert_eq!(
            a[0].r#type.as_ref().unwrap().value,
            "Dict[str,\n            int]"
        );
        assert_eq!(a[0].description.value, "The value.");
    }

    #[test]
    fn test_parse_returns() {
        let input = "Summary.\n\nReturns:\n    int: The result.";
        let result = parse_google(input);
        let r = returns(&result);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].return_type.as_ref().unwrap().value, "int");
        assert_eq!(r[0].description.value, "The result.");
    }

    #[test]
    fn test_parse_returns_multiple() {
        let input = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
        let result = parse_google(input);
        assert_eq!(returns(&result).len(), 2);
    }

    #[test]
    fn test_parse_raises() {
        let input = "Summary.\n\nRaises:\n    ValueError: If invalid.";
        let result = parse_google(input);
        let r = raises(&result);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].r#type.value, "ValueError");
        assert_eq!(r[0].description.value, "If invalid.");
    }

    #[test]
    fn test_parse_span_accuracy() {
        let input = "Summary line.";
        let result = parse_google(input);
        assert_eq!(result.summary.range.start().raw(), 0);
        assert_eq!(result.summary.range.end().raw(), 13);
        assert_eq!(
            result.summary.range.source_text(&result.source),
            "Summary line."
        );
    }
}
