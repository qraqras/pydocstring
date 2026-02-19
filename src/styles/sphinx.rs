//! Sphinx-style docstring support.
//!
//! **Note:** Sphinx style is not supported in v1. The AST types are defined
//! for forward compatibility, but the parser only extracts the summary line
//! and emits an error diagnostic. Full support is planned for a future release.

pub mod ast;
pub mod parser;

pub use ast::{
    SphinxDocstring, SphinxException, SphinxField, SphinxParameter, SphinxReturns, SphinxVariable,
};
pub use parser::parse_sphinx;
