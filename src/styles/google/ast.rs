use core::fmt;

use crate::ast::{TextRange, Spanned};

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
#[derive(Debug, Clone, PartialEq)]
pub enum GoogleSectionBody {
    /// Args / Arguments section.
    Args(Vec<GoogleArgument>),
    /// Returns / Return section.
    Returns(Vec<GoogleReturns>),
    /// Yields / Yield section.
    Yields(Vec<GoogleReturns>),
    /// Raises section.
    Raises(Vec<GoogleException>),
    /// Attributes section.
    Attributes(Vec<GoogleAttribute>),
    /// Note / Notes section (free text).
    Note(Spanned<String>),
    /// Example / Examples section (free text).
    Example(Spanned<String>),
    /// Todo section (bulleted items).
    Todo(Vec<Spanned<String>>),
    /// References section (free text).
    References(Spanned<String>),
    /// Warnings section (free text).
    Warnings(Spanned<String>),
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

    /// Returns the Note section content, if present.
    pub fn note(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Note(text) => Some(text),
            _ => None,
        })
    }

    /// Returns the Example section content, if present.
    pub fn example(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            GoogleSectionBody::Example(text) => Some(text),
            _ => None,
        })
    }

    /// Returns todo items from all Todo sections.
    pub fn todo(&self) -> Vec<&Spanned<String>> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                GoogleSectionBody::Todo(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
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
