use core::fmt;

use crate::ast::{Spanned, TextRange};

// =============================================================================
// NumPy Style Types
// =============================================================================

/// A single NumPy-style section, combining header and body.
///
/// ```text
/// Parameters       <-- header
/// ----------       <-- header (underline)
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
/// Parameters     <-- name line
/// ----------     <-- underline line
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
    pub param_type: Option<Spanned<String>>,
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

/// NumPy-style warning (from Warns section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyWarning {
    /// Source span.
    pub range: TextRange,
    /// Warning type (e.g., "DeprecationWarning") with its span.
    pub warning_type: Spanned<String>,
    /// When the warning is issued, with its span.
    pub description: Spanned<String>,
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
    pub attr_type: Option<Spanned<String>>,
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

/// NumPy-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyException {
    /// Source span.
    pub range: TextRange,
    /// Exception type with its span.
    pub exception_type: Spanned<String>,
    /// Description of when raised, with its span.
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

    // ---- Convenience accessors ------------------------------------------------

    /// Returns an iterator over parameters from all Parameters sections.
    pub fn parameters(&self) -> Vec<&NumPyParameter> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Parameters(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over return values from all Returns sections.
    pub fn returns(&self) -> Vec<&NumPyReturns> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Returns(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over yield values from all Yields sections.
    pub fn yields(&self) -> Vec<&NumPyReturns> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Yields(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over receive values from all Receives sections.
    pub fn receives(&self) -> Vec<&NumPyParameter> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Receives(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over other parameters from all Other Parameters sections.
    pub fn other_parameters(&self) -> Vec<&NumPyParameter> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::OtherParameters(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over exceptions from all Raises sections.
    pub fn raises(&self) -> Vec<&NumPyException> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over warnings from all Warns sections.
    pub fn warns(&self) -> Vec<&NumPyWarning> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Warns(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns the Warnings section content, if present.
    pub fn warnings(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            NumPySectionBody::Warnings(text) => Some(text),
            _ => None,
        })
    }

    /// Returns an iterator over see-also items from all See Also sections.
    pub fn see_also(&self) -> Vec<&SeeAlsoItem> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::SeeAlso(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns the Notes section content, if present.
    pub fn notes(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            NumPySectionBody::Notes(text) => Some(text),
            _ => None,
        })
    }

    /// Returns an iterator over references from all References sections.
    pub fn references(&self) -> Vec<&NumPyReference> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::References(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns the Examples section content, if present.
    pub fn examples(&self) -> Option<&Spanned<String>> {
        self.sections.iter().find_map(|s| match &s.body {
            NumPySectionBody::Examples(text) => Some(text),
            _ => None,
        })
    }

    /// Returns an iterator over attributes from all Attributes sections.
    pub fn attributes(&self) -> Vec<&NumPyAttribute> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Attributes(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Returns an iterator over methods from all Methods sections.
    pub fn methods(&self) -> Vec<&NumPyMethod> {
        self.sections
            .iter()
            .filter_map(|s| match &s.body {
                NumPySectionBody::Methods(v) => Some(v.iter()),
                _ => None,
            })
            .flatten()
            .collect()
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
