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
//! let result = parse_numpy(docstring).unwrap();
//! assert_eq!(result.summary.value, "Brief description.");
//! ```

pub mod ast;
pub mod error;
pub mod google;
pub mod numpy;
pub mod parser;
pub mod sphinx;

pub use ast::{
    AttributeView, Docstring, DocstringLike, ExceptionView, ParameterView, Position, ReturnsView,
    Span, Spanned, Style,
};
pub use error::ParseError;
pub use google::{
    GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleReturns,
};
pub use numpy::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod, NumPyParameter,
    NumPyReference, NumPyReturns, NumPySection, NumPySectionBody, NumPySectionHeader, NumPyWarning,
    SeeAlsoItem,
};
pub use parser::{detect_style, parse};
pub use sphinx::{
    SphinxDocstring, SphinxException, SphinxField, SphinxParameter, SphinxReturns, SphinxVariable,
};
