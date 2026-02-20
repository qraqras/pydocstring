//! # pydocstring
//!
//! A fast, zero-dependency Rust parser for Python docstrings with full AST and
//! source location tracking. Supports **NumPy** and **Google** styles.
//!
//! ## Quick Start
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
//!
//! ## Style Auto-Detection
//!
//! ```rust
//! use pydocstring::{detect_style, Style};
//!
//! let numpy_doc = "Summary.\n\nParameters\n----------\nx : int\n    Desc.";
//! assert_eq!(detect_style(numpy_doc), Style::NumPy);
//!
//! let google_doc = "Summary.\n\nArgs:\n    x: Desc.";
//! assert_eq!(detect_style(google_doc), Style::Google);
//! ```
//!
//! ## Features
//!
//! - Zero external dependencies — pure Rust
//! - Accurate source spans (byte offsets) on every AST node
//! - Diagnostic-based error reporting (partial results + diagnostics)
//! - NumPy style: fully supported
//! - Google style: fully supported

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
