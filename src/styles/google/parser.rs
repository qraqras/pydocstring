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

use crate::ast::{build_line_offsets, indent_len, make_span, make_spanned, Spanned};
use crate::error::{Diagnostic, ParseResult};
use crate::styles::google::ast::{
    GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader,
};

// =============================================================================
// Constants
// =============================================================================

/// Known Google-style section header names.
const GOOGLE_SECTION_NAMES: &[&str] = &[
    "Args:",
    "Arguments:",
    "Returns:",
    "Return:",
    "Raises:",
    "Yields:",
    "Yield:",
    "Example:",
    "Examples:",
    "Note:",
    "Notes:",
    "Attributes:",
    "Todo:",
    "References:",
    "Warnings:",
];

// =============================================================================
// Section detection
// =============================================================================

/// Check if a line is a Google-style section header at the given base indentation.
///
/// A section header is a line at `base_indent` that:
/// 1. Matches a known section name (case-insensitive), OR
/// 2. Looks like `Word:` or `Two Words:` (short, ends with `:`, no trailing content)
fn is_section_header(line: &str, base_indent: usize) -> bool {
    let indent = indent_len(line);
    if indent != base_indent {
        return false;
    }
    let trimmed = line.trim();
    if !trimmed.ends_with(':') || trimmed.len() < 2 {
        return false;
    }
    // Known sections always match
    if GOOGLE_SECTION_NAMES
        .iter()
        .any(|&s| s.eq_ignore_ascii_case(trimmed))
    {
        return true;
    }
    // Unknown section heuristic: short header (≤ 40 chars), no embedded colons
    // except the trailing one, starts with an alphabetic char
    let name_part = &trimmed[..trimmed.len() - 1];
    !name_part.is_empty()
        && name_part.len() <= 40
        && !name_part.contains(':')
        && name_part.starts_with(|c: char| c.is_ascii_alphabetic())
}

/// Detect the entry indentation level for a section body.
///
/// Scans forward from `start` to find the first non-empty line (that is not
/// itself a section header). Returns its indentation level, or `base_indent + 4`
/// as a sensible default.
fn detect_entry_indent(lines: &[&str], start: usize, base_indent: usize) -> usize {
    for i in start..lines.len() {
        let line = lines[i];
        if is_section_header(line, base_indent) {
            break;
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return indent_len(line);
        }
    }
    base_indent + 4
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

/// Check if brackets are balanced in a string.
fn is_brackets_balanced(s: &str) -> bool {
    let mut depth: i32 = 0;
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    depth == 0
}

/// Strip a trailing `, optional` (or `,optional`) marker from a type annotation.
///
/// Returns `(clean_type, optional_byte_offset)` where the offset is relative to the
/// start of `type_content` and points to the `o` in `optional`.
fn strip_optional(type_content: &str) -> (&str, Option<usize>) {
    if let Some(pos) = type_content.rfind(", optional") {
        let suffix = &type_content[pos + ", optional".len()..];
        if suffix.trim().is_empty() && is_brackets_balanced(&type_content[..pos]) {
            return (type_content[..pos].trim(), Some(pos + 2));
        }
    }
    if let Some(pos) = type_content.rfind(",optional") {
        let suffix = &type_content[pos + ",optional".len()..];
        if suffix.trim().is_empty() && is_brackets_balanced(&type_content[..pos]) {
            return (type_content[..pos].trim(), Some(pos + 1));
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    entry_indent: usize,
    base_indent: usize,
) -> (Spanned<String>, usize) {
    let mut i = start;
    let mut desc_parts: Vec<&str> = Vec::new();
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < lines.len() {
        let line = lines[i];
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
    make_spanned(
        combined,
        first_line_idx,
        first_col,
        cont.span.end.line as usize,
        cont.span.end.column as usize,
        offsets,
    )
}

// =============================================================================
// Entry header parsing
// =============================================================================

/// Parsed components of a Google-style entry header line.
///
/// All byte offsets are relative to the start of the **trimmed** line text.
struct EntryHeader<'a> {
    /// Entry name (parameter name, exception type, etc.).
    name: &'a str,
    /// Type annotation: `(clean_type, type_byte_offset, optional_byte_offset)`.
    ///
    /// `type_byte_offset` points to the start of the clean type string inside the
    /// trimmed text. `optional_byte_offset` points to the `o` in `optional`.
    type_info: Option<(&'a str, usize, Option<usize>)>,
    /// First-line description: `(text, byte_offset)`.
    desc: (&'a str, usize),
}

/// Parse a Google-style entry header line.
///
/// Recognised patterns:
/// - `name (type, optional): description`
/// - `name (type): description`
/// - `name: description`
/// - `*args: description`
/// - `**kwargs (dict): description`
///
/// All byte offsets in `EntryHeader` are relative to the start of `text`.
fn parse_entry_header(text: &str) -> EntryHeader<'_> {
    // --- Pattern 1: `name (type): desc` ---
    if let Some(space_paren) = text.find(" (") {
        let paren_start = space_paren + 1;
        if let Some(paren_end) = find_matching_close(text, paren_start) {
            let name = text[..space_paren].trim_end();

            // Type content between the parentheses
            let type_raw = &text[paren_start + 1..paren_end];
            let type_trimmed = type_raw.trim();
            let type_leading_ws = type_raw.len() - type_raw.trim_start().len();
            let type_offset = paren_start + 1 + type_leading_ws;

            // Strip optional marker
            let (clean_type, opt_rel) = strip_optional(type_trimmed);
            let opt_offset = opt_rel.map(|r| type_offset + r);

            let type_info = if clean_type.is_empty() && opt_offset.is_some() {
                // Only `optional` inside parens — no type, but we still record optional
                Some(("", type_offset, opt_offset))
            } else if !clean_type.is_empty() {
                Some((clean_type, type_offset, opt_offset))
            } else {
                None
            };

            // Description after `): ` or `):`
            let after_paren = &text[paren_end + 1..];
            let desc = extract_desc_after_colon(after_paren, paren_end + 1);

            return EntryHeader {
                name,
                type_info,
                desc,
            };
        }
    }

    // --- Pattern 2: `name: description` (no type) ---
    if let Some(colon_pos) = text.find(": ") {
        let name = &text[..colon_pos];
        let desc_start = colon_pos + 2;
        return EntryHeader {
            name,
            type_info: None,
            desc: (&text[desc_start..], desc_start),
        };
    }

    // --- Pattern 3: `name:` with description on the next line ---
    if text.ends_with(':') {
        return EntryHeader {
            name: &text[..text.len() - 1],
            type_info: None,
            desc: ("", text.len()),
        };
    }

    // --- Fallback: bare name or plain text ---
    EntryHeader {
        name: text,
        type_info: None,
        desc: ("", text.len()),
    }
}

/// Extract description text after a colon following the closing paren.
///
/// `after_paren` is the portion of text after `)`, and `base_offset` is its
/// byte offset within the full trimmed line.
fn extract_desc_after_colon(after_paren: &str, base_offset: usize) -> (&str, usize) {
    let stripped = after_paren.trim_start();
    if stripped.starts_with(':') {
        let after_colon = &stripped[1..];
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
///
/// let input = "Summary.\n\nArgs:\n    x (int): The value.\n\nReturns:\n    int: The result.";
/// let doc = &parse_google(input).value;
///
/// assert_eq!(doc.summary.value, "Summary.");
/// assert_eq!(doc.args().len(), 1);
/// assert_eq!(doc.args()[0].name.value, "x");
/// assert_eq!(doc.returns().len(), 1);
/// ```
pub fn parse_google(input: &str) -> ParseResult<GoogleDocstring> {
    let offsets = build_line_offsets(input);
    let lines: Vec<&str> = input.lines().collect();
    let mut docstring = GoogleDocstring::new();
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

    // Detect base indentation from the first non-empty line
    let base_indent = indent_len(lines[i]);

    // --- Summary ---
    if !is_section_header(lines[i], base_indent) {
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

    // --- Extended description ---
    if i < lines.len() && !is_section_header(lines[i], base_indent) {
        let start_line = i;
        let mut desc_lines: Vec<&str> = Vec::new();
        let mut last_non_empty = i;

        while i < lines.len() && !is_section_header(lines[i], base_indent) {
            let trimmed = lines[i].trim();
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
            let first_col = indent_len(lines[start_line]);
            let last_trimmed = lines[last_non_empty].trim();
            let last_col = indent_len(lines[last_non_empty]) + last_trimmed.len();
            docstring.description = Some(make_spanned(
                joined,
                start_line,
                first_col,
                last_non_empty,
                last_col,
                &offsets,
            ));
        }
    }

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // --- Sections ---
    while i < lines.len() {
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }

        if is_section_header(lines[i], base_indent) {
            let section_start = i;
            let header_line = lines[i];
            let header_trimmed = header_line.trim();
            let header_col = indent_len(header_line);

            // Build section header (name without trailing colon)
            let header_name = &header_trimmed[..header_trimmed.len() - 1];
            let header = GoogleSectionHeader {
                span: make_span(
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
            };

            i += 1; // skip header line

            let normalized = header_trimmed.to_ascii_lowercase();
            let (body, next_i) = match normalized.as_str() {
                "args:" | "arguments:" => {
                    let result = parse_args(&lines, i, &offsets, base_indent);
                    diagnostics.extend(result.diagnostics);
                    let (args, ni) = result.value;
                    (GoogleSectionBody::Args(args), ni)
                }
                "returns:" | "return:" => {
                    let result = parse_returns_section(&lines, i, &offsets, base_indent);
                    diagnostics.extend(result.diagnostics);
                    let (returns, ni) = result.value;
                    (GoogleSectionBody::Returns(returns), ni)
                }
                "yields:" | "yield:" => {
                    let result = parse_returns_section(&lines, i, &offsets, base_indent);
                    diagnostics.extend(result.diagnostics);
                    let (yields, ni) = result.value;
                    (GoogleSectionBody::Yields(yields), ni)
                }
                "raises:" => {
                    let result = parse_raises_section(&lines, i, &offsets, base_indent);
                    diagnostics.extend(result.diagnostics);
                    let (raises, ni) = result.value;
                    (GoogleSectionBody::Raises(raises), ni)
                }
                "attributes:" => {
                    let result = parse_attributes_section(&lines, i, &offsets, base_indent);
                    diagnostics.extend(result.diagnostics);
                    let (attrs, ni) = result.value;
                    (GoogleSectionBody::Attributes(attrs), ni)
                }
                "note:" | "notes:" => {
                    let (content, ni) =
                        parse_freetext_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::Note(content), ni)
                }
                "example:" | "examples:" => {
                    let (content, ni) =
                        parse_freetext_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::Example(content), ni)
                }
                "todo:" => {
                    let (items, ni) = parse_todo_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::Todo(items), ni)
                }
                "references:" => {
                    let (content, ni) =
                        parse_freetext_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::References(content), ni)
                }
                "warnings:" => {
                    let (content, ni) =
                        parse_freetext_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::Warnings(content), ni)
                }
                _ => {
                    let (content, ni) =
                        parse_freetext_section(&lines, i, &offsets, base_indent).value;
                    (GoogleSectionBody::Unknown(content), ni)
                }
            };

            // Detect empty section body
            let is_empty = match &body {
                GoogleSectionBody::Args(v) => v.is_empty(),
                GoogleSectionBody::Returns(v) | GoogleSectionBody::Yields(v) => v.is_empty(),
                GoogleSectionBody::Raises(v) => v.is_empty(),
                GoogleSectionBody::Attributes(v) => v.is_empty(),
                GoogleSectionBody::Note(s)
                | GoogleSectionBody::Example(s)
                | GoogleSectionBody::References(s)
                | GoogleSectionBody::Warnings(s)
                | GoogleSectionBody::Unknown(s) => s.value.is_empty(),
                GoogleSectionBody::Todo(v) => v.is_empty(),
            };
            if is_empty {
                diagnostics.push(Diagnostic::warning(
                    make_span(
                        section_start,
                        header_col,
                        section_start,
                        header_col + header_trimmed.len(),
                        &offsets,
                    ),
                    format!("empty section body for '{}'", header_name),
                ));
            }

            // Compute section span
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

            docstring.sections.push(GoogleSection {
                span: make_span(
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
    let last_line = lines.len().saturating_sub(1);
    let last_col = lines.last().map(|l| l.len()).unwrap_or(0);
    docstring.span = make_span(0, 0, last_line, last_col, &offsets);

    ParseResult::with_diagnostics(docstring, diagnostics)
}

// =============================================================================
// Args parsing
// =============================================================================

/// Parse the Args / Arguments section body.
fn parse_args(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Vec<GoogleArgument>, usize)> {
    let mut args = Vec::new();
    let mut diagnostics = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start, base_indent);

    while i < lines.len() {
        if is_section_header(lines[i], base_indent) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        // Entry line at entry indent level
        if indent <= entry_indent {
            let col = indent;
            let entry_start = i;

            // Detect unclosed type parentheses before parsing
            if let Some(sp) = trimmed.find(" (") {
                let paren_start = sp + 1;
                if find_matching_close(trimmed, paren_start).is_none() {
                    let paren_col = col + paren_start;
                    diagnostics.push(Diagnostic::warning(
                        make_span(i, paren_col, i, col + trimmed.len(), offsets),
                        "unclosed parenthesis in type annotation",
                    ));
                }
            }

            // Detect missing colon separator
            if !trimmed.contains(':') {
                diagnostics.push(Diagnostic::warning(
                    make_span(i, col, i, col + trimmed.len(), offsets),
                    format!(
                        "missing ':' after parameter name '{}'",
                        trimmed.split_whitespace().next().unwrap_or(trimmed)
                    ),
                ));
            }

            let header = parse_entry_header(trimmed);

            // Name
            let name_col = col;
            let name_spanned = make_spanned(
                header.name.to_string(),
                i,
                name_col,
                i,
                name_col + header.name.len(),
                offsets,
            );

            // Type and optional
            let (arg_type, optional) = match header.type_info {
                Some((type_str, type_off, opt_off)) => {
                    let arg_t = if !type_str.is_empty() {
                        let tc = col + type_off;
                        Some(make_spanned(
                            type_str.to_string(),
                            i,
                            tc,
                            i,
                            tc + type_str.len(),
                            offsets,
                        ))
                    } else {
                        None
                    };
                    let opt = opt_off.map(|o| {
                        let oc = col + o;
                        make_span(i, oc, i, oc + "optional".len(), offsets)
                    });
                    (arg_t, opt)
                }
                None => (None, None),
            };

            // Description: inline fragment + continuation lines
            let (first_desc, desc_off) = header.desc;
            let first_desc_col = col + desc_off;

            i += 1;
            let (cont_desc, next_i) =
                collect_description(lines, i, offsets, entry_indent, base_indent);
            let full_desc = build_full_description(
                first_desc,
                first_desc_col,
                entry_start,
                &cont_desc,
                offsets,
            );

            // Detect missing description
            if full_desc.value.is_empty() {
                diagnostics.push(Diagnostic::hint(
                    make_span(entry_start, col, entry_start, col + trimmed.len(), offsets),
                    format!("missing description for parameter '{}'", header.name),
                ));
            }

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                (
                    full_desc.span.end.line as usize,
                    full_desc.span.end.column as usize,
                )
            };

            args.push(GoogleArgument {
                span: make_span(entry_start, col, end_line, end_col, offsets),
                name: name_spanned,
                arg_type,
                description: full_desc,
                optional,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    ParseResult::with_diagnostics((args, i), diagnostics)
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Vec<GoogleReturns>, usize)> {
    let mut returns = Vec::new();
    let mut diagnostics = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start, base_indent);

    while i < lines.len() {
        if is_section_header(lines[i], base_indent) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        if indent <= entry_indent {
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
            } else if trimmed.ends_with(':') {
                // Type only, description on next line
                let type_str = &trimmed[..trimmed.len() - 1];
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
                collect_description(lines, i, offsets, entry_indent, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            // Detect missing description
            if full_desc.value.is_empty() {
                let label = return_type
                    .as_ref()
                    .map_or("return entry".to_string(), |rt| {
                        format!("return type '{}'", rt.value)
                    });
                diagnostics.push(Diagnostic::hint(
                    make_span(entry_start, col, entry_start, col + trimmed.len(), offsets),
                    format!("missing description for {}", label),
                ));
            }

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                (
                    full_desc.span.end.line as usize,
                    full_desc.span.end.column as usize,
                )
            };

            returns.push(GoogleReturns {
                span: make_span(entry_start, col, end_line, end_col, offsets),
                return_type,
                description: full_desc,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    ParseResult::with_diagnostics((returns, i), diagnostics)
}

// =============================================================================
// Raises parsing
// =============================================================================

/// Parse the Raises section body.
///
/// Format: `ExceptionType: description`
fn parse_raises_section(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Vec<GoogleException>, usize)> {
    let mut raises = Vec::new();
    let mut diagnostics = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start, base_indent);

    while i < lines.len() {
        if is_section_header(lines[i], base_indent) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        if indent <= entry_indent {
            let col = indent;
            let entry_start = i;

            // Detect missing colon separator
            if !trimmed.contains(':') {
                diagnostics.push(Diagnostic::warning(
                    make_span(i, col, i, col + trimmed.len(), offsets),
                    format!(
                        "missing ':' after exception type '{}'",
                        trimmed.split_whitespace().next().unwrap_or(trimmed)
                    ),
                ));
            }

            let (exc_type_str, first_desc, desc_col) = if let Some(colon_pos) = trimmed.find(": ") {
                let et = &trimmed[..colon_pos];
                let desc = &trimmed[colon_pos + 2..];
                (et, desc, col + colon_pos + 2)
            } else if trimmed.ends_with(':') {
                (&trimmed[..trimmed.len() - 1], "", col + trimmed.len())
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
                collect_description(lines, i, offsets, entry_indent, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            // Detect missing description
            if full_desc.value.is_empty() {
                diagnostics.push(Diagnostic::hint(
                    make_span(entry_start, col, entry_start, col + trimmed.len(), offsets),
                    format!("missing description for exception '{}'", exc_type_str),
                ));
            }

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                (
                    full_desc.span.end.line as usize,
                    full_desc.span.end.column as usize,
                )
            };

            raises.push(GoogleException {
                span: make_span(entry_start, col, end_line, end_col, offsets),
                exception_type: exc_type,
                description: full_desc,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    ParseResult::with_diagnostics((raises, i), diagnostics)
}

// =============================================================================
// Attributes parsing
// =============================================================================

/// Parse the Attributes section body.
///
/// Same format as Args: `name (type): description`
fn parse_attributes_section(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Vec<GoogleAttribute>, usize)> {
    let mut attrs = Vec::new();
    let mut diagnostics = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start, base_indent);

    while i < lines.len() {
        if is_section_header(lines[i], base_indent) {
            break;
        }

        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        if indent <= base_indent {
            break;
        }

        if indent <= entry_indent {
            let col = indent;
            let entry_start = i;

            // Detect unclosed type parentheses
            if let Some(sp) = trimmed.find(" (") {
                let paren_start = sp + 1;
                if find_matching_close(trimmed, paren_start).is_none() {
                    let paren_col = col + paren_start;
                    diagnostics.push(Diagnostic::warning(
                        make_span(i, paren_col, i, col + trimmed.len(), offsets),
                        "unclosed parenthesis in type annotation",
                    ));
                }
            }

            // Detect missing colon separator
            if !trimmed.contains(':') {
                diagnostics.push(Diagnostic::warning(
                    make_span(i, col, i, col + trimmed.len(), offsets),
                    format!(
                        "missing ':' after attribute name '{}'",
                        trimmed.split_whitespace().next().unwrap_or(trimmed)
                    ),
                ));
            }

            let header = parse_entry_header(trimmed);

            let name_col = col;
            let name_spanned = make_spanned(
                header.name.to_string(),
                i,
                name_col,
                i,
                name_col + header.name.len(),
                offsets,
            );

            let attr_type = match header.type_info {
                Some((type_str, type_off, _)) if !type_str.is_empty() => {
                    let tc = col + type_off;
                    Some(make_spanned(
                        type_str.to_string(),
                        i,
                        tc,
                        i,
                        tc + type_str.len(),
                        offsets,
                    ))
                }
                _ => None,
            };

            let (first_desc, desc_off) = header.desc;
            let first_desc_col = col + desc_off;

            i += 1;
            let (cont_desc, next_i) =
                collect_description(lines, i, offsets, entry_indent, base_indent);
            let full_desc = build_full_description(
                first_desc,
                first_desc_col,
                entry_start,
                &cont_desc,
                offsets,
            );

            // Detect missing description
            if full_desc.value.is_empty() {
                diagnostics.push(Diagnostic::hint(
                    make_span(entry_start, col, entry_start, col + trimmed.len(), offsets),
                    format!("missing description for attribute '{}'", header.name),
                ));
            }

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                (
                    full_desc.span.end.line as usize,
                    full_desc.span.end.column as usize,
                )
            };

            attrs.push(GoogleAttribute {
                span: make_span(entry_start, col, end_line, end_col, offsets),
                name: name_spanned,
                attr_type,
                description: full_desc,
            });

            i = next_i;
        } else {
            i += 1;
        }
    }

    ParseResult::with_diagnostics((attrs, i), diagnostics)
}

// =============================================================================
// Free-text section parsing
// =============================================================================

/// Parse a free-text section body (Note, Example, References, Warnings, …).
///
/// Collects all indented lines until the next section header.
fn parse_freetext_section(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Spanned<String>, usize)> {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut i = start;
    let mut first_content_line: Option<usize> = None;
    let mut last_content_line = start;

    while i < lines.len() {
        let line = lines[i];
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
// Todo section parsing
// =============================================================================

/// Parse the Todo section body.
///
/// Items are typically bulleted lines:
/// ```text
/// Todo:
///     * Item one.
///     * Item two.
/// ```
fn parse_todo_section(
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> ParseResult<(Vec<Spanned<String>>, usize)> {
    let mut items = Vec::new();
    let mut i = start;
    let entry_indent = detect_entry_indent(lines, start, base_indent);

    // (text, start_line, start_col, end_line, end_col)
    let mut current_item: Option<(String, usize, usize, usize, usize)> = None;

    while i < lines.len() {
        let line = lines[i];
        if is_section_header(line, base_indent) {
            break;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() && indent_len(line) <= base_indent {
            break;
        }

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let indent = indent_len(line);
        let is_bullet =
            trimmed.starts_with("* ") || trimmed.starts_with("- ") || trimmed.starts_with("+ ");

        // New item: at entry indent or bullet marker
        if indent <= entry_indent {
            // Flush previous item
            if let Some((text, sl, sc, el, ec)) = current_item.take() {
                items.push(make_spanned(text, sl, sc, el, ec, offsets));
            }

            let col = indent;
            let content = if is_bullet { &trimmed[2..] } else { trimmed };
            let content_col = if is_bullet { col + 2 } else { col };

            current_item = Some((
                content.to_string(),
                i,
                content_col,
                i,
                content_col + content.len(),
            ));
        } else if let Some(ref mut item) = current_item {
            // Continuation of current item
            item.0.push('\n');
            item.0.push_str(trimmed);
            item.3 = i;
            item.4 = indent + trimmed.len();
        }

        i += 1;
    }

    // Flush last item
    if let Some((text, sl, sc, el, ec)) = current_item {
        items.push(make_spanned(text, sl, sc, el, ec, offsets));
    }

    ParseResult::ok((items, i))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers --

    #[test]
    fn test_is_section_header() {
        assert!(is_section_header("Args:", 0));
        assert!(is_section_header("    Args:", 4));
        assert!(!is_section_header("    Args:", 0));
        // "NotASection:" is now detected as an unknown section header
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
    }

    #[test]
    fn test_detect_entry_indent_basic() {
        let lines = vec!["    x (int): Description."];
        assert_eq!(detect_entry_indent(&lines, 0, 0), 4);

        let lines = vec!["", "    x (int): Description."];
        assert_eq!(detect_entry_indent(&lines, 0, 0), 4);

        // Default when empty
        assert_eq!(detect_entry_indent(&[], 0, 0), 4);
    }

    // -- entry header parsing --

    #[test]
    fn test_parse_entry_header_with_type() {
        let header = parse_entry_header("name (int): Description");
        assert_eq!(header.name, "name");
        assert!(header.type_info.is_some());
        let (ts, _, _) = header.type_info.unwrap();
        assert_eq!(ts, "int");
        assert_eq!(header.desc.0, "Description");
    }

    #[test]
    fn test_parse_entry_header_optional() {
        let header = parse_entry_header("name (int, optional): Description");
        assert_eq!(header.name, "name");
        let (ts, _, opt) = header.type_info.unwrap();
        assert_eq!(ts, "int");
        assert!(opt.is_some());
    }

    #[test]
    fn test_parse_entry_header_no_type() {
        let header = parse_entry_header("name: Description");
        assert_eq!(header.name, "name");
        assert!(header.type_info.is_none());
        assert_eq!(header.desc.0, "Description");
    }

    #[test]
    fn test_parse_entry_header_complex_type() {
        let header = parse_entry_header("data (Dict[str, List[int]]): Values");
        assert_eq!(header.name, "data");
        let (ts, _, _) = header.type_info.unwrap();
        assert_eq!(ts, "Dict[str, List[int]]");
        assert_eq!(header.desc.0, "Values");
    }

    #[test]
    fn test_parse_entry_header_colon_only() {
        let header = parse_entry_header("x:");
        assert_eq!(header.name, "x");
        assert!(header.type_info.is_none());
        assert_eq!(header.desc.0, "");
    }

    #[test]
    fn test_parse_entry_header_varargs() {
        let header = parse_entry_header("*args: Positional arguments");
        assert_eq!(header.name, "*args");
        assert_eq!(header.desc.0, "Positional arguments");

        let header = parse_entry_header("**kwargs (dict): Keyword arguments");
        assert_eq!(header.name, "**kwargs");
        let (ts, _, _) = header.type_info.unwrap();
        assert_eq!(ts, "dict");
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
    }

    // -- full parser --

    #[test]
    fn test_parse_simple_summary() {
        let result = parse_google("Brief description.").value;
        assert_eq!(result.summary.value, "Brief description.");
    }

    #[test]
    fn test_parse_empty() {
        let result = parse_google("").value;
        assert_eq!(result.summary.value, "");
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_google("   \n   \n").value;
        assert_eq!(result.summary.value, "");
    }

    #[test]
    fn test_parse_summary_with_description() {
        let input = "Brief summary.\n\nExtended description.\nMore text.";
        let result = parse_google(input).value;
        assert_eq!(result.summary.value, "Brief summary.");
        assert_eq!(
            result.description.as_ref().unwrap().value,
            "Extended description.\nMore text."
        );
    }

    #[test]
    fn test_parse_args() {
        let input = "Summary.\n\nArgs:\n    x (int): The value.\n    y (str): The name.";
        let result = parse_google(input).value;
        assert_eq!(result.args().len(), 2);
        assert_eq!(result.args()[0].name.value, "x");
        assert_eq!(result.args()[0].arg_type.as_ref().unwrap().value, "int");
        assert_eq!(result.args()[0].description.value, "The value.");
        assert_eq!(result.args()[1].name.value, "y");
    }

    #[test]
    fn test_parse_args_multiline_desc() {
        let input = "Summary.\n\nArgs:\n    x (int): First line.\n        Second line.";
        let result = parse_google(input).value;
        assert_eq!(
            result.args()[0].description.value,
            "First line.\nSecond line."
        );
    }

    #[test]
    fn test_parse_returns() {
        let input = "Summary.\n\nReturns:\n    int: The result.";
        let result = parse_google(input).value;
        assert_eq!(result.returns().len(), 1);
        assert_eq!(
            result.returns()[0].return_type.as_ref().unwrap().value,
            "int"
        );
        assert_eq!(result.returns()[0].description.value, "The result.");
    }

    #[test]
    fn test_parse_returns_multiple() {
        let input = "Summary.\n\nReturns:\n    int: The count.\n    str: The message.";
        let result = parse_google(input).value;
        assert_eq!(result.returns().len(), 2);
    }

    #[test]
    fn test_parse_raises() {
        let input = "Summary.\n\nRaises:\n    ValueError: If invalid.";
        let result = parse_google(input).value;
        assert_eq!(result.raises().len(), 1);
        assert_eq!(result.raises()[0].exception_type.value, "ValueError");
        assert_eq!(result.raises()[0].description.value, "If invalid.");
    }

    #[test]
    fn test_parse_span_accuracy() {
        let input = "Summary line.";
        let result = parse_google(input).value;
        assert_eq!(result.summary.span.start.line, 0);
        assert_eq!(result.summary.span.start.column, 0);
        assert_eq!(result.summary.span.end.column, 13);
        assert_eq!(
            result.summary.span.source_text(&result.source),
            "Summary line."
        );
    }
}
