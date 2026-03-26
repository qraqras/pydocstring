//! Unified typed visitor for all docstring ASTs.
//!
//! A single [`DocstringVisitor`] trait covers both Google-style and NumPy-style
//! nodes.  Call a `visit_*` method directly to start traversal, or call
//! [`walk_node`] from within an override to continue into children.
//!
//! Traversal follows the same protocol as `ast.NodeVisitor` in Python:
//! - Each `visit_*` method's **default implementation calls [`walk_node`]**,
//!   which visits the node's children.
//! - Override a method to add behaviour *before* and/or *after* children are
//!   visited by explicitly calling `walk_node`, or omit the call to prune the
//!   subtree entirely.
//!
//! # Example
//!
//! ```rust
//! use pydocstring::parse::google::{parse_google, GoogleSection};
//! use pydocstring::parse::visitor::{DocstringVisitor, walk_node};
//!
//! struct SectionPrinter;
//!
//! impl DocstringVisitor for SectionPrinter {
//!     type Error = std::convert::Infallible;
//!
//!     fn visit_google_section(&mut self, source: &str, section: &GoogleSection<'_>) -> Result<(), Self::Error> {
//!         println!("enter: {}", section.header().name().text(source));
//!         walk_node(source, section.syntax(), self)?; // visit children
//!         println!("leave: {}", section.header().name().text(source));
//!         Ok(())
//!     }
//! }
//!
//! let result = parse_google("Args:\n    x: desc\n");
//! let doc = pydocstring::parse::google::GoogleDocstring::cast(result.root()).unwrap();
//! let mut printer = SectionPrinter;
//! printer.visit_google_docstring(result.source(), &doc).unwrap();
//! ```

use crate::parse::google::nodes::{
    GoogleArg, GoogleAttribute, GoogleDocstring, GoogleException, GoogleMethod, GoogleReturn, GoogleSection,
    GoogleSeeAlsoItem, GoogleWarning, GoogleYield,
};
use crate::parse::numpy::nodes::{
    NumPyAttribute, NumPyDeprecation, NumPyDocstring, NumPyException, NumPyMethod, NumPyParameter, NumPyReference,
    NumPyReturns, NumPySection, NumPySeeAlsoItem, NumPyWarning, NumPyYields,
};
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

/// Unified typed visitor for Google-style and NumPy-style docstring ASTs.
///
/// Each `visit_*` method's default implementation calls [`walk_node`] to
/// continue into children.  Override a method and call [`walk_node`] explicitly
/// to add pre/post logic, or omit the call to prune that subtree.
///
/// The `source` parameter is the original docstring source text, required for
/// reading token text (e.g. `arg.name().text(source)`).
///
/// `type Error` is the error type returned by all `visit_*` methods.  Use
/// [`std::convert::Infallible`] for infallible visitors.
pub trait DocstringVisitor: Sized {
    /// The error type returned by visitor methods.
    type Error;

    // ── Google ────────────────────────────────────────────────────────────
    /// Called for the Google docstring root.
    fn visit_google_docstring(&mut self, source: &str, doc: &GoogleDocstring<'_>) -> Result<(), Self::Error> {
        walk_node(source, doc.syntax(), self)
    }
    /// Called for each Google section.
    fn visit_google_section(&mut self, source: &str, sec: &GoogleSection<'_>) -> Result<(), Self::Error> {
        walk_node(source, sec.syntax(), self)
    }
    /// Called for each argument entry.
    fn visit_google_arg(&mut self, source: &str, arg: &GoogleArg<'_>) -> Result<(), Self::Error> {
        walk_node(source, arg.syntax(), self)
    }
    /// Called for the Return entry in a Returns section, if present.
    fn visit_google_return(&mut self, source: &str, rtn: &GoogleReturn<'_>) -> Result<(), Self::Error> {
        walk_node(source, rtn.syntax(), self)
    }
    /// Called for the Yield entry in a Yields section, if present.
    fn visit_google_yield(&mut self, source: &str, yld: &GoogleYield<'_>) -> Result<(), Self::Error> {
        walk_node(source, yld.syntax(), self)
    }
    /// Called for each exception entry.
    fn visit_google_exception(&mut self, source: &str, exc: &GoogleException<'_>) -> Result<(), Self::Error> {
        walk_node(source, exc.syntax(), self)
    }
    /// Called for each warning entry.
    fn visit_google_warning(&mut self, source: &str, wrn: &GoogleWarning<'_>) -> Result<(), Self::Error> {
        walk_node(source, wrn.syntax(), self)
    }
    /// Called for each See Also item.
    fn visit_google_see_also_item(&mut self, source: &str, sai: &GoogleSeeAlsoItem<'_>) -> Result<(), Self::Error> {
        walk_node(source, sai.syntax(), self)
    }
    /// Called for each attribute entry.
    fn visit_google_attribute(&mut self, source: &str, att: &GoogleAttribute<'_>) -> Result<(), Self::Error> {
        walk_node(source, att.syntax(), self)
    }
    /// Called for each method entry.
    fn visit_google_method(&mut self, source: &str, mtd: &GoogleMethod<'_>) -> Result<(), Self::Error> {
        walk_node(source, mtd.syntax(), self)
    }
    // ── NumPy ─────────────────────────────────────────────────────────────
    /// Called for the NumPy docstring root.
    fn visit_numpy_docstring(&mut self, source: &str, doc: &NumPyDocstring<'_>) -> Result<(), Self::Error> {
        walk_node(source, doc.syntax(), self)
    }
    /// Called for the deprecation notice, if present.
    fn visit_numpy_deprecation(&mut self, source: &str, dep: &NumPyDeprecation<'_>) -> Result<(), Self::Error> {
        walk_node(source, dep.syntax(), self)
    }
    /// Called for each NumPy section.
    fn visit_numpy_section(&mut self, source: &str, sec: &NumPySection<'_>) -> Result<(), Self::Error> {
        walk_node(source, sec.syntax(), self)
    }
    /// Called for each parameter entry.
    fn visit_numpy_parameter(&mut self, source: &str, prm: &NumPyParameter<'_>) -> Result<(), Self::Error> {
        walk_node(source, prm.syntax(), self)
    }
    /// Called for each Returns entry.
    fn visit_numpy_returns(&mut self, source: &str, rtn: &NumPyReturns<'_>) -> Result<(), Self::Error> {
        walk_node(source, rtn.syntax(), self)
    }
    /// Called for each Yields entry.
    fn visit_numpy_yields(&mut self, source: &str, yld: &NumPyYields<'_>) -> Result<(), Self::Error> {
        walk_node(source, yld.syntax(), self)
    }
    /// Called for each exception entry.
    fn visit_numpy_exception(&mut self, source: &str, exc: &NumPyException<'_>) -> Result<(), Self::Error> {
        walk_node(source, exc.syntax(), self)
    }
    /// Called for each warning entry.
    fn visit_numpy_warning(&mut self, source: &str, wrn: &NumPyWarning<'_>) -> Result<(), Self::Error> {
        walk_node(source, wrn.syntax(), self)
    }
    /// Called for each See Also item.
    fn visit_numpy_see_also_item(&mut self, source: &str, sai: &NumPySeeAlsoItem<'_>) -> Result<(), Self::Error> {
        walk_node(source, sai.syntax(), self)
    }
    /// Called for each reference entry.
    fn visit_numpy_reference(&mut self, source: &str, r#ref: &NumPyReference<'_>) -> Result<(), Self::Error> {
        walk_node(source, r#ref.syntax(), self)
    }
    /// Called for each attribute entry.
    fn visit_numpy_attribute(&mut self, source: &str, att: &NumPyAttribute<'_>) -> Result<(), Self::Error> {
        walk_node(source, att.syntax(), self)
    }
    /// Called for each method entry.
    fn visit_numpy_method(&mut self, source: &str, mtd: &NumPyMethod<'_>) -> Result<(), Self::Error> {
        walk_node(source, mtd.syntax(), self)
    }
}

/// Traverse the children of any docstring node, dispatching each recognised
/// child to the corresponding `visit_*` method.
///
/// Call this from a `visit_*` override to continue into children.
pub fn walk_node<V: DocstringVisitor>(source: &str, node: &SyntaxNode, visitor: &mut V) -> Result<(), V::Error> {
    for child in node.children() {
        match child {
            SyntaxElement::Token(_) => continue, // ignore tokens by default
            SyntaxElement::Node(n) => match n.kind() {
                // Google
                SyntaxKind::GOOGLE_SECTION => visitor.visit_google_section(source, &GoogleSection(n))?,
                SyntaxKind::GOOGLE_ARG => visitor.visit_google_arg(source, &GoogleArg(n))?,
                SyntaxKind::GOOGLE_RETURNS => visitor.visit_google_return(source, &GoogleReturn(n))?,
                SyntaxKind::GOOGLE_YIELDS => visitor.visit_google_yield(source, &GoogleYield(n))?,
                SyntaxKind::GOOGLE_EXCEPTION => visitor.visit_google_exception(source, &GoogleException(n))?,
                SyntaxKind::GOOGLE_WARNING => visitor.visit_google_warning(source, &GoogleWarning(n))?,
                SyntaxKind::GOOGLE_SEE_ALSO_ITEM => {
                    visitor.visit_google_see_also_item(source, &GoogleSeeAlsoItem(n))?
                }
                SyntaxKind::GOOGLE_ATTRIBUTE => visitor.visit_google_attribute(source, &GoogleAttribute(n))?,
                SyntaxKind::GOOGLE_METHOD => visitor.visit_google_method(source, &GoogleMethod(n))?,
                // NumPy
                SyntaxKind::NUMPY_SECTION => visitor.visit_numpy_section(source, &NumPySection(n))?,
                SyntaxKind::NUMPY_DEPRECATION => visitor.visit_numpy_deprecation(source, &NumPyDeprecation(n))?,
                SyntaxKind::NUMPY_PARAMETER => visitor.visit_numpy_parameter(source, &NumPyParameter(n))?,
                SyntaxKind::NUMPY_RETURNS => visitor.visit_numpy_returns(source, &NumPyReturns(n))?,
                SyntaxKind::NUMPY_YIELDS => visitor.visit_numpy_yields(source, &NumPyYields(n))?,
                SyntaxKind::NUMPY_EXCEPTION => visitor.visit_numpy_exception(source, &NumPyException(n))?,
                SyntaxKind::NUMPY_WARNING => visitor.visit_numpy_warning(source, &NumPyWarning(n))?,
                SyntaxKind::NUMPY_SEE_ALSO_ITEM => visitor.visit_numpy_see_also_item(source, &NumPySeeAlsoItem(n))?,
                SyntaxKind::NUMPY_REFERENCE => visitor.visit_numpy_reference(source, &NumPyReference(n))?,
                SyntaxKind::NUMPY_ATTRIBUTE => visitor.visit_numpy_attribute(source, &NumPyAttribute(n))?,
                SyntaxKind::NUMPY_METHOD => visitor.visit_numpy_method(source, &NumPyMethod(n))?,
                // Other
                _ => {}
            },
        }
    }
    Ok(())
}
