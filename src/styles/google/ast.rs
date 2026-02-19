use core::fmt;

use crate::ast::{Spanned, TextRange};

// =============================================================================
// Google Style Types
// =============================================================================

/// A single Google-style section, combining header and body.
///
/// ```text
/// Args:                <-- header
///     x (int): Value.  <-- body
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleSection {
    /// Source range of the entire section (header + body).
    pub range: TextRange,
    /// Section header (the `Args:` line).
    pub header: GoogleSectionHeader,
    /// Section body content.
    pub body: GoogleSectionBody,
}

/// Google-style section header.
///
/// Represents a parsed section header like `Args:` or `Returns:`.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleSectionHeader {
    /// Source range of the header line.
    pub range: TextRange,
    /// Section name (e.g., "Args", "Returns") with its span.
    /// Stored without the trailing colon.
    pub name: Spanned<String>,
}

/// Body content of a Google-style section.
///
/// Each variant corresponds to a specific section kind.
/// Section names follow the Sphinx Napoleon extension.
#[derive(Debug, Clone, PartialEq)]
pub enum GoogleSectionBody {
    // ----- Parameter-like sections -----
    /// Args / Arguments / Parameters / Params section.
    Args(Vec<GoogleArgument>),
    /// Keyword Args / Keyword Arguments section.
    KeywordArgs(Vec<GoogleArgument>),
    /// Other Parameters section.
    OtherParameters(Vec<GoogleArgument>),
    /// Receive / Receives section.
    Receives(Vec<GoogleArgument>),

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
    /// Attributes / Attribute section.
    Attributes(Vec<GoogleAttribute>),
    /// Methods section.
    Methods(Vec<GoogleMethod>),
    /// See Also section.
    SeeAlso(Vec<GoogleSeeAlsoItem>),

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
    /// Extended description.
    pub description: Option<Spanned<String>>,
    /// All sections in order of appearance.
    pub sections: Vec<GoogleSection>,
}

/// Google-style argument.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleArgument {
    /// Source range.
    pub range: TextRange,
    /// Argument name with its span.
    pub name: Spanned<String>,
    /// Argument type (inside parentheses) with its span.
    pub arg_type: Option<Spanned<String>>,
    /// Argument description with its span.
    pub description: Spanned<String>,
    /// Source range of the `optional` marker, if present.
    /// `None` means not marked as optional, `Some(range)` gives the location of `optional` text.
    pub optional: Option<TextRange>,
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
    pub exception_type: Spanned<String>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleAttribute {
    /// Source range.
    pub range: TextRange,
    /// Attribute name with its span.
    pub name: Spanned<String>,
    /// Attribute type (inside parentheses) with its span.
    pub attr_type: Option<Spanned<String>>,
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

impl GoogleDocstring {
    /// Creates a new empty Google-style docstring.
    pub fn new() -> Self {
        Self {
            source: String::new(),
            range: TextRange::empty(),
            summary: Spanned::empty_string(),
            description: None,
            sections: Vec::new(),
        }
    }

    // ---- Convenience accessors ------------------------------------------------

    /// Returns arguments from all Args sections.
    pub fn args(&self) -> Vec<&GoogleArgument> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Args(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns return values from all Returns sections.
    pub fn returns(&self) -> Vec<&GoogleReturns> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Returns(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns yield values from all Yields sections.
    pub fn yields(&self) -> Vec<&GoogleReturns> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Yields(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns exceptions from all Raises sections.
    pub fn raises(&self) -> Vec<&GoogleException> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns attributes from all Attributes sections.
    pub fn attributes(&self) -> Vec<&GoogleAttribute> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Attributes(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns the Notes section content, if present.
    pub fn notes(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Notes(text) => Some(text),
            _ => None,
        })
    }

    /// Returns the Examples section content, if present.
    pub fn examples(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Examples(text) => Some(text),
            _ => None,
        })
    }

    /// Returns keyword arguments from all Keyword Args sections.
    pub fn keyword_args(&self) -> Vec<&GoogleArgument> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::KeywordArgs(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns other parameters from all Other Parameters sections.
    pub fn other_parameters(&self) -> Vec<&GoogleArgument> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::OtherParameters(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns receive values from all Receives sections.
    pub fn receives(&self) -> Vec<&GoogleArgument> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Receives(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns warnings from all Warns sections.
    pub fn warns(&self) -> Vec<&GoogleWarning> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Warns(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns methods from all Methods sections.
    pub fn methods(&self) -> Vec<&GoogleMethod> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Methods(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns See Also items from all See Also sections.
    pub fn see_also(&self) -> Vec<&GoogleSeeAlsoItem> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::SeeAlso(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns the Todo section content, if present.
    pub fn todo(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Todo(text) => Some(text),
            _ => None,
        })
    }

    /// Returns the References section content, if present.
    pub fn references(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::References(text) => Some(text),
            _ => None,
        })
    }

    /// Returns the Warnings section content, if present.
    pub fn warnings(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Warnings(text) => Some(text),
            _ => None,
        })
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
