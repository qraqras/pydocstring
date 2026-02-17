//! Borrowed view types for zero-copy access across docstring styles.
//!
//! These types provide a unified, style-agnostic view into parsed docstrings
//! without copying data. They are returned by [`DocstringLike`](crate::traits::DocstringLike)
//! trait methods.

use crate::span::Span;

/// A borrowed view of a parameter (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterView<'a> {
    /// Parameter name.
    pub name: &'a str,
    /// Parameter type annotation.
    pub param_type: Option<&'a str>,
    /// Parameter description.
    pub description: &'a str,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Source span of the parameter definition.
    pub span: Span,
}

/// A borrowed view of a return value (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnsView<'a> {
    /// Return value name (if named).
    pub name: Option<&'a str>,
    /// Return type annotation.
    pub return_type: Option<&'a str>,
    /// Description of the return value.
    pub description: &'a str,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an exception (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionView<'a> {
    /// Exception type name.
    pub exception_type: &'a str,
    /// Description of when the exception is raised.
    pub description: &'a str,
    /// Source span.
    pub span: Span,
}

/// A borrowed view of an attribute (style-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeView<'a> {
    /// Attribute name.
    pub name: &'a str,
    /// Attribute type annotation.
    pub attr_type: Option<&'a str>,
    /// Description.
    pub description: &'a str,
    /// Source span.
    pub span: Span,
}
