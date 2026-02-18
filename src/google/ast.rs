use core::fmt;

use crate::ast::{Span, Spanned};
use crate::ast::{AttributeView, DocstringLike, ExceptionView, ParameterView, ReturnsView};

// =============================================================================
// Google Style Types
// =============================================================================

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
    /// Source span of the entire docstring.
    pub span: Span,
    /// Brief summary (first line).
    pub summary: Spanned<String>,
    /// Extended description.
    pub description: Option<Spanned<String>>,
    /// Function/method arguments.
    pub args: Vec<GoogleArgument>,
    /// Return value(s).
    pub returns: Option<GoogleReturns>,
    /// Generator yields.
    pub yields: Option<GoogleReturns>,
    /// Exceptions that may be raised.
    pub raises: Vec<GoogleException>,
    /// Class attributes.
    pub attributes: Vec<GoogleAttribute>,
    /// Note section.
    pub note: Option<Spanned<String>>,
    /// Example section.
    pub example: Option<Spanned<String>>,
    /// Todo section.
    pub todo: Vec<Spanned<String>>,
}

/// Google-style argument.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleArgument {
    /// Source span.
    pub span: Span,
    /// Argument name with its span.
    pub name: Spanned<String>,
    /// Argument type (inside parentheses) with its span.
    pub arg_type: Option<Spanned<String>>,
    /// Argument description with its span.
    pub description: Spanned<String>,
    /// Source span of the `optional` marker, if present.
    /// `None` means not marked as optional, `Some(span)` gives the location of `optional` text.
    pub optional: Option<Span>,
}

/// Google-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleReturns {
    /// Source span.
    pub span: Span,
    /// Return type with its span.
    pub return_type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleException {
    /// Source span.
    pub span: Span,
    /// Exception type with its span.
    pub exception_type: Spanned<String>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Google-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleAttribute {
    /// Source span.
    pub span: Span,
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
            span: Span::empty(),
            summary: Spanned::empty_string(),
            description: None,
            args: Vec::new(),
            returns: None,
            yields: None,
            raises: Vec::new(),
            attributes: Vec::new(),
            note: None,
            example: None,
            todo: Vec::new(),
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

impl DocstringLike for GoogleDocstring {
    fn summary(&self) -> &str {
        &self.summary.value
    }

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.value.as_str())
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        self.args
            .iter()
            .map(|a| ParameterView {
                name: a.name.as_spanned_str(),
                param_type: a.arg_type.as_ref().map(|t| t.as_spanned_str()),
                description: a.description.as_spanned_str(),
                optional: a.optional,
                span: a.span,
            })
            .collect()
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        match &self.returns {
            Some(r) => vec![ReturnsView {
                name: None,
                return_type: r.return_type.as_ref().map(|t| t.as_spanned_str()),
                description: r.description.as_spanned_str(),
                span: r.span,
            }],
            None => Vec::new(),
        }
    }

    fn raises(&self) -> Vec<ExceptionView<'_>> {
        self.raises
            .iter()
            .map(|e| ExceptionView {
                exception_type: e.exception_type.as_spanned_str(),
                description: e.description.as_spanned_str(),
                span: e.span,
            })
            .collect()
    }

    fn attributes(&self) -> Vec<AttributeView<'_>> {
        self.attributes
            .iter()
            .map(|a| AttributeView {
                name: a.name.as_spanned_str(),
                attr_type: a.attr_type.as_ref().map(|t| t.as_spanned_str()),
                description: a.description.as_spanned_str(),
                span: a.span,
            })
            .collect()
    }
}
