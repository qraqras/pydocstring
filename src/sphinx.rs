//! Sphinx-style docstring support.
//!
//! This module contains the AST types and parser for Sphinx-style docstrings.

pub mod ast;
pub mod parser;

pub use ast::{
    SphinxDocstring, SphinxException, SphinxField, SphinxParameter, SphinxReturns, SphinxVariable,
};
pub use parser::parse_sphinx;
