//! NumPy-style docstring support.
//!
//! This module contains the AST types and parser for NumPy-style docstrings.

pub mod kind;
pub mod nodes;
pub mod parser;

pub use kind::NumPySectionKind;
pub use parser::parse_numpy;
