//! Source cursor for line-oriented docstring parsing.
//!
//! [`Cursor`] bundles the source text, line-offset table, and current
//! line position into a single struct, eliminating the need to thread
//! `(source, &offsets, total_lines)` through every helper function.

use crate::ast::{Spanned, TextRange, TextSize};

// =============================================================================
// Cursor
// =============================================================================

/// A read/write cursor over a source string, providing line-oriented
/// navigation and span construction helpers.
///
/// Callers advance the cursor by mutating [`Cursor::line`] directly
/// (or via convenience methods like [`advance`](Cursor::advance) and
/// [`skip_blank_lines`](Cursor::skip_blank_lines)).  Sub-parsers
/// receive `&mut Cursor` and leave it positioned after the last
/// consumed line.
pub(crate) struct Cursor<'a> {
    source: &'a str,
    offsets: Vec<usize>,
    total: usize,
    /// Current line index (0-based).
    pub line: usize,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor over `source`, starting at line 0.
    pub fn new(source: &'a str) -> Self {
        let offsets = build_line_offsets(source);
        let total = count_lines(source, &offsets);
        Self {
            source,
            offsets,
            total,
            line: 0,
        }
    }

    // ── Source access ────────────────────────────────────────────────

    /// The full source text.
    pub fn source(&self) -> &'a str {
        self.source
    }

    // ── Position ────────────────────────────────────────────────────

    /// Whether the cursor has reached or passed the end of the source.
    pub fn is_eof(&self) -> bool {
        self.line >= self.total
    }

    /// Total number of lines in the source.
    pub fn total_lines(&self) -> usize {
        self.total
    }

    /// Advance the cursor by one line.
    pub fn advance(&mut self) {
        self.line += 1;
    }

    /// Skip blank (whitespace-only) lines starting at the current position.
    pub fn skip_blank_lines(&mut self) {
        while !self.is_eof() && self.current_line_text().trim().is_empty() {
            self.line += 1;
        }
    }

    // ── Current-line helpers ────────────────────────────────────────

    /// Text of the current line (without trailing newline).
    pub fn current_line_text(&self) -> &'a str {
        self.line_text(self.line)
    }

    /// Trimmed text of the current line.
    pub fn current_trimmed(&self) -> &'a str {
        self.current_line_text().trim()
    }

    /// Leading-whitespace byte count of the current line.
    pub fn current_indent(&self) -> usize {
        indent_len(self.current_line_text())
    }

    /// Whether the current line is blank (empty or whitespace-only).
    pub fn current_is_blank(&self) -> bool {
        self.current_line_text().trim().is_empty()
    }

    // ── Arbitrary-line helpers ──────────────────────────────────────

    /// Text of line `idx` (without trailing newline).
    ///
    /// Returns `""` if `idx` is out of bounds.
    pub fn line_text(&self, idx: usize) -> &'a str {
        if idx >= self.offsets.len() {
            return "";
        }
        let start = self.offsets[idx];
        let end = if idx + 1 < self.offsets.len() {
            self.offsets[idx + 1].saturating_sub(1)
        } else {
            self.source.len()
        };
        if start >= self.source.len() {
            return "";
        }
        &self.source[start..end]
    }

    /// Leading-whitespace byte count of line `idx`.
    #[allow(dead_code)]
    pub fn line_indent(&self, idx: usize) -> usize {
        indent_len(self.line_text(idx))
    }

    // ── Span construction ──────────────────────────────────────────

    /// Build a [`TextRange`] from (line, col) pairs.
    pub fn make_range(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> TextRange {
        TextRange::new(
            TextSize::new((self.offsets[start_line] + start_col) as u32),
            TextSize::new((self.offsets[end_line] + end_col) as u32),
        )
    }

    /// Build a [`Spanned<String>`] from (line, col) pairs.
    pub fn make_spanned(
        &self,
        value: String,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Spanned<String> {
        Spanned::new(
            value,
            self.make_range(start_line, start_col, end_line, end_col),
        )
    }

    // ── Offset utilities ───────────────────────────────────────────

    /// Convert a byte offset to `(line, col)`.
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let line = self
            .offsets
            .partition_point(|&o| o <= offset)
            .saturating_sub(1);
        let col = offset - self.offsets[line];
        (line, col)
    }

    /// Byte offset of `inner` within the source string.
    ///
    /// Both `inner` and the source must point into the same allocation.
    pub fn substr_offset(&self, inner: &str) -> usize {
        inner.as_ptr() as usize - self.source.as_ptr() as usize
    }

    /// Find the matching closing bracket for an opening bracket at `open_pos`.
    ///
    /// Handles nested `()`, `[]`, `{}`.
    pub fn find_matching_close(&self, open_pos: usize) -> Option<usize> {
        let mut depth = 0;
        for (i, c) in self.source[open_pos..].char_indices() {
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
}

// =============================================================================
// Standalone helpers (still useful outside Cursor)
// =============================================================================

/// Number of leading whitespace bytes in `line`.
pub(crate) fn indent_len(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Build a table mapping each line index to its starting byte offset.
fn build_line_offsets(input: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, byte) in input.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Number of text lines in source.
fn count_lines(source: &str, offsets: &[usize]) -> usize {
    if source.is_empty() {
        0
    } else if source.ends_with('\n') {
        offsets.len() - 1
    } else {
        offsets.len()
    }
}
