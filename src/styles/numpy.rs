//! NumPy-style docstring support.
//!
//! This module contains the AST types and parser for NumPy-style docstrings.

pub mod ast;
pub mod parser;

pub use ast::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyDocstringItem, NumPyException,
    NumPyMethod, NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPySectionBody,
    NumPySectionHeader, NumPySectionKind, NumPyWarning, SeeAlsoItem,
};
pub use parser::parse_numpy;
