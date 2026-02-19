//! Diagnostic types for docstring parsing.
//!
//! This module provides a diagnostic-based result type. Parsers always return
//! a value (partial results) together with zero or more diagnostics, instead
//! of failing with an error.

use core::fmt;

use crate::ast::Span;

// =============================================================================
// Diagnostic
// =============================================================================

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The parser could not recover meaningful structure for a region.
    Error,
    /// The input is parseable but likely incorrect.
    Warning,
    /// Stylistic or informational hint.
    Hint,
}

/// A diagnostic produced during parsing.
///
/// Each diagnostic points to a source span and carries a human-readable message.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Where the problem was detected.
    pub span: Span,
    /// How severe the problem is.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
}

impl Diagnostic {
    /// Creates a new diagnostic.
    pub fn new(span: Span, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            span,
            severity,
            message: message.into(),
        }
    }

    /// Shorthand for creating an error diagnostic.
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self::new(span, Severity::Error, message)
    }

    /// Shorthand for creating a warning diagnostic.
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self::new(span, Severity::Warning, message)
    }

    /// Shorthand for creating a hint diagnostic.
    pub fn hint(span: Span, message: impl Into<String>) -> Self {
        Self::new(span, Severity::Hint, message)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        };
        write!(
            f,
            "{} at {}:{}: {}",
            sev, self.span.start.line, self.span.start.column, self.message
        )
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

// =============================================================================
// ParseResult
// =============================================================================

/// Result of parsing a docstring.
///
/// Always contains a value — even when the input is malformed the parser will
/// produce a best-effort AST. Problems detected during parsing are collected
/// in `diagnostics`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult<T> {
    /// The parsed value (always present).
    pub value: T,
    /// Diagnostics collected during parsing (may be empty).
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> ParseResult<T> {
    /// Creates a parse result with no diagnostics.
    pub fn ok(value: T) -> Self {
        Self {
            value,
            diagnostics: Vec::new(),
        }
    }

    /// Creates a parse result with diagnostics.
    pub fn with_diagnostics(value: T, diagnostics: Vec<Diagnostic>) -> Self {
        Self { value, diagnostics }
    }

    /// Returns `true` if there are any error-level diagnostics.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Returns `true` if there are any warning-level or error-level diagnostics.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error | Severity::Warning))
    }

    /// Returns an iterator over error-level diagnostics.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }

    /// Returns an iterator over warning-level diagnostics.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }

    /// Discards diagnostics and returns the inner value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Transforms the inner value, preserving diagnostics.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ParseResult<U> {
        ParseResult {
            value: f(self.value),
            diagnostics: self.diagnostics,
        }
    }
}
