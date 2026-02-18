//! Borrowed view types for zero-copy access across docstring styles.
//!
//! These types provide a unified, style-agnostic view into parsed docstrings
//! without copying data. They are returned by [`DocstringLike`](crate::traits::DocstringLike)
//! trait methods.

use crate::span::{Span, Spanned};

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
