use core::fmt;

use crate::ast::{AttributeView, DocstringLike, ExceptionView, ParameterView, ReturnsView};
use crate::ast::{TextRange, Spanned};

// =============================================================================
// Sphinx Style Types
// =============================================================================

/// Sphinx-style docstring.
///
/// **Note:** Sphinx style is not supported in v1. This type is defined for
/// forward compatibility. The parser currently only extracts the summary line.
///
/// Sphinx format uses field lists like:
/// ```text
/// :param name: Description
/// :type name: type
/// :returns: Description
/// :rtype: type
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxDocstring {
    /// Original source text of the docstring.
    pub source: String,
    /// Source span of the entire docstring.
    pub range: TextRange,
    /// Brief summary (first paragraph).
    pub summary: Spanned<String>,
    /// Extended description.
    pub description: Option<Spanned<String>>,
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
    pub range: TextRange,
    /// Parameter name with its span.
    pub name: Spanned<String>,
    /// Parameter type (from :type: field) with its span.
    pub param_type: Option<Spanned<String>>,
    /// Parameter description (from :param: field) with its span.
    pub description: Spanned<String>,
}

/// Sphinx-style return value.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxReturns {
    /// Source span.
    pub range: TextRange,
    /// Return type (from :rtype: field) with its span.
    pub return_type: Option<Spanned<String>>,
    /// Description (from :returns: or :return: field) with its span.
    pub description: Spanned<String>,
}

/// Sphinx-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxException {
    /// Source span.
    pub range: TextRange,
    /// Exception type with its span.
    pub exception_type: Spanned<String>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Sphinx-style variable (var, ivar, cvar).
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxVariable {
    /// Source span.
    pub range: TextRange,
    /// Variable name with its span.
    pub name: Spanned<String>,
    /// Variable type with its span.
    pub var_type: Option<Spanned<String>>,
    /// Description with its span.
    pub description: Spanned<String>,
}

/// Custom Sphinx field.
#[derive(Debug, Clone, PartialEq)]
pub struct SphinxField {
    /// Source span.
    pub range: TextRange,
    /// Field name (e.g., "deprecated", "since", "author") with its span.
    pub field_name: Spanned<String>,
    /// Field argument (e.g., variable name for :type:) with its span.
    pub argument: Option<Spanned<String>>,
    /// Field content with its span.
    pub content: Spanned<String>,
}

impl SphinxDocstring {
    /// Creates a new empty Sphinx-style docstring.
    pub fn new() -> Self {
        Self {
            source: String::new(),
            range: TextRange::empty(),
            summary: Spanned::empty_string(),
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
        write!(f, "SphinxDocstring(summary: {})", self.summary.value)
    }
}

impl DocstringLike for SphinxDocstring {
    fn summary(&self) -> &str {
        &self.summary.value
    }

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.value.as_str())
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        self.parameters
            .iter()
            .map(|p| ParameterView {
                name: p.name.as_spanned_str(),
                param_type: p.param_type.as_ref().map(|t| t.as_spanned_str()),
                description: p.description.as_spanned_str(),
                optional: None,
                range: p.range,
            })
            .collect()
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        match &self.returns {
            Some(r) => vec![ReturnsView {
                name: None,
                return_type: r.return_type.as_ref().map(|t| t.as_spanned_str()),
                description: r.description.as_spanned_str(),
                range: r.range,
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
                range: e.range,
            })
            .collect()
    }

    fn attributes(&self) -> Vec<AttributeView<'_>> {
        self.variables
            .iter()
            .chain(self.instance_variables.iter())
            .chain(self.class_variables.iter())
            .map(|v| AttributeView {
                name: v.name.as_spanned_str(),
                attr_type: v.var_type.as_ref().map(|t| t.as_spanned_str()),
                description: v.description.as_spanned_str(),
                range: v.range,
            })
            .collect()
    }
}
