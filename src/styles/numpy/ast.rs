use core::fmt;

use crate::ast::{Spanned, TextRange};

// =============================================================================
// NumPy Style Types
// =============================================================================

/// A single NumPy-style section, combining header and body.
///
/// ```text
/// Parameters       <-- header
/// ----------       <-- header
/// x : int          <-- body
///     Description  <-- body
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NumPySection {
    /// Source span of the entire section (header + body).
    pub range: TextRange,
    /// Section header (name + underline).
    pub header: NumPySectionHeader,
    /// Section body content.
    pub body: NumPySectionBody,
}

/// NumPy-style section header.
///
/// Represents a parsed section header like:
/// ```text
/// Parameters     <-- name
/// ----------     <-- underline
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NumPySectionHeader {
    /// Source span of the entire header (name line + underline line).
    pub range: TextRange,
    /// Section name (e.g., "Parameters", "Returns") with its span.
    pub name: Spanned<String>,
    /// Underline (dashes) line with its span.
    pub underline: Spanned<String>,
}

/// Body content of a NumPy-style section.
///
/// Each variant corresponds to a specific section kind.
#[derive(Debug, Clone, PartialEq)]
pub enum NumPySectionBody {
    /// Parameters section.
    Parameters(Vec<NumPyParameter>),
    /// Returns section.
    Returns(Vec<NumPyReturns>),
    /// Yields section.
    Yields(Vec<NumPyReturns>),
    /// Receives section.
    Receives(Vec<NumPyParameter>),
    /// Other Parameters section.
    OtherParameters(Vec<NumPyParameter>),
    /// Raises section.
    Raises(Vec<NumPyException>),
    /// Warns section.
    Warns(Vec<NumPyWarning>),
    /// Warnings section (free text).
    Warnings(Spanned<String>),
    /// See Also section.
    SeeAlso(Vec<SeeAlsoItem>),
    /// Notes section (free text).
    Notes(Spanned<String>),
    /// References section.
    References(Vec<NumPyReference>),
    /// Examples section (free text, doctest format).
    Examples(Spanned<String>),
    /// Attributes section.
    Attributes(Vec<NumPyAttribute>),
    /// Methods section.
    Methods(Vec<NumPyMethod>),
    /// Unknown / unrecognized section (free text).
    Unknown(Spanned<String>),
}

/// NumPy-style docstring.
///
/// Supports sections with underlines like:
/// ```text
/// Parameters
/// ----------
/// name : type
///     Description
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyDocstring {
    /// Original source text of the docstring.
    pub source: String,
    /// Source span of the entire docstring.
    pub range: TextRange,
    /// Brief summary (first line).
    pub summary: Spanned<String>,
    /// Deprecation warning (if applicable).
    pub deprecation: Option<NumPyDeprecation>,
    /// Extended summary (multiple sentences before any section header).
    /// Clarifies functionality, may reference parameters.
    pub extended_summary: Option<Spanned<String>>,
    /// All sections in order of appearance.
    pub sections: Vec<NumPySection>,
}

/// NumPy-style deprecation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyDeprecation {
    /// Source span.
    pub range: TextRange,
    /// Version when deprecated (e.g., "1.6.0") with its span.
    pub version: Spanned<String>,
    /// Reason for deprecation and recommendation (free text body), with its span.
    pub description: Spanned<String>,
}

/// NumPy-style parameter.
///
/// Can represent a single parameter or multiple parameters with the same type:
/// `x : int` or `x1, x2 : array_like`
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyParameter {
    /// Source span of this parameter definition.
    pub range: TextRange,
    /// Parameter names (supports multiple names like `x1, x2`), each with its own span.
    pub names: Vec<Spanned<String>>,
    /// Parameter type (e.g., "int", "str", "array_like") with its span.
    /// Type is optional for parameters but required for returns.
    pub r#type: Option<Spanned<String>>,
    /// Parameter description with its span.
    pub description: Spanned<String>,
    /// The `optional` marker, if present.
    /// `None` means not marked as optional.
    pub optional: Option<Spanned<String>>,
    /// Default value (e.g., "True", "-1", "None") with its span.
    pub default: Option<Spanned<String>>,
}

/// NumPy-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReturns {
    /// Source span.
    pub range: TextRange,
    /// Return value name (optional in NumPy style) with its span.
    pub name: Option<Spanned<String>>,
    /// Return type with its span.
    pub return_type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// NumPy-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyException {
    /// Source span.
    pub range: TextRange,
    /// Exception type with its span.
    pub r#type: Spanned<String>,
    /// Description of when raised, with its span.
    pub description: Spanned<String>,
}

/// NumPy-style warning (from Warns section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyWarning {
    /// Source span.
    pub range: TextRange,
    /// Warning type (e.g., "DeprecationWarning") with its span.
    pub r#type: Spanned<String>,
    /// When the warning is issued, with its span.
    pub description: Spanned<String>,
}

/// See Also item.
///
/// Supports multiple items and optional descriptions:
/// - `func_a : Description`
/// - `func_b, func_c` (multiple names, no description)
#[derive(Debug, Clone, PartialEq)]
pub struct SeeAlsoItem {
    /// Source span.
    pub range: TextRange,
    /// Reference names (can be multiple like `func_b, func_c`), each with its own span.
    pub names: Vec<Spanned<String>>,
    /// Optional description with its span.
    pub description: Option<Spanned<String>>,
}

/// Numbered reference (from References section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReference {
    /// Source span.
    pub range: TextRange,
    /// Reference number (e.g., "1", "2", "3") with its span.
    pub number: Spanned<String>,
    /// Reference content (author, title, etc) with its span.
    pub content: Spanned<String>,
}

/// NumPy-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyAttribute {
    /// Source span.
    pub range: TextRange,
    /// Attribute name with its span.
    pub name: Spanned<String>,
    /// Attribute type with its span.
    pub r#type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// NumPy-style method (for classes).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyMethod {
    /// Source span.
    pub range: TextRange,
    /// Method name with its span.
    pub name: Spanned<String>,
    /// Brief description with its span.
    pub description: Spanned<String>,
}

impl NumPyDocstring {
    /// Creates a new empty NumPy-style docstring.
    pub fn new() -> Self {
        Self {
            source: String::new(),
            range: TextRange::empty(),
            summary: Spanned::empty_string(),
            deprecation: None,
            extended_summary: None,
            sections: Vec::new(),
        }
    }
}

impl Default for NumPyDocstring {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NumPyDocstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NumPyDocstring(summary: {})", self.summary.value)
    }
}
