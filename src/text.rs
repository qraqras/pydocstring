//! Source location types (offset-only).
//!
//! This module provides [`TextSize`] (a byte offset) and [`TextRange`]
//! (a half-open byte range) for tracking source positions.
//! Inspired by ruff / rust-analyzer's `text-size` crate.

use core::fmt;
use core::ops;

// =============================================================================
// TextSize
// =============================================================================

/// A byte offset in the source text.
///
/// Newtype over `u32` for type safety (prevents mixing with line numbers, etc.).
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

// =============================================================================
// TextRange
// =============================================================================

/// A range in the source text `[start, end)`, represented as byte offsets.
///
/// Stores only offsets.
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

    /// Creates a range from an absolute byte offset and a length.
    pub const fn from_offset_len(offset: usize, len: usize) -> Self {
        Self {
            start: TextSize::new(offset as u32),
            end: TextSize::new((offset + len) as u32),
        }
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

    /// Extend this range's end to include `other`.
    pub fn extend(&mut self, other: TextRange) {
        self.end = other.end;
    }
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
