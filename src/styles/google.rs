//! Google-style docstring support.
//!
//! This module contains the AST types and parser for Google-style docstrings.

pub mod ast;
pub mod parser;

pub use ast::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleException, GoogleMethod, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader, GoogleSeeAlsoItem, GoogleWarning,
};
pub use parser::parse_google;
