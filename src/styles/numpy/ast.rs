use core::fmt;

use crate::ast::TextRange;

// =============================================================================
// NumPy Style Types
// =============================================================================

/// NumPy-style section kinds.
///
/// Each variant represents a recognised section name (or group of aliases),
/// or [`Unknown`](Self::Unknown) for unrecognised names.
/// Use [`NumPySectionKind::from_name`] to convert a lowercased section name
/// to a variant.
///
/// Having an enum instead of a plain string list gives compile-time
/// exhaustiveness checks: every variant must be handled when matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumPySectionKind {
    /// `Parameters` / `Params`
    Parameters,
    /// `Returns` / `Return`
    Returns,
    /// `Yields` / `Yield`
    Yields,
    /// `Receives` / `Receive`
    Receives,
    /// `Other Parameters` / `Other Params`
    OtherParameters,
    /// `Raises` / `Raise`
    Raises,
    /// `Warns` / `Warn`
    Warns,
    /// `Warnings` / `Warning`
    Warnings,
    /// `See Also`
    SeeAlso,
    /// `Notes` / `Note`
    Notes,
    /// `References`
    References,
    /// `Examples` / `Example`
    Examples,
    /// `Attributes`
    Attributes,
    /// `Methods`
    Methods,
    /// Unrecognised section name.
    Unknown,
}

impl NumPySectionKind {
    /// All known section kinds (useful for iteration / testing).
    pub const ALL: &[NumPySectionKind] = &[
        Self::Parameters,
        Self::Returns,
        Self::Yields,
        Self::Receives,
        Self::OtherParameters,
        Self::Raises,
        Self::Warns,
        Self::Warnings,
        Self::SeeAlso,
        Self::Notes,
        Self::References,
        Self::Examples,
        Self::Attributes,
        Self::Methods,
    ];

    /// Convert a **lowercased** section name to a [`NumPySectionKind`].
    ///
    /// Returns [`Unknown`](Self::Unknown) for unrecognised names (which are
    /// dispatched as `NumPySectionBody::Unknown` by the parser).
    pub fn from_name(name: &str) -> Self {
        match name {
            "parameters" | "parameter" | "params" | "param" => Self::Parameters,
            "returns" | "return" => Self::Returns,
            "yields" | "yield" => Self::Yields,
            "receives" | "receive" => Self::Receives,
            "other parameters" | "other parameter" | "other params" | "other param" => {
                Self::OtherParameters
            }
            "raises" | "raise" => Self::Raises,
            "warns" | "warn" => Self::Warns,
            "warnings" | "warning" => Self::Warnings,
            "see also" => Self::SeeAlso,
            "notes" | "note" => Self::Notes,
            "references" => Self::References,
            "examples" | "example" => Self::Examples,
            "attributes" => Self::Attributes,
            "methods" => Self::Methods,
            _ => Self::Unknown,
        }
    }

    /// Check if a lowercased name is a known (non-[`Unknown`](Self::Unknown)) section name.
    pub fn is_known(name: &str) -> bool {
        !matches!(Self::from_name(name), Self::Unknown)
    }
}

impl fmt::Display for NumPySectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Parameters => "Parameters",
            Self::Returns => "Returns",
            Self::Yields => "Yields",
            Self::Receives => "Receives",
            Self::OtherParameters => "Other Parameters",
            Self::Raises => "Raises",
            Self::Warns => "Warns",
            Self::Warnings => "Warnings",
            Self::SeeAlso => "See Also",
            Self::Notes => "Notes",
            Self::References => "References",
            Self::Examples => "Examples",
            Self::Attributes => "Attributes",
            Self::Methods => "Methods",
            Self::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

/// A single item in the body of a NumPy-style docstring (after the summary
/// and optional extended summary).
///
/// Preserving both sections and stray lines in a single ordered `Vec` ensures
/// the original source order is never lost, which matters for linters that
/// want to report diagnostics in document order.
#[derive(Debug, Clone, PartialEq)]
pub enum NumPyDocstringItem {
    /// A recognised (or unknown-name) section with header + underline + body.
    Section(NumPySection),
    /// A non-blank line that appeared between sections but was neither blank
    /// nor recognised as a section header (i.e. not followed by an underline).
    ///
    /// Typical causes include misplaced prose, a section name whose underline
    /// was accidentally omitted, or text that belongs to a previous section.
    StrayLine(TextRange),
}

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
    /// Resolved section kind.
    pub kind: NumPySectionKind,
    /// Section name as written in source (e.g., "Parameters", "Params") with its span.
    pub name: TextRange,
    /// Underline (dashes) line with its span.
    pub underline: TextRange,
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
    Warnings(TextRange),
    /// See Also section.
    SeeAlso(Vec<SeeAlsoItem>),
    /// Notes section (free text).
    Notes(TextRange),
    /// References section.
    References(Vec<NumPyReference>),
    /// Examples section (free text, doctest format).
    Examples(TextRange),
    /// Attributes section.
    Attributes(Vec<NumPyAttribute>),
    /// Methods section.
    Methods(Vec<NumPyMethod>),
    /// Unknown / unrecognized section (free text).
    Unknown(TextRange),
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
    /// Brief summary (first paragraph, up to the first blank line).
    pub summary: Option<TextRange>,
    /// Deprecation warning (if applicable).
    pub deprecation: Option<NumPyDeprecation>,
    /// Extended summary (multiple sentences before any section header).
    /// Clarifies functionality, may reference parameters.
    pub extended_summary: Option<TextRange>,
    /// All items (sections and stray lines) in order of appearance.
    pub items: Vec<NumPyDocstringItem>,
}

/// NumPy-style deprecation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyDeprecation {
    /// Source span.
    pub range: TextRange,
    /// The `..` RST directive marker, with its span.
    pub directive_marker: Option<TextRange>,
    /// The `deprecated` keyword, with its span.
    pub keyword: Option<TextRange>,
    /// The `::` double-colon separator, with its span.
    pub double_colon: Option<TextRange>,
    /// Version when deprecated (e.g., "1.6.0") with its span.
    pub version: TextRange,
    /// Reason for deprecation and recommendation (free text body), with its span.
    pub description: TextRange,
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
    pub names: Vec<TextRange>,
    /// The colon separator (`:`) between name(s) and type, if present.
    ///
    /// `None` when the colon is missing (best-effort parse of a bare name).
    /// A linter can use this to report a missing colon.
    pub colon: Option<TextRange>,
    /// Parameter type (e.g., "int", "str", "array_like") with its span.
    /// Type is optional for parameters but required for returns.
    pub r#type: Option<TextRange>,
    /// Parameter description with its span.
    pub description: TextRange,
    /// The `optional` marker, if present.
    /// `None` means not marked as optional.
    pub optional: Option<TextRange>,
    /// The `default` keyword, if present (e.g., `"default"`).
    pub default_keyword: Option<TextRange>,
    /// The separator after `default` (`=` or `:`), if present.
    /// `None` when the value follows after whitespace only (e.g., `default True`).
    pub default_separator: Option<TextRange>,
    /// Default value (e.g., "True", "-1", "None") with its span.
    /// `None` when `default` appears alone without a value.
    pub default_value: Option<TextRange>,
}

/// NumPy-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReturns {
    /// Source span.
    pub range: TextRange,
    /// Return value name (optional in NumPy style) with its span.
    pub name: Option<TextRange>,
    /// The colon (`:`) separating name from type, with its span, if present.
    pub colon: Option<TextRange>,
    /// Return type with its span.
    pub return_type: Option<TextRange>,
    /// Description with its span.
    pub description: TextRange,
}

/// NumPy-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyException {
    /// Source span.
    pub range: TextRange,
    /// Exception type with its span.
    pub r#type: TextRange,
    /// The colon (`:`) separating type from description, with its span, if present.
    pub colon: Option<TextRange>,
    /// Description of when raised, with its span.
    pub description: TextRange,
}

/// NumPy-style warning (from Warns section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyWarning {
    /// Source span.
    pub range: TextRange,
    /// Warning type (e.g., "DeprecationWarning") with its span.
    pub r#type: TextRange,
    /// The colon (`:`) separating type from description, with its span, if present.
    pub colon: Option<TextRange>,
    /// When the warning is issued, with its span.
    pub description: TextRange,
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
    pub names: Vec<TextRange>,
    /// The colon (`:`) separating names from description, with its span, if present.
    pub colon: Option<TextRange>,
    /// Optional description with its span.
    pub description: Option<TextRange>,
}

/// Numbered reference (from References section).
///
/// Represents an RST citation reference like `.. [1] Author, Title`.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReference {
    /// Source span.
    pub range: TextRange,
    /// The RST directive marker (`..`) with its span, if present.
    ///
    /// `None` for non-RST (plain text) references.
    pub directive_marker: Option<TextRange>,
    /// Opening bracket (`[`) enclosing the reference number, with its span, if present.
    pub open_bracket: Option<TextRange>,
    /// Reference number (e.g., "1", "2", "3") with its span.
    pub number: TextRange,
    /// Closing bracket (`]`) enclosing the reference number, with its span, if present.
    pub close_bracket: Option<TextRange>,
    /// Reference content (author, title, etc) with its span.
    pub content: TextRange,
}

/// NumPy-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyAttribute {
    /// Source span.
    pub range: TextRange,
    /// Attribute name with its span.
    pub name: TextRange,
    /// The colon (`:`) separating name from type, with its span, if present.
    pub colon: Option<TextRange>,
    /// Attribute type with its span.
    pub r#type: Option<TextRange>,
    /// Description with its span.
    pub description: TextRange,
}

/// NumPy-style method (for classes).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyMethod {
    /// Source span.
    pub range: TextRange,
    /// Method name with its span.
    pub name: TextRange,
    /// The colon (`:`) separating name from description, with its span, if present.
    pub colon: Option<TextRange>,
    /// Brief description with its span.
    pub description: TextRange,
}

impl NumPyDocstring {
    /// Creates a new empty NumPy-style docstring with the given source.
    pub fn new(input: &str) -> Self {
        Self {
            source: input.to_string(),
            range: TextRange::empty(),
            summary: None,
            deprecation: None,
            extended_summary: None,
            items: Vec::new(),
        }
    }
}

impl Default for NumPyDocstring {
    fn default() -> Self {
        Self::new("")
    }
}

impl fmt::Display for NumPyDocstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NumPyDocstring(summary: {})",
            self.summary
                .as_ref()
                .map_or("", |s| s.source_text(&self.source))
        )
    }
}
