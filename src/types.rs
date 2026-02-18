//! Type definitions for parsed docstrings.
//!
//! Each docstring style has its own specific type to accurately represent
//! the features and semantics unique to that style.

pub mod google;
pub mod numpy;
pub mod sphinx;

pub use google::{
    GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleReturns,
};
pub use numpy::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod,
    NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader, NumPyWarning, SeeAlsoItem,
};
pub use sphinx::{
    SphinxDocstring, SphinxException, SphinxField, SphinxParameter, SphinxReturns,
    SphinxVariable,
};

use crate::traits::DocstringLike;
use crate::views::{AttributeView, ExceptionView, ParameterView, ReturnsView};

// =============================================================================
// Unified types
// =============================================================================

/// Docstring style identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// NumPy style (section headers with underlines).
    NumPy,
    /// Google style (section headers with colons).
    Google,
    /// Sphinx style (field lists with `:param:`, `:type:`, etc.).
    Sphinx,
}

/// A parsed docstring of any style.
///
/// Wraps the style-specific types and implements [`DocstringLike`] for
/// unified access. Use pattern matching to access style-specific fields.
///
/// # Example
///
/// ```rust
/// use pydocstring::{parse, Docstring, DocstringLike};
///
/// let doc = parse("Brief summary.").unwrap();
/// assert_eq!(doc.summary(), "Brief summary.");
/// assert_eq!(doc.style(), pydocstring::Style::Google);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Docstring {
    /// NumPy-style docstring.
    NumPy(NumPyDocstring),
    /// Google-style docstring.
    Google(GoogleDocstring),
    /// Sphinx-style docstring.
    Sphinx(SphinxDocstring),
}

impl Docstring {
    /// Returns the detected style.
    pub fn style(&self) -> Style {
        match self {
            Docstring::NumPy(_) => Style::NumPy,
            Docstring::Google(_) => Style::Google,
            Docstring::Sphinx(_) => Style::Sphinx,
        }
    }

    /// Returns a reference to the inner `NumPyDocstring`, if this is NumPy style.
    pub fn as_numpy(&self) -> Option<&NumPyDocstring> {
        match self {
            Docstring::NumPy(d) => Some(d),
            _ => None,
        }
    }

    /// Returns a reference to the inner `GoogleDocstring`, if this is Google style.
    pub fn as_google(&self) -> Option<&GoogleDocstring> {
        match self {
            Docstring::Google(d) => Some(d),
            _ => None,
        }
    }

    /// Returns a reference to the inner `SphinxDocstring`, if this is Sphinx style.
    pub fn as_sphinx(&self) -> Option<&SphinxDocstring> {
        match self {
            Docstring::Sphinx(d) => Some(d),
            _ => None,
        }
    }
}

impl DocstringLike for Docstring {
    fn summary(&self) -> &str {
        match self {
            Docstring::NumPy(d) => d.summary(),
            Docstring::Google(d) => d.summary(),
            Docstring::Sphinx(d) => d.summary(),
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Docstring::NumPy(d) => d.description(),
            Docstring::Google(d) => d.description(),
            Docstring::Sphinx(d) => d.description(),
        }
    }

    fn parameters(&self) -> Vec<ParameterView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::parameters(d),
            Docstring::Google(d) => d.parameters(),
            Docstring::Sphinx(d) => d.parameters(),
        }
    }

    fn returns(&self) -> Vec<ReturnsView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::returns(d),
            Docstring::Google(d) => d.returns(),
            Docstring::Sphinx(d) => d.returns(),
        }
    }

    fn raises(&self) -> Vec<ExceptionView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::raises(d),
            Docstring::Google(d) => d.raises(),
            Docstring::Sphinx(d) => d.raises(),
        }
    }

    fn attributes(&self) -> Vec<AttributeView<'_>> {
        match self {
            Docstring::NumPy(d) => DocstringLike::attributes(d),
            Docstring::Google(d) => d.attributes(),
            Docstring::Sphinx(d) => d.attributes(),
        }
    }
}

impl core::fmt::Display for Style {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Style::NumPy => write!(f, "numpy"),
            Style::Google => write!(f, "google"),
            Style::Sphinx => write!(f, "sphinx"),
        }
    }
}
