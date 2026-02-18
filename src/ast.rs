//! Core AST types, source location primitives, traits, and shared utilities.
//!
//! This module provides:
//! - [`Span`], [`Position`], [`Spanned`] — source location tracking
//! - [`ParameterView`], [`ReturnsView`], [`ExceptionView`], [`AttributeView`] — style-agnostic view types
//! - [`DocstringLike`] — unified trait for accessing docstring elements
//! - [`Docstring`], [`Style`] — unified docstring types
//! - Common span-construction and indentation utilities used by parsers

use core::fmt;

use crate::google::GoogleDocstring;
use crate::numpy::NumPyDocstring;
use crate::sphinx::SphinxDocstring;

// =============================================================================
// Source location types
// =============================================================================

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
/// use pydocstring::ast::{Span, Position, Spanned};
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

// =============================================================================
// View types (style-agnostic borrowed access)
// =============================================================================

/// A borrowed view of a parameter (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterView<'a> {
    /// Parameter name with its span.
    pub name: Spanned<&'a str>,
    /// Parameter type annotation with its span.
    pub param_type: Option<Spanned<&'a str>>,
    /// Parameter description with its span.
    pub description: Spanned<&'a str>,
    /// Source span of the `optional` marker, if present.
    /// `None` means not marked as optional, `Some(span)` gives the location of `optional` text.
    pub optional: Option<Span>,
    /// Source span of the entire parameter definition.
    pub span: Span,
}

/// A borrowed view of a return value (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnsView<'a> {
    /// Return value name (if named) with its span.
    pub name: Option<Spanned<&'a str>>,
    /// Return type annotation with its span.
    pub return_type: Option<Spanned<&'a str>>,
    /// Description of the return value with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an exception (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionView<'a> {
    /// Exception type name with its span.
    pub exception_type: Spanned<&'a str>,
    /// Description of when the exception is raised, with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an attribute (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeView<'a> {
    /// Attribute name with its span.
    pub name: Spanned<&'a str>,
    /// Attribute type annotation with its span.
    pub attr_type: Option<Spanned<&'a str>>,
    /// Description with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

// =============================================================================
// Trait
// =============================================================================

/// A parsed docstring of any style.
///
/// This trait abstracts over `NumPyDocstring`, `GoogleDocstring`, and `SphinxDocstring`,
/// providing zero-cost access to common docstring elements.
///
/// # Example
///
/// ```rust
/// use pydocstring::ast::DocstringLike;
///
/// fn check_params_documented(doc: &impl DocstringLike) -> Vec<String> {
///     doc.parameters()
///         .iter()
///         .filter(|p| p.description.value.is_empty())
///         .map(|p| p.name.value.to_string())
///         .collect()
/// }
/// ```
pub trait DocstringLike {
    /// Returns the brief summary line.
    fn summary(&self) -> &str;

    /// Returns the extended description, if any.
    fn description(&self) -> Option<&str>;

    /// Returns parameters as style-agnostic views.
    fn parameters(&self) -> Vec<ParameterView<'_>>;

    /// Returns return values as style-agnostic views.
    fn returns(&self) -> Vec<ReturnsView<'_>>;

    /// Returns exceptions as style-agnostic views.
    fn raises(&self) -> Vec<ExceptionView<'_>>;

    /// Returns attributes as style-agnostic views.
    fn attributes(&self) -> Vec<AttributeView<'_>>;
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

/// Create a [`Span`] from start/end line and column pairs.
pub(crate) fn make_span(
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    offsets: &[usize],
) -> Span {
    Span::new(
        Position::new(
            start_line as u32,
            start_col as u32,
            (offsets[start_line] + start_col) as u32,
        ),
        Position::new(
            end_line as u32,
            end_col as u32,
            (offsets[end_line] + end_col) as u32,
        ),
    )
}

/// Create a [`Spanned<String>`] with a computed span.
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
        make_span(start_line, start_col, end_line, end_col, offsets),
    )
}

/// Number of leading whitespace bytes in `line`.
pub(crate) fn indent_len(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// =============================================================================
// Unified types
// =============================================================================

/// Docstring style identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// NumPy style (section headers with underlines).
    NumPy,
    /// Google style (section headers with colons).
    Google,
    /// Sphinx style (field lists with `:param:`, `:type:`, etc.).
    Sphinx,
}

/// A parsed docstring of any style.
///
/// Wraps the style-specific types and implements [`DocstringLike`] for
/// unified access. Use pattern matching to access style-specific fields.
///
/// # Example
///
/// ```rust
/// use pydocstring::{parse, Docstring, DocstringLike};
///
/// let doc = parse("Brief summary.").unwrap();
/// assert_eq!(doc.summary(), "Brief summary.");
/// assert_eq!(doc.style(), pydocstring::Style::Google);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Docstring {
    /// NumPy-style docstring.
    NumPy(NumPyDocstring),
    /// Google-style docstring.
    Google(GoogleDocstring),
    /// Sphinx-style docstring.
    Sphinx(SphinxDocstring),
}

impl Docstring {
    /// Returns the detected style.
    pub fn style(&self) -> Style {
        match self {
            Docstring::NumPy(_) => Style::NumPy,
            Docstring::Google(_) => Style::Google,
            Docstring::Sphinx(_) => Style::Sphinx,
        }
    }

    /// Returns a reference to the inner `NumPyDocstring`, if this is NumPy style.
    pub fn as_numpy(&self) -> Option<&NumPyDocstring> {
        match self {
            Docstring::NumPy(d) => Some(d),
            _ => None,
        }
    }

    /// Returns a reference to the inner `GoogleDocstring`, if this is Google style.
    pub fn as_google(&self) -> Option<&GoogleDocstring> {
        match self {
            Docstring::Google(d) => Some(d),
            _ => None,
        }
    }

    /// Returns a reference to the inner `SphinxDocstring`, if this is Sphinx style.
    pub fn as_sphinx(&self) -> Option<&SphinxDocstring> {
        match self {
            Docstring::Sphinx(d) => Some(d),
            _ => None,
        }
    }
}

impl DocstringLike for Docstring {
    fn summary(&self) -> &str {
        match self {
            Docstring::NumPy(d) => d.summary(),
            Docstring::Google(d) => d.summary(),
            Docstring::Sphinx(d) => d.summary(),
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Docstring::NumPy(d) => d.description(),
            Docstring::Google(d) => d.description(),
            Docstring::Sphinx(d) => d.description(),
        }
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::parameters(d),
            Docstring::Google(d) => d.parameters(),
            Docstring::Sphinx(d) => d.parameters(),
        }
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::returns(d),
            Docstring::Google(d) => d.returns(),
            Docstring::Sphinx(d) => d.returns(),
        }
    }

    fn raises(&self) -> Vec<ExceptionView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::raises(d),
            Docstring::Google(d) => d.raises(),
            Docstring::Sphinx(d) => d.raises(),
        }
    }

    fn attributes(&self) -> Vec<AttributeView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::attributes(d),
            Docstring::Google(d) => d.attributes(),
            Docstring::Sphinx(d) => d.attributes(),
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::NumPy => write!(f, "numpy"),
            Style::Google => write!(f, "google"),
            Style::Sphinx => write!(f, "sphinx"),
        }
    }
}
