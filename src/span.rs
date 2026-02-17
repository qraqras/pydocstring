//! Source location types for tracking positions in docstrings.
//!
//! These types are used by linters and formatters to report
//! precise locations of issues within docstrings.

/// A range in the source text, represented as byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start position (inclusive).
    pub start: Position,
    /// End position (exclusive).
    pub end: Position,
}

/// A position in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// 0-indexed line number.
    pub line: u32,
    /// 0-indexed column (UTF-8 byte offset from line start).
    pub column: u32,
}

impl Span {
    /// Creates a new span from start and end positions.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates an empty span at the origin (0, 0).
    pub fn empty() -> Self {
        Self {
            start: Position::zero(),
            end: Position::zero(),
        }
    }
}

impl Position {
    /// Creates a new position.
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// Creates a position at the origin (0, 0).
    pub fn zero() -> Self {
        Self { line: 0, column: 0 }
    }
}
