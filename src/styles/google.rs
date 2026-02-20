//! Google-style docstring support.
//!
//! This module contains the AST types and parser for Google-style docstrings.

pub mod ast;
pub mod parser;

pub use ast::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleDocstringItem, GoogleException,
    GoogleMethod, GoogleReturns, GoogleSection, GoogleSectionBody, GoogleSectionHeader,
    GoogleSectionKind, GoogleSeeAlsoItem, GoogleWarning,
};
pub use parser::parse_google;
