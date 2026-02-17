use core::fmt;

use crate::span::Span;
use crate::traits::DocstringLike;
use crate::views::{AttributeView, ExceptionView, ParameterView, ReturnsView};

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
    /// Source span of the entire docstring.
    pub span: Span,
    /// Brief summary (first line).
    pub summary: String,
    /// Extended description.
    pub description: Option<String>,
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
    pub note: Option<String>,
    /// Example section.
    pub example: Option<String>,
    /// Todo section.
    pub todo: Vec<String>,
}

/// Google-style argument.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleArgument {
    /// Source span.
    pub span: Span,
    /// Argument name.
    pub name: String,
    /// Argument type (inside parentheses).
    pub arg_type: Option<String>,
    /// Argument description.
    pub description: String,
    /// Whether marked as optional.
    pub optional: bool,
}

/// Google-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleReturns {
    /// Source span.
    pub span: Span,
    /// Return type.
    pub return_type: Option<String>,
    /// Description.
    pub description: String,
}

/// Google-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleException {
    /// Source span.
    pub span: Span,
    /// Exception type.
    pub exception_type: String,
    /// Description.
    pub description: String,
}

/// Google-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleAttribute {
    /// Source span.
    pub span: Span,
    /// Attribute name.
    pub name: String,
    /// Attribute type (inside parentheses).
    pub attr_type: Option<String>,
    /// Description.
    pub description: String,
}

impl GoogleDocstring {
    /// Creates a new empty Google-style docstring.
    pub fn new() -> Self {
        Self {
            span: Span::empty(),
            summary: String::new(),
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
        write!(f, "GoogleDocstring(summary: {})", self.summary)
    }
}

impl DocstringLike for GoogleDocstring {
    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        self.args
            .iter()
            .map(|a| ParameterView {
                name: &a.name,
                param_type: a.arg_type.as_deref(),
                description: &a.description,
                optional: a.optional,
                span: a.span,
            })
            .collect()
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        match &self.returns {
            Some(r) => vec![ReturnsView {
                name: None,
                return_type: r.return_type.as_deref(),
                description: &r.description,
                span: r.span,
            }],
            None => Vec::new(),
        }
    }

    fn raises(&self) -> Vec<ExceptionView<'_>> {
        self.raises
            .iter()
            .map(|e| ExceptionView {
                exception_type: &e.exception_type,
                description: &e.description,
                span: e.span,
            })
            .collect()
    }

    fn attributes(&self) -> Vec<AttributeView<'_>> {
        self.attributes
            .iter()
            .map(|a| AttributeView {
                name: &a.name,
                attr_type: a.attr_type.as_deref(),
                description: &a.description,
                span: a.span,
            })
            .collect()
    }
}
