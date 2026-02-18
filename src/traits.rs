//! Style-agnostic traits and view types for docstring access.
//!
//! These traits provide a unified interface over different docstring styles,
//! enabling linters and formatters to operate without knowing the specific style.
//! Inspired by Biome's approach to multi-language support.
//!
//! The view types (`ParameterView`, `ReturnsView`, etc.) provide zero-copy
//! borrowed access into parsed docstrings of any style.

use crate::ast::{Span, Spanned};

// =============================================================================
// View types
// =============================================================================

/// A borrowed view of a parameter (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterView<'a> {
    /// Parameter name with its span.
    pub name: Spanned<&'a str>,
    /// Parameter type annotation with its span.
    pub param_type: Option<Spanned<&'a str>>,
    /// Parameter description with its span.
    pub description: Spanned<&'a str>,
    /// Source span of the `optional` marker, if present.
    /// `None` means not marked as optional, `Some(span)` gives the location of `optional` text.
    pub optional: Option<Span>,
    /// Source span of the entire parameter definition.
    pub span: Span,
}

/// A borrowed view of a return value (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnsView<'a> {
    /// Return value name (if named) with its span.
    pub name: Option<Spanned<&'a str>>,
    /// Return type annotation with its span.
    pub return_type: Option<Spanned<&'a str>>,
    /// Description of the return value with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an exception (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionView<'a> {
    /// Exception type name with its span.
    pub exception_type: Spanned<&'a str>,
    /// Description of when the exception is raised, with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an attribute (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeView<'a> {
    /// Attribute name with its span.
    pub name: Spanned<&'a str>,
    /// Attribute type annotation with its span.
    pub attr_type: Option<Spanned<&'a str>>,
    /// Description with its span.
    pub description: Spanned<&'a str>,
    /// Source span.
    pub span: Span,
}

// =============================================================================
// Trait
// =============================================================================

/// A parsed docstring of any style.
///
/// This trait abstracts over `NumPyDocstring`, `GoogleDocstring`, and `SphinxDocstring`,
/// providing zero-cost access to common docstring elements.
///
/// # Example
///
/// ```rust
/// use pydocstring::traits::DocstringLike;
///
/// fn check_params_documented(doc: &impl DocstringLike) -> Vec<String> {
///     doc.parameters()
///         .iter()
///         .filter(|p| p.description.value.is_empty())
///         .map(|p| p.name.value.to_string())
///         .collect()
/// }
/// ```
pub trait DocstringLike {
    /// Returns the brief summary line.
    fn summary(&self) -> &str;

    /// Returns the extended description, if any.
    fn description(&self) -> Option<&str>;

    /// Returns parameters as style-agnostic views.
    fn parameters(&self) -> Vec<ParameterView<'_>>;

    /// Returns return values as style-agnostic views.
    fn returns(&self) -> Vec<ReturnsView<'_>>;

    /// Returns exceptions as style-agnostic views.
    fn raises(&self) -> Vec<ExceptionView<'_>>;

    /// Returns attributes as style-agnostic views.
    fn attributes(&self) -> Vec<AttributeView<'_>>;
}
