//! Core AST types, source location primitives, and shared utilities.
//!
//! This module provides:
//! - [`TextSize`], [`TextRange`], [`Spanned`] — source location tracking (ruff-style, offset-only)
//! - [`LineIndex`] — line/column computation from byte offsets
//! - [`Style`] — docstring style identifier
//! - Common utilities used by parsers

use core::fmt;
use core::ops;

// =============================================================================
// Source location types (ruff-style, offset-only)
// =============================================================================

/// A byte offset in the source text.
///
/// Newtype over `u32` for type safety (prevents mixing with line numbers, etc.).
/// Inspired by ruff's `TextSize` (from the `text-size` crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TextSize(u32);

impl TextSize {
    /// Creates a new text size from a raw byte offset.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw byte offset.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for TextSize {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<TextSize> for u32 {
    fn from(size: TextSize) -> Self {
        size.0
    }
}

impl From<TextSize> for usize {
    fn from(size: TextSize) -> Self {
        size.0 as usize
    }
}

impl From<usize> for TextSize {
    fn from(raw: usize) -> Self {
        Self(raw as u32)
    }
}

impl ops::Add for TextSize {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl ops::Sub for TextSize {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for TextSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A range in the source text `[start, end)`, represented as byte offsets.
///
/// Stores only offsets — line/column information is computed on demand
/// via [`LineIndex`]. Inspired by ruff's `TextRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    /// Creates a new range from start (inclusive) and end (exclusive) offsets.
    pub const fn new(start: TextSize, end: TextSize) -> Self {
        Self { start, end }
    }

    /// Creates an empty range at offset 0.
    pub const fn empty() -> Self {
        Self {
            start: TextSize::new(0),
            end: TextSize::new(0),
        }
    }

    /// Start offset (inclusive).
    pub const fn start(self) -> TextSize {
        self.start
    }

    /// End offset (exclusive).
    pub const fn end(self) -> TextSize {
        self.end
    }

    /// Length of the range in bytes.
    pub const fn len(self) -> TextSize {
        TextSize::new(self.end.0 - self.start.0)
    }

    /// Whether the range is empty.
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }

    /// Whether `offset` is contained in this range.
    pub const fn contains(self, offset: TextSize) -> bool {
        self.start.0 <= offset.0 && offset.0 < self.end.0
    }

    /// Extracts the corresponding slice from the source text.
    ///
    /// Returns an empty string if the range is empty or offsets are out of bounds.
    pub fn source_text<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start.0 as usize;
        let end = self.end.0 as usize;
        if start <= end && end <= source.len() {
            &source[start..end]
        } else {
            ""
        }
    }
}

// =============================================================================
// LineIndex — on-demand line/column computation
// =============================================================================

/// Index for mapping byte offsets to line/column positions.
///
/// Built once per source text, then queried as needed (e.g. for error display).
///
/// ```rust
/// use pydocstring::ast::{LineIndex, TextSize};
///
/// let source = "first\nsecond\nthird";
/// let index = LineIndex::from_source(source);
/// let (line, col) = index.line_col(TextSize::new(6));
/// assert_eq!(line, 1); // 0-indexed: second line
/// assert_eq!(col, 0);  // start of line
/// ```
pub struct LineIndex {
    /// Byte offset of each line start.
    line_starts: Vec<TextSize>,
}

impl LineIndex {
    /// Build a line index from source text.
    pub fn from_source(source: &str) -> Self {
        let mut line_starts = vec![TextSize::new(0)];
        for (i, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(TextSize::new((i + 1) as u32));
            }
        }
        Self { line_starts }
    }

    /// Returns 0-indexed (line, column) for a byte offset.
    ///
    /// Column is the byte offset from the start of the line.
    pub fn line_col(&self, offset: TextSize) -> (u32, u32) {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let col = offset.raw() - self.line_starts[line].raw();
        (line as u32, col)
    }

    /// Returns the 0-indexed line number for a byte offset.
    pub fn line(&self, offset: TextSize) -> u32 {
        self.line_col(offset).0
    }

    /// Returns the byte offset of the start of a given line.
    pub fn line_start(&self, line: u32) -> TextSize {
        self.line_starts
            .get(line as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Number of lines in the source.
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }
}

// =============================================================================
// Spanned
// =============================================================================

/// A value annotated with source location information.
///
/// Used to track the precise location of each semantic element in the docstring,
/// enabling linters to report errors at specific positions (e.g., a parameter name,
/// its type annotation, or its description individually).
///
/// # Example
///
/// ```rust
/// use pydocstring::ast::{TextRange, TextSize, Spanned};
///
/// let name = Spanned::new(
///     "x".to_string(),
///     TextRange::new(TextSize::new(30), TextSize::new(31)),
/// );
/// assert_eq!(name.value, "x");
/// assert_eq!(name.range.start().raw(), 30);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The value.
    pub value: T,
    /// Source range of this value.
    pub range: TextRange,
}

impl<T> Spanned<T> {
    /// Creates a new spanned value.
    pub fn new(value: T, range: TextRange) -> Self {
        Self { value, range }
    }

    /// Creates a spanned value with an empty range.
    ///
    /// Useful as a placeholder during construction or when
    /// range information is not yet available.
    pub fn dummy(value: T) -> Self {
        Self {
            value,
            range: TextRange::empty(),
        }
    }

    /// Unwraps the spanned value, discarding the range.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl Spanned<String> {
    /// Creates an empty spanned string with an empty range.
    pub fn empty_string() -> Self {
        Self {
            value: String::new(),
            range: TextRange::empty(),
        }
    }

    /// Borrows as a `Spanned<&str>`, preserving the range.
    pub fn as_spanned_str(&self) -> Spanned<&str> {
        Spanned {
            value: self.value.as_str(),
            range: self.range,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

// =============================================================================
// Shared utilities (used by style-specific parsers)
// =============================================================================

/// Build a table mapping each line index to its starting byte offset in the source.
pub(crate) fn build_line_offsets(input: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, byte) in input.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Create a [`TextRange`] from start/end line and column pairs.
pub(crate) fn make_range(
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    offsets: &[usize],
) -> TextRange {
    TextRange::new(
        TextSize::new((offsets[start_line] + start_col) as u32),
        TextSize::new((offsets[end_line] + end_col) as u32),
    )
}

/// Create a [`TextRange`] directly from byte offsets.
pub(crate) fn make_range_raw(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32))
}

/// Create a [`Spanned<String>`] with a computed range.
pub(crate) fn make_spanned(
    value: String,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    offsets: &[usize],
) -> Spanned<String> {
    Spanned::new(
        value,
        make_range(start_line, start_col, end_line, end_col, offsets),
    )
}

/// Resolve a byte offset back to (line, col) using the line-offset table.
///
/// Used internally when a parser needs to read back line/col from a
/// previously-created [`TextRange`].
pub(crate) fn offset_to_line_col(offset: usize, offsets: &[usize]) -> (usize, usize) {
    let line = offsets.partition_point(|&o| o <= offset).saturating_sub(1);
    let col = offset - offsets[line];
    (line, col)
}

/// Number of leading whitespace bytes in `line`.
pub(crate) fn indent_len(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// =============================================================================
// Style
// =============================================================================

/// Docstring style identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// NumPy style (section headers with underlines).
    NumPy,
    /// Google style (section headers with colons).
    Google,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::NumPy => write!(f, "numpy"),
            Style::Google => write!(f, "google"),
        }
    }
}
