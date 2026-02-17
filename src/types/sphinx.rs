use core::fmt;

use crate::span::Span;
use crate::traits::DocstringLike;
use crate::views::{AttributeView, ExceptionView, ParameterView, ReturnsView};

// =============================================================================
// Sphinx Style Types
// =============================================================================

/// Sphinx-style docstring.
///
/// Supports field lists like:
/// ```text
/// :param name: Description
/// :type name: type
/// :returns: Description
/// :rtype: type
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxDocstring {
    /// Source span of the entire docstring.
    pub span: Span,
    /// Brief summary (first paragraph).
    pub summary: String,
    /// Extended description.
    pub description: Option<String>,
    /// Parameters with separate type information.
    pub parameters: Vec<SphinxParameter>,
    /// Return value information.
    pub returns: Option<SphinxReturns>,
    /// Exceptions that may be raised.
    pub raises: Vec<SphinxException>,
    /// Variables (:var:).
    pub variables: Vec<SphinxVariable>,
    /// Instance variables (:ivar:).
    pub instance_variables: Vec<SphinxVariable>,
    /// Class variables (:cvar:).
    pub class_variables: Vec<SphinxVariable>,
    /// Additional custom fields.
    pub custom_fields: Vec<SphinxField>,
}

/// Sphinx-style parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxParameter {
    /// Source span.
    pub span: Span,
    /// Parameter name.
    pub name: String,
    /// Parameter type (from :type: field).
    pub param_type: Option<String>,
    /// Parameter description (from :param: field).
    pub description: String,
}

/// Sphinx-style return value.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxReturns {
    /// Source span.
    pub span: Span,
    /// Return type (from :rtype: field).
    pub return_type: Option<String>,
    /// Description (from :returns: or :return: field).
    pub description: String,
}

/// Sphinx-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxException {
    /// Source span.
    pub span: Span,
    /// Exception type.
    pub exception_type: String,
    /// Description.
    pub description: String,
}

/// Sphinx-style variable (var, ivar, cvar).
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxVariable {
    /// Source span.
    pub span: Span,
    /// Variable name.
    pub name: String,
    /// Variable type.
    pub var_type: Option<String>,
    /// Description.
    pub description: String,
}

/// Custom Sphinx field.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxField {
    /// Source span.
    pub span: Span,
    /// Field name (e.g., "deprecated", "since", "author").
    pub field_name: String,
    /// Field argument (e.g., variable name for :type:).
    pub argument: Option<String>,
    /// Field content.
    pub content: String,
}

impl SphinxDocstring {
    /// Creates a new empty Sphinx-style docstring.
    pub fn new() -> Self {
        Self {
            span: Span::empty(),
            summary: String::new(),
            description: None,
            parameters: Vec::new(),
            returns: None,
            raises: Vec::new(),
            variables: Vec::new(),
            instance_variables: Vec::new(),
            class_variables: Vec::new(),
            custom_fields: Vec::new(),
        }
    }
}

impl Default for SphinxDocstring {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SphinxDocstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SphinxDocstring(summary: {})", self.summary)
    }
}

impl DocstringLike for SphinxDocstring {
    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        self.parameters
            .iter()
            .map(|p| ParameterView {
                name: &p.name,
                param_type: p.param_type.as_deref(),
                description: &p.description,
                optional: false,
                span: p.span,
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
        self.variables
            .iter()
            .chain(self.instance_variables.iter())
            .chain(self.class_variables.iter())
            .map(|v| AttributeView {
                name: &v.name,
                attr_type: v.var_type.as_deref(),
                description: &v.description,
                span: v.span,
            })
            .collect()
    }
}
