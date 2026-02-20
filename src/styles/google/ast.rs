use core::fmt;

use crate::ast::{Spanned, TextRange};

// =============================================================================
// Google Style Types
// =============================================================================

/// A single Google-style section, combining header and body.
///
/// ```text
/// Parameters:               <-- header
///     x (int): Description  <-- body
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleSection {
    /// Source range of the entire section (header + body).
    pub range: TextRange,
    /// Section header (the `Parameters:` line).
    pub header: GoogleSectionHeader,
    /// Section body content.
    pub body: GoogleSectionBody,
}

/// Google-style section header.
///
/// Represents a parsed section header like `Args:` or `Returns:`.
/// ```text
/// Parameters:  <-- name, colon
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleSectionHeader {
    /// Source range of the header line.
    pub range: TextRange,
    /// Section name (e.g., "Args", "Returns") with its span.
    /// Stored without the trailing colon.
    pub name: Spanned<String>,
    /// The trailing colon (`:`) with its span, if present.
    pub colon: Option<Spanned<String>>,
}

/// Body content of a Google-style section.
///
/// Each variant corresponds to a specific section kind.
/// Section names follow the Napoleon convention.
#[derive(Debug, Clone, PartialEq)]
pub enum GoogleSectionBody {
    // ----- Parameter-like sections -----
    /// Args / Arguments / Parameters / Params section.
    Args(Vec<GoogleArg>),
    /// Keyword Args / Keyword Arguments section.
    KeywordArgs(Vec<GoogleArg>),
    /// Other Parameters section.
    OtherParameters(Vec<GoogleArg>),
    /// Receive / Receives section.
    Receives(Vec<GoogleArg>),

    // ----- Return-like sections -----
    /// Returns / Return section.
    Returns(Vec<GoogleReturns>),
    /// Yields / Yield section.
    Yields(Vec<GoogleReturns>),

    // ----- Exception / warning-like sections -----
    /// Raises / Raise section.
    Raises(Vec<GoogleException>),
    /// Warns / Warn section.
    Warns(Vec<GoogleWarning>),

    // ----- Structured sections -----
    /// See Also section.
    SeeAlso(Vec<GoogleSeeAlsoItem>),
    /// Attributes / Attribute section.
    Attributes(Vec<GoogleAttribute>),
    /// Methods section.
    Methods(Vec<GoogleMethod>),

    // ----- Free-text / admonition sections -----
    /// Note / Notes section (free text).
    Notes(Spanned<String>),
    /// Example / Examples section (free text).
    Examples(Spanned<String>),
    /// Todo section (free text, admonition in Napoleon).
    Todo(Spanned<String>),
    /// References section (free text).
    References(Spanned<String>),
    /// Warning / Warnings section (free text).
    Warnings(Spanned<String>),
    /// Attention admonition (free text).
    Attention(Spanned<String>),
    /// Caution admonition (free text).
    Caution(Spanned<String>),
    /// Danger admonition (free text).
    Danger(Spanned<String>),
    /// Error admonition (free text).
    Error(Spanned<String>),
    /// Hint admonition (free text).
    Hint(Spanned<String>),
    /// Important admonition (free text).
    Important(Spanned<String>),
    /// Tip admonition (free text).
    Tip(Spanned<String>),

    // ----- Fallback -----
    /// Unknown / unrecognized section (free text).
    Unknown(Spanned<String>),
}

/// Google-style docstring.
///
/// Supports sections with colons like:
/// ```text
/// Args:
///     name (type): Description
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleDocstring {
    /// Original source text of the docstring.
    pub source: String,
    /// Source range of the entire docstring.
    pub range: TextRange,
    /// Brief summary (first line).
    pub summary: Spanned<String>,
    /// Extended summary (multiple paragraphs before any section header).
    pub extended_summary: Option<Spanned<String>>,
    /// All sections in order of appearance.
    pub sections: Vec<GoogleSection>,
}

/// Google-style argument.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleArg {
    /// Source range.
    pub range: TextRange,
    /// Argument name with its span.
    pub name: Spanned<String>,
    /// Argument type (inside parentheses) with its span.
    pub r#type: Option<Spanned<String>>,
    /// Argument description with its span.
    pub description: Spanned<String>,
    /// The `optional` marker, if present.
    /// `None` means not marked as optional.
    pub optional: Option<Spanned<String>>,
}

/// Google-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleReturns {
    /// Source range.
    pub range: TextRange,
    /// Return type with its span.
    pub return_type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleException {
    /// Source range.
    pub range: TextRange,
    /// Exception type with its span.
    pub r#type: Spanned<String>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style warning (from Warns section).
///
/// Same shape as [`GoogleException`] but represents a warning class.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleWarning {
    /// Source range.
    pub range: TextRange,
    /// Warning type (e.g., "DeprecationWarning") with its span.
    pub warning_type: Spanned<String>,
    /// Description of when the warning is issued, with its span.
    pub description: Spanned<String>,
}

/// Google-style See Also item.
///
/// Supports both `:role:`name`` cross-references and plain names.
///
/// ```text
/// See Also:
///     func_a: Description of func_a.
///     func_b, func_c
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleSeeAlsoItem {
    /// Source range.
    pub range: TextRange,
    /// Reference names (can be multiple like `func_b, func_c`), each with its own span.
    pub names: Vec<Spanned<String>>,
    /// Optional description with its span.
    pub description: Option<Spanned<String>>,
}

/// Google-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleAttribute {
    /// Source range.
    pub range: TextRange,
    /// Attribute name with its span.
    pub name: Spanned<String>,
    /// Attribute type (inside parentheses) with its span.
    pub r#type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style method entry (from Methods section).
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleMethod {
    /// Source range.
    pub range: TextRange,
    /// Method name with its span.
    pub name: Spanned<String>,
    /// Brief description with its span.
    pub description: Spanned<String>,
}

impl GoogleDocstring {
    /// Creates a new empty Google-style docstring.
    pub fn new() -> Self {
        Self {
            source: String::new(),
            range: TextRange::empty(),
            summary: Spanned::empty_string(),
            extended_summary: None,
            sections: Vec::new(),
        }
    }
}

impl Default for GoogleDocstring {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GoogleDocstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GoogleDocstring(summary: {})", self.summary.value)
    }
}
