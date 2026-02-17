use core::fmt;

use crate::span::Span;
use crate::traits::DocstringLike;
use crate::views::{AttributeView, ExceptionView, ParameterView, ReturnsView};

// =============================================================================
// NumPy Style Types
// =============================================================================

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
    /// Source span of the entire docstring.
    pub span: Span,
    /// Function/method signature (optional, for C functions or when not available via introspection).
    /// Example: "add(a, b)"
    pub signature: Option<String>,
    /// Brief summary (first line).
    pub summary: String,
    /// Deprecation warning (if applicable).
    pub deprecation: Option<NumPyDeprecation>,
    /// Extended summary (multiple sentences before any section header).
    /// Clarifies functionality, may reference parameters.
    pub extended_summary: Option<String>,
    /// Function/method parameters.
    pub parameters: Vec<NumPyParameter>,
    /// Class/module attributes.
    pub attributes: Vec<NumPyAttribute>,
    /// Class/module methods (for classes with many methods).
    pub methods: Vec<NumPyMethod>,
    /// Return value(s).
    pub returns: Vec<NumPyReturns>,
    /// Generator yields.
    pub yields: Vec<NumPyReturns>,
    /// Generator receives (pairs with Yields).
    pub receives: Vec<NumPyParameter>,
    /// Infrequently used parameters.
    pub other_parameters: Vec<NumPyParameter>,
    /// Exceptions that may be raised.
    pub raises: Vec<NumPyException>,
    /// Warnings that may be issued.
    pub warns: Vec<NumPyWarning>,
    /// General warnings section (free text).
    pub warnings: Option<String>,
    /// See Also section.
    pub see_also: Vec<SeeAlsoItem>,
    /// Notes section (free text, supports reST, includes implementation details and theory).
    pub notes: Option<String>,
    /// References section.
    pub references: Vec<NumPyReference>,
    /// Examples section (doctest format).
    pub examples: Option<String>,
}

/// NumPy-style parameter.
///
/// Can represent a single parameter or multiple parameters with the same type:
/// `x : int` or `x1, x2 : array_like`
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyParameter {
    /// Source span of this parameter definition.
    pub span: Span,
    /// Parameter names (supports multiple names like `x1, x2`).
    pub names: Vec<String>,
    /// Parameter type (e.g., "int", "str", "array_like").
    /// Type is optional for parameters but required for returns.
    pub param_type: Option<String>,
    /// Parameter description.
    pub description: String,
    /// Whether marked as optional.
    pub optional: bool,
    /// Default value (e.g., "True", "-1", "None").
    pub default: Option<String>,
}

/// NumPy-style return or yield value.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReturns {
    /// Source span.
    pub span: Span,
    /// Return value name (optional in NumPy style).
    pub name: Option<String>,
    /// Return type.
    pub return_type: Option<String>,
    /// Description.
    pub description: String,
}

/// NumPy-style warning (from Warns section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyWarning {
    /// Source span.
    pub span: Span,
    /// Warning type (e.g., "DeprecationWarning").
    pub warning_type: String,
    /// When the warning is issued.
    pub description: String,
}

/// NumPy-style deprecation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyDeprecation {
    /// Source span.
    pub span: Span,
    /// Version when deprecated (e.g., "1.6.0").
    pub version: String,
    /// Version when will be removed.
    pub removal_version: Option<String>,
    /// Reason for deprecation and recommendation.
    pub reason: String,
}

/// Numbered reference (from References section).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyReference {
    /// Source span.
    pub span: Span,
    /// Reference number (1, 2, 3, ...).
    pub number: u32,
    /// Reference content (author, title, etc).
    pub content: String,
}

/// NumPy-style attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyAttribute {
    /// Source span.
    pub span: Span,
    /// Attribute name.
    pub name: String,
    /// Attribute type.
    pub attr_type: Option<String>,
    /// Description.
    pub description: String,
}

/// NumPy-style method (for classes).
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyMethod {
    /// Source span.
    pub span: Span,
    /// Method name.
    pub name: String,
    /// Brief description.
    pub description: String,
}

/// See Also item.
///
/// Supports multiple items and optional descriptions:
/// - `func_a : Description`
/// - `func_b, func_c` (multiple names, no description)
#[derive(Debug, Clone, PartialEq)]
pub struct SeeAlsoItem {
    /// Source span.
    pub span: Span,
    /// Reference names (can be multiple like `func_b, func_c`).
    pub names: Vec<String>,
    /// Optional description.
    pub description: Option<String>,
}

/// NumPy-style exception.
#[derive(Debug, Clone, PartialEq)]
pub struct NumPyException {
    /// Source span.
    pub span: Span,
    /// Exception type.
    pub exception_type: String,
    /// Description of when raised.
    pub description: String,
}

impl NumPyDocstring {
    /// Creates a new empty NumPy-style docstring.
    pub fn new() -> Self {
        Self {
            span: Span::empty(),
            signature: None,
            summary: String::new(),
            deprecation: None,
            extended_summary: None,
            parameters: Vec::new(),
            attributes: Vec::new(),
            methods: Vec::new(),
            returns: Vec::new(),
            yields: Vec::new(),
            receives: Vec::new(),
            other_parameters: Vec::new(),
            raises: Vec::new(),
            warns: Vec::new(),
            warnings: None,
            see_also: Vec::new(),
            notes: None,
            references: Vec::new(),
            examples: None,
        }
    }
}

impl Default for NumPyDocstring {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NumPyDocstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NumPyDocstring(summary: {})", self.summary)
    }
}

impl DocstringLike for NumPyDocstring {
    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.extended_summary.as_deref()
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        self.parameters
            .iter()
            .flat_map(|p| {
                p.names.iter().map(move |name| ParameterView {
                    name: name.as_str(),
                    param_type: p.param_type.as_deref(),
                    description: &p.description,
                    optional: p.optional,
                    span: p.span,
                })
            })
            .collect()
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        self.returns
            .iter()
            .map(|r| ReturnsView {
                name: r.name.as_deref(),
                return_type: r.return_type.as_deref(),
                description: &r.description,
                span: r.span,
            })
            .collect()
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
