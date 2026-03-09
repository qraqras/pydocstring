//! Emit (code generation) from the style-independent document model.
//!
//! Each sub-module converts a [`Docstring`](crate::model::Docstring) into a
//! formatted string for a particular docstring style.

pub mod google;
pub mod numpy;
