//! Integration tests for Google-style docstring parser.

pub use pydocstring::GoogleSectionBody;
pub use pydocstring::google::parse_google;
pub use pydocstring::google::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleDocstringItem, GoogleException,
    GoogleMethod, GoogleReturns, GoogleSection, GoogleSeeAlsoItem, GoogleWarning,
};
pub use pydocstring::{LineIndex, TextSize};

mod args;
mod edge_cases;
mod freetext;
mod raises;
mod returns;
mod sections;
mod structured;
mod summary;

// =============================================================================
// Shared helpers
// =============================================================================

pub fn all_sections(doc: &GoogleDocstring) -> Vec<&GoogleSection> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .collect()
}

pub fn args(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Args(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn returns(doc: &GoogleDocstring) -> Option<&GoogleReturns> {
    doc.items.iter().find_map(|item| match item {
        GoogleDocstringItem::Section(s) => match &s.body {
            GoogleSectionBody::Returns(r) => Some(r),
            _ => None,
        },
        _ => None,
    })
}

pub fn yields(doc: &GoogleDocstring) -> Option<&GoogleReturns> {
    doc.items.iter().find_map(|item| match item {
        GoogleDocstringItem::Section(s) => match &s.body {
            GoogleSectionBody::Yields(r) => Some(r),
            _ => None,
        },
        _ => None,
    })
}

pub fn raises(doc: &GoogleDocstring) -> Vec<&GoogleException> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Raises(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn attributes(doc: &GoogleDocstring) -> Vec<&GoogleAttribute> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Attributes(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn keyword_args(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::KeywordArgs(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn other_parameters(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::OtherParameters(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn receives(doc: &GoogleDocstring) -> Vec<&GoogleArg> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Receives(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn warns(doc: &GoogleDocstring) -> Vec<&GoogleWarning> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Warns(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn see_also(doc: &GoogleDocstring) -> Vec<&GoogleSeeAlsoItem> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::SeeAlso(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn methods(doc: &GoogleDocstring) -> Vec<&GoogleMethod> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => match &s.body {
                GoogleSectionBody::Methods(v) => Some(v.iter()),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn notes(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Notes(v) => Some(v),
            _ => None,
        })
}

pub fn examples(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Examples(v) => Some(v),
            _ => None,
        })
}

pub fn todo(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Todo(v) => Some(v),
            _ => None,
        })
}

pub fn references(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::References(v) => Some(v),
            _ => None,
        })
}

pub fn warnings(doc: &GoogleDocstring) -> Option<&pydocstring::TextRange> {
    doc.items
        .iter()
        .filter_map(|item| match item {
            GoogleDocstringItem::Section(s) => Some(s),
            _ => None,
        })
        .find_map(|s| match &s.body {
            GoogleSectionBody::Warnings(v) => Some(v),
            _ => None,
        })
}
