//! # pydocstring
//!
//! A fast Rust parser for Python docstrings supporting NumPy, Google, and Sphinx styles.
//!
//! ## Example
//!
//! ```rust
//! use pydocstring::parser::numpy::parse_numpy;
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
//! assert_eq!(result.summary, "Brief description.");
//! ```

pub mod error;
pub mod parser;
pub mod span;
pub mod traits;
pub mod types;
pub mod views;

pub use error::ParseError;
pub use parser::{detect_style, parse};
pub use span::{Position, Span};
pub use traits::DocstringLike;
pub use types::{
    Docstring,
    // Google types
    GoogleArgument,
    GoogleAttribute,
    GoogleDocstring,
    GoogleException,
    GoogleReturns,
    NumPyAttribute,
    NumPyDeprecation,
    NumPyDocstring,
    NumPyException,
    NumPyMethod,
    // NumPy types
    NumPyParameter,
    NumPyReference,
    NumPyReturns,
    NumPyWarning,
    SeeAlsoItem,
    SphinxDocstring,
    SphinxException,
    SphinxField,
    // Sphinx types
    SphinxParameter,
    SphinxReturns,
    SphinxVariable,
    Style,
};
pub use views::{AttributeView, ExceptionView, ParameterView, ReturnsView};
