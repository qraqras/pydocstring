//! Error types for docstring parsing.

use core::fmt;

/// Errors that can occur during docstring parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Invalid format in the docstring.
    InvalidFormat(String),
    /// Unexpected end of input.
    UnexpectedEnd,
    /// Missing required section.
    MissingSection(String),
    /// Invalid parameter format.
    InvalidParameter(String),
    /// Invalid return format.
    InvalidReturn(String),
    /// Generic parsing error.
    ParseError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            ParseError::MissingSection(section) => write!(f, "Missing section: {}", section),
            ParseError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            ParseError::InvalidReturn(msg) => write!(f, "Invalid return: {}", msg),
            ParseError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;
