//! # pydocstring
//!
//! A fast Rust parser for Python docstrings supporting NumPy, Google, and Sphinx styles.
//!
//! ## Example
//!
//! ```rust
//! use pydocstring::numpy::parse_numpy;
//!
//! let docstring = r#"
//! Brief description.
//!
//! Parameters
//! ----------
//! x : int
//!     The first parameter.
//! "#;
//!
//! let result = parse_numpy(docstring);
//! assert_eq!(result.value.summary.value, "Brief description.");
//! ```

pub mod ast;
pub mod error;
pub mod parser;
pub mod styles;

pub use ast::{
    AttributeView, Docstring, DocstringLike, ExceptionView, LineIndex, ParameterView, ReturnsView,
    Spanned, Style, TextRange, TextSize,
};
pub use error::{Diagnostic, ParseResult, Severity};
pub use parser::{detect_style, parse};
pub use styles::google::{
    self, GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader,
};
pub use styles::numpy::{
    self, NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod,
    NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader, NumPyWarning, SeeAlsoItem,
};
// Sphinx style: AST types are exported for forward compatibility, but the
// parser is not supported in v1. Calling `parse_sphinx` will return an error
// diagnostic.
pub use styles::sphinx::{
    self, SphinxDocstring, SphinxException, SphinxField, SphinxParameter, SphinxReturns,
    SphinxVariable,
};
