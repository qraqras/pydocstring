//! Style-agnostic traits for docstring access.
//!
//! These traits provide a unified interface over different docstring styles,
//! enabling linters and formatters to operate without knowing the specific style.
//! Inspired by Biome's approach to multi-language support.

use crate::views::{AttributeView, ExceptionView, ParameterView, ReturnsView};

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
///         .filter(|p| p.description.is_empty())
///         .map(|p| p.name.to_string())
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
