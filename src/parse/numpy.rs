//! NumPy-style docstring support.
//!
//! This module contains the AST types and parser for NumPy-style docstrings.

pub mod kind;
pub mod nodes;
pub mod parser;
pub mod to_model;

pub use kind::NumPySectionKind;
pub use nodes::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod, NumPyParameter, NumPyReference,
    NumPyReturns, NumPySection, NumPySectionHeader, NumPySeeAlsoItem, NumPyWarning,
};
pub use parser::parse_numpy;
