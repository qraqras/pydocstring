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
    Spanned, build_line_offsets, indent_len, make_range, make_spanned, offset_to_line_col,
};
use crate::styles::google::ast::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleException, GoogleMethod, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader, GoogleSeeAlsoItem, GoogleWarning,
};

// =============================================================================
// Section detection
// =============================================================================

/// Known Google-style section names (lowercase, without colon).
///
/// Used to recognise colonless headers like `Args` in addition to the
/// standard `Args:` form.
const KNOWN_SECTIONS: &[&str] = &[
    "args",
    "arguments",
    "parameters",
    "params",
    "keyword args",
    "keyword arguments",
    "other parameters",
    "receive",
    "receives",
    "returns",
    "return",
    "yields",
    "yield",
    "raises",
    "raise",
    "warns",
    "warn",
    "attributes",
    "attribute",
    "methods",
    "see also",
    "note",
    "notes",
    "example",
    "examples",
    "todo",
    "references",
    "warning",
    "warnings",
    "attention",
    "caution",
    "danger",
    "error",
    "hint",
    "important",
    "tip",
];

/// Check if a lowercased, trimmed name matches a known section name.
fn is_known_section_name(name: &str) -> bool {
    KNOWN_SECTIONS.contains(&name)
}

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
        is_known_section_name(&name.to_ascii_lowercase())
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
    if let Some(name) = text.strip_suffix(':') {
        return EntryHeader {
            name,
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
    let lines: Vec<&str> = input.lines().collect();
    let mut docstring = GoogleDocstring::new();
    docstring.source = input.to_string();

    if lines.is_empty() {
        return docstring;
    }

    let mut i = 0;

    // --- Skip leading blank lines ---
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    if i >= lines.len() {
        return docstring;
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
            let (body, next_i) = match normalized.as_str() {
                // ----- Parameter-like sections -----
                "args" | "arguments" | "parameters" | "params" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Args(args), ni)
                }
                "keyword args" | "keyword arguments" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::KeywordArgs(args), ni)
                }
                "other parameters" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::OtherParameters(args), ni)
                }
                "receive" | "receives" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Receives(args), ni)
                }
                // ----- Return/yield sections -----
                "returns" | "return" => {
                    let (returns, ni) = parse_returns_section(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Returns(returns), ni)
                }
                "yields" | "yield" => {
                    let (yields, ni) = parse_returns_section(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Yields(yields), ni)
                }
                // ----- Exception/warning sections -----
                "raises" | "raise" => {
                    let (raises, ni) = parse_raises_section(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Raises(raises), ni)
                }
                "warns" | "warn" => {
                    let (raises, ni) = parse_raises_section(&lines, i, &offsets, base_indent);
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
                "attributes" | "attribute" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
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
                "methods" => {
                    let (args, ni) = parse_args(&lines, i, &offsets, base_indent);
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
                "see also" => {
                    let (items, ni) = parse_see_also_section(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::SeeAlso(items), ni)
                }
                // ----- Free-text / admonition sections -----
                "note" | "notes" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Notes(content), ni)
                }
                "example" | "examples" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Examples(content), ni)
                }
                "todo" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Todo(content), ni)
                }
                "references" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::References(content), ni)
                }
                "warning" | "warnings" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Warnings(content), ni)
                }
                "attention" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Attention(content), ni)
                }
                "caution" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Caution(content), ni)
                }
                "danger" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Danger(content), ni)
                }
                "error" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Error(content), ni)
                }
                "hint" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Hint(content), ni)
                }
                "important" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Important(content), ni)
                }
                "tip" => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Tip(content), ni)
                }
                _ => {
                    let (content, ni) = parse_section_content(&lines, i, &offsets, base_indent);
                    (GoogleSectionBody::Unknown(content), ni)
                }
            };

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
    let last_line = lines.len().saturating_sub(1);
    let last_col = lines.last().map(|l| l.len()).unwrap_or(0);
    docstring.range = make_range(0, 0, last_line, last_col, &offsets);

    docstring
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
) -> (Vec<GoogleArg>, usize) {
    let mut args = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

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

        let ei = *entry_indent.get_or_insert(indent);

        // Entry line at entry indent level
        if indent <= ei {
            let col = indent;
            let entry_start = i;

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
                        make_spanned(
                            "optional".to_string(),
                            i,
                            oc,
                            i,
                            oc + "optional".len(),
                            offsets,
                        )
                    });
                    (arg_t, opt)
                }
                None => (None, None),
            };

            // Description: inline fragment + continuation lines
            let (first_desc, desc_off) = header.desc;
            let first_desc_col = col + desc_off;

            i += 1;
            let (cont_desc, next_i) = collect_description(lines, i, offsets, ei, base_indent);
            let full_desc = build_full_description(
                first_desc,
                first_desc_col,
                entry_start,
                &cont_desc,
                offsets,
            );

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                {
                    let (el, ec) =
                        offset_to_line_col(full_desc.range.end().raw() as usize, offsets);
                    (el, ec)
                }
            };

            args.push(GoogleArg {
                range: make_range(entry_start, col, end_line, end_col, offsets),
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> (Vec<GoogleReturns>, usize) {
    let mut returns = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

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
            let (cont_desc, next_i) = collect_description(lines, i, offsets, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                {
                    let (el, ec) =
                        offset_to_line_col(full_desc.range.end().raw() as usize, offsets);
                    (el, ec)
                }
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> (Vec<GoogleException>, usize) {
    let mut raises = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

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

        let ei = *entry_indent.get_or_insert(indent);

        if indent <= ei {
            let col = indent;
            let entry_start = i;

            let (exc_type_str, first_desc, desc_col) = if let Some(colon_pos) = trimmed.find(": ") {
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
            let (cont_desc, next_i) = collect_description(lines, i, offsets, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let (end_line, end_col) = if full_desc.value.is_empty() {
                (entry_start, col + trimmed.len())
            } else {
                {
                    let (el, ec) =
                        offset_to_line_col(full_desc.range.end().raw() as usize, offsets);
                    (el, ec)
                }
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> (Spanned<String>, usize) {
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
    lines: &[&str],
    start: usize,
    offsets: &[usize],
    base_indent: usize,
) -> (Vec<GoogleSeeAlsoItem>, usize) {
    let mut items = Vec::new();
    let mut i = start;
    let mut entry_indent: Option<usize> = None;

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
            let (cont_desc, next_i) = collect_description(lines, i, offsets, ei, base_indent);
            let full_desc =
                build_full_description(first_desc, desc_col, entry_start, &cont_desc, offsets);

            let description = if full_desc.value.is_empty() {
                None
            } else {
                Some(full_desc)
            };

            let (end_line, end_col) = if let Some(ref d) = description {
                {
                    let (el, ec) = offset_to_line_col(d.range.end().raw() as usize, offsets);
                    (el, ec)
                }
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
