use core::fmt;

use crate::ast::{Spanned, TextRange};

// =============================================================================
// Google Style Types
// =============================================================================

/// Google-style section kinds.
///
/// Each variant represents a recognised section name (or group of aliases),
/// or [`Unknown`](Self::Unknown) for unrecognised names.
/// Use [`GoogleSectionKind::from_name`] to convert a lowercased section name
/// to a variant.
///
/// Having an enum instead of a plain string list gives compile-time
/// exhaustiveness checks: every variant must be handled when matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoogleSectionKind {
    /// `Args` / `Arguments` / `Parameters` / `Params`
    Args,
    /// `Keyword Args` / `Keyword Arguments`
    KeywordArgs,
    /// `Other Parameters`
    OtherParameters,
    /// `Receive` / `Receives`
    Receives,
    /// `Returns` / `Return`
    Returns,
    /// `Yields` / `Yield`
    Yields,
    /// `Raises` / `Raise`
    Raises,
    /// `Warns` / `Warn`
    Warns,
    /// `Attributes` / `Attribute`
    Attributes,
    /// `Methods`
    Methods,
    /// `See Also`
    SeeAlso,
    /// `Note` / `Notes`
    Notes,
    /// `Example` / `Examples`
    Examples,
    /// `Todo`
    Todo,
    /// `References`
    References,
    /// `Warning` / `Warnings`
    Warnings,
    /// `Attention`
    Attention,
    /// `Caution`
    Caution,
    /// `Danger`
    Danger,
    /// `Error`
    Error,
    /// `Hint`
    Hint,
    /// `Important`
    Important,
    /// `Tip`
    Tip,
    /// Unrecognised section name.
    Unknown,
}

impl GoogleSectionKind {
    /// All known section kinds (useful for iteration / testing).
    pub const ALL: &[GoogleSectionKind] = &[
        Self::Args,
        Self::KeywordArgs,
        Self::OtherParameters,
        Self::Receives,
        Self::Returns,
        Self::Yields,
        Self::Raises,
        Self::Warns,
        Self::Attributes,
        Self::Methods,
        Self::SeeAlso,
        Self::Notes,
        Self::Examples,
        Self::Todo,
        Self::References,
        Self::Warnings,
        Self::Attention,
        Self::Caution,
        Self::Danger,
        Self::Error,
        Self::Hint,
        Self::Important,
        Self::Tip,
    ];

    /// Convert a **lowercased** section name to a [`GoogleSectionKind`].
    ///
    /// Returns [`Unknown`](Self::Unknown) for unrecognised names (which are
    /// dispatched as `GoogleSectionBody::Unknown` by the parser).
    pub fn from_name(name: &str) -> Self {
        match name {
            "args" | "arguments" | "params" | "parameters" => Self::Args,
            "keyword args" | "keyword arguments" | "keyword params" | "keyword parameters" => {
                Self::KeywordArgs
            }
            "other args" | "other arguments" | "other params" | "other parameters" => {
                Self::OtherParameters
            }
            "receives" | "receive" => Self::Receives,
            "returns" | "return" => Self::Returns,
            "yields" | "yield" => Self::Yields,
            "raises" | "raise" => Self::Raises,
            "warns" | "warn" => Self::Warns,
            "see also" => Self::SeeAlso,
            "attributes" | "attribute" => Self::Attributes,
            "methods" => Self::Methods,
            "notes" | "note" => Self::Notes,
            "examples" | "example" => Self::Examples,
            "todo" => Self::Todo,
            "references" => Self::References,
            "warnings" | "warning" => Self::Warnings,
            "attention" => Self::Attention,
            "caution" => Self::Caution,
            "danger" => Self::Danger,
            "error" => Self::Error,
            "hint" => Self::Hint,
            "important" => Self::Important,
            "tip" => Self::Tip,
            _ => Self::Unknown,
        }
    }

    /// Check if a lowercased name is a known (non-[`Unknown`](Self::Unknown)) section name.
    pub fn is_known(name: &str) -> bool {
        !matches!(Self::from_name(name), Self::Unknown)
    }
}

impl fmt::Display for GoogleSectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Args => "Args",
            Self::KeywordArgs => "Keyword Args",
            Self::OtherParameters => "Other Parameters",
            Self::Receives => "Receives",
            Self::Returns => "Returns",
            Self::Yields => "Yields",
            Self::Raises => "Raises",
            Self::Warns => "Warns",
            Self::SeeAlso => "See Also",
            Self::Attributes => "Attributes",
            Self::Methods => "Methods",
            Self::Notes => "Notes",
            Self::Examples => "Examples",
            Self::Todo => "Todo",
            Self::References => "References",
            Self::Warnings => "Warnings",
            Self::Attention => "Attention",
            Self::Caution => "Caution",
            Self::Danger => "Danger",
            Self::Error => "Error",
            Self::Hint => "Hint",
            Self::Important => "Important",
            Self::Tip => "Tip",
            Self::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

/// A single item in the body of a Google-style docstring (after the summary
/// and optional extended summary).
///
/// Preserving both sections and stray lines in a single ordered `Vec` ensures
/// the original source order is never lost, which matters for linters that
/// want to report diagnostics in document order.
#[derive(Debug, Clone, PartialEq)]
pub enum GoogleDocstringItem {
    /// A recognised (or unknown-name) section, e.g. `Args:` or `Custom:`.
    Section(GoogleSection),
    /// A non-blank line that appeared between sections but was neither blank
    /// nor recognised as a section header.
    ///
    /// Typical causes include misplaced prose, a section name whose colon was
    /// accidentally omitted, or an entry that was not indented correctly.
    StrayLine(Spanned<String>),
}

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
    /// Resolved section kind.
    pub kind: GoogleSectionKind,
    /// Section name as written in source (e.g., "Args", "Parameters") with its span.
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
    Returns(GoogleReturns),
    /// Yields / Yield section.
    Yields(GoogleReturns),

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
    /// All sections and stray lines in document order.
    ///
    /// Use [`sections()`](Self::sections) to iterate only over
    /// [`GoogleSection`] items, or [`stray_lines()`](Self::stray_lines) to
    /// iterate only over stray lines.
    pub items: Vec<GoogleDocstringItem>,
}

/// Google-style argument.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleArg {
    /// Source range.
    pub range: TextRange,
    /// Argument name with its span.
    pub name: Spanned<String>,
    /// Opening bracket (`(`, `[`, `{`, or `<`) enclosing the type, with its span.
    pub open_bracket: Option<Spanned<String>>,
    /// Argument type (inside brackets) with its span.
    pub r#type: Option<Spanned<String>>,
    /// Closing bracket (`)`, `]`, `}`, or `>`) enclosing the type, with its span.
    pub close_bracket: Option<Spanned<String>>,
    /// The colon (`:`) separating name/type from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// The colon (`:`) separating type and description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// The colon (`:`) separating type from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// The colon (`:`) separating type from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// The colon (`:`) separating names from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// Opening bracket (`(`, `[`, `{`, or `<`) enclosing the type, with its span.
    pub open_bracket: Option<Spanned<String>>,
    /// Attribute type (inside brackets) with its span.
    pub r#type: Option<Spanned<String>>,
    /// Closing bracket (`)`, `]`, `}`, or `>`) enclosing the type, with its span.
    pub close_bracket: Option<Spanned<String>>,
    /// The colon (`:`) separating name/type from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
    /// Opening bracket (`(`, `[`, `{`, or `<`) enclosing the signature/type, with its span.
    pub open_bracket: Option<Spanned<String>>,
    /// Method signature or type (inside brackets) with its span.
    pub r#type: Option<Spanned<String>>,
    /// Closing bracket (`)`, `]`, `}`, or `>`) enclosing the signature/type, with its span.
    pub close_bracket: Option<Spanned<String>>,
    /// The colon (`:`) separating name from description, with its span, if present.
    pub colon: Option<Spanned<String>>,
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
            items: Vec::new(),
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
