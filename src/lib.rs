//! # pydocstring
//!
//! A fast Rust parser for Python docstrings supporting NumPy and Google styles.
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

pub use ast::{LineIndex, Spanned, Style, TextRange, TextSize};
pub use error::{Diagnostic, ParseResult, Severity};
pub use parser::detect_style;
pub use styles::google::{
    self, GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleMethod,
    GoogleReturns, GoogleSection, GoogleSectionBody, GoogleSectionHeader, GoogleSeeAlsoItem,
    GoogleWarning,
};
pub use styles::numpy::{
    self, NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod,
    NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader, NumPyWarning, SeeAlsoItem,
};
