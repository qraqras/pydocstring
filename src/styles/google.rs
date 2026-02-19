//! Google-style docstring support.
//!
//! This module contains the AST types and parser for Google-style docstrings.

pub mod ast;
pub mod parser;

pub use ast::{
    GoogleArgument, GoogleAttribute, GoogleDocstring, GoogleException, GoogleReturns,
    GoogleSection, GoogleSectionBody, GoogleSectionHeader,
};
pub use parser::parse_google;
