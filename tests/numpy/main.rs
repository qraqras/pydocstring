//! Integration tests for NumPy-style docstring parser.

pub use pydocstring::NumPySectionBody;
pub use pydocstring::NumPySectionKind;
pub use pydocstring::TextSize;
pub use pydocstring::numpy::parse_numpy;
pub use pydocstring::numpy::{
    NumPyAttribute, NumPyDocstring, NumPyDocstringItem, NumPyException, NumPyMethod,
    NumPyParameter, NumPyReference, NumPyReturns, NumPySection, NumPyWarning, SeeAlsoItem,
};

mod edge_cases;
mod freetext;
mod parameters;
mod raises;
mod returns;
mod sections;
mod structured;
mod summary;

// =============================================================================
// Shared helpers
// =============================================================================

/// Extract all sections from a docstring, ignoring stray lines.
pub fn sections(doc: &NumPyDocstring) -> Vec<&NumPySection> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            NumPyDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .collect()
}

pub fn parameters(doc: &NumPyDocstring) -> Vec<&NumPyParameter> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Parameters(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn returns(doc: &NumPyDocstring) -> Vec<&NumPyReturns> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Returns(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn raises(doc: &NumPyDocstring) -> Vec<&NumPyException> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Raises(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn warns(doc: &NumPyDocstring) -> Vec<&NumPyWarning> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Warns(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn see_also(doc: &NumPyDocstring) -> Vec<&SeeAlsoItem> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::SeeAlso(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn references(doc: &NumPyDocstring) -> Vec<&NumPyReference> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::References(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn notes(doc: &NumPyDocstring) -> Option<&pydocstring::TextRange> {
    sections(doc).iter().find_map(|s| match &s.body {
        NumPySectionBody::Notes(v) => v.as_ref(),
        _ => None,
    })
}

pub fn examples(doc: &NumPyDocstring) -> Option<&pydocstring::TextRange> {
    sections(doc).iter().find_map(|s| match &s.body {
        NumPySectionBody::Examples(v) => v.as_ref(),
        _ => None,
    })
}

pub fn yields(doc: &NumPyDocstring) -> Vec<&NumPyReturns> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Yields(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn receives(doc: &NumPyDocstring) -> Vec<&NumPyParameter> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Receives(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn other_parameters(doc: &NumPyDocstring) -> Vec<&NumPyParameter> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::OtherParameters(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn attributes(doc: &NumPyDocstring) -> Vec<&NumPyAttribute> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Attributes(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn methods(doc: &NumPyDocstring) -> Vec<&NumPyMethod> {
    sections(doc)
        .iter()
        .filter_map(|s| match &s.body {
            NumPySectionBody::Methods(v) => Some(v.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn warnings_text(doc: &NumPyDocstring) -> Option<&pydocstring::TextRange> {
    sections(doc).iter().find_map(|s| match &s.body {
        NumPySectionBody::Warnings(v) => v.as_ref(),
        _ => None,
    })
}

pub fn unknown_text(doc: &NumPyDocstring) -> Option<&pydocstring::TextRange> {
    sections(doc).iter().find_map(|s| match &s.body {
        NumPySectionBody::Unknown(v) => v.as_ref(),
        _ => None,
    })
}
