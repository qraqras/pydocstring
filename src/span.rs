//! Source location types for tracking positions in docstrings.
//!
//! These types are used by linters and formatters to report
//! precise locations of issues within docstrings.

use core::fmt;

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
    /// Byte offset from the start of the source text.
    pub offset: u32,
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

    /// Extracts the corresponding slice from the source text.
    ///
    /// Returns an empty string if the span is empty or offsets are out of range.
    pub fn source_text<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start.offset as usize;
        let end = self.end.offset as usize;
        if start <= end && end <= source.len() {
            &source[start..end]
        } else {
            ""
        }
    }
}

impl Position {
    /// Creates a new position.
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Creates a position at the origin (0, 0, 0).
    pub fn zero() -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

/// A value annotated with source location information.
///
/// Used to track the precise location of each semantic element in the docstring,
/// enabling linters to report errors at specific positions (e.g., a parameter name,
/// its type annotation, or its description individually).
///
/// # Example
///
/// ```rust
/// use pydocstring::span::{Span, Position, Spanned};
///
/// let name = Spanned::new(
///     "x".to_string(),
///     Span::new(Position::new(3, 0, 30), Position::new(3, 1, 31)),
/// );
/// assert_eq!(name.value, "x");
/// assert_eq!(name.span.start.line, 3);
/// assert_eq!(name.span.start.offset, 30);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The value.
    pub value: T,
    /// Source span of this value.
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Creates a new spanned value.
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Creates a spanned value with an empty span.
    ///
    /// Useful as a placeholder during construction or when
    /// span information is not yet available.
    pub fn dummy(value: T) -> Self {
        Self {
            value,
            span: Span::empty(),
        }
    }

    /// Unwraps the spanned value, discarding the span.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl Spanned<String> {
    /// Creates an empty spanned string with an empty span.
    pub fn empty_string() -> Self {
        Self {
            value: String::new(),
            span: Span::empty(),
        }
    }

    /// Borrows as a `Spanned<&str>`, preserving the span.
    pub fn as_spanned_str(&self) -> Spanned<&str> {
        Spanned {
            value: self.value.as_str(),
            span: self.span,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
