use pyo3::prelude::*;

use pydocstring_core::model;
use pydocstring_core::parse::google;
use pydocstring_core::parse::google::kind::GoogleSectionKind;
use pydocstring_core::parse::google::nodes as gn;
use pydocstring_core::parse::numpy::kind::NumPySectionKind;
use pydocstring_core::parse::numpy::nodes as nn;
use pydocstring_core::syntax::{Parsed, SyntaxKind, SyntaxNode, SyntaxToken};
use pydocstring_core::text::TextRange;

// ─── TextRange ──────────────────────────────────────────────────────────────

#[pyclass(name = "TextRange", frozen)]
struct PyTextRange {
    start: u32,
    end: u32,
}

#[pymethods]
impl PyTextRange {
    #[getter]
    fn start(&self) -> u32 {
        self.start
    }
    #[getter]
    fn end(&self) -> u32 {
        self.end
    }
    fn __repr__(&self) -> String {
        format!("TextRange({}..{})", self.start, self.end)
    }
}

impl From<&TextRange> for PyTextRange {
    fn from(r: &TextRange) -> Self {
        Self {
            start: r.start().raw(),
            end: r.end().raw(),
        }
    }
}

// ─── LineColumn ─────────────────────────────────────────────────────────────

/// A line/column position in the source text.
///
/// `lineno` is 1-based; `col` is the 0-based Unicode codepoint offset from
/// the start of the line (compatible with Python's `ast` module).
#[pyclass(name = "LineColumn", frozen)]
struct PyLineColumn {
    #[pyo3(get)]
    lineno: u32,
    #[pyo3(get)]
    col: u32,
}

#[pymethods]
impl PyLineColumn {
    fn __repr__(&self) -> String {
        format!("LineColumn(lineno={}, col={})", self.lineno, self.col)
    }
}

/// Convert a byte offset into a `PyLineColumn` with codepoint-based `col`.
///
/// Returns an error if `byte_offset` is beyond the source length or does not
/// land on a UTF-8 character boundary.
fn byte_offset_to_line_col(source: &str, byte_offset: usize) -> PyResult<PyLineColumn> {
    if byte_offset > source.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "offset {} is out of bounds (source length: {})",
            byte_offset,
            source.len()
        )));
    }
    let mut lineno = 1u32;
    let mut line_start = 0usize;
    for (i, b) in source.bytes().enumerate() {
        if i >= byte_offset {
            break;
        }
        if b == b'\n' {
            lineno += 1;
            line_start = i + 1;
        }
    }
    // Verify the offset falls on a char boundary before slicing.
    if !source.is_char_boundary(byte_offset) || !source.is_char_boundary(line_start) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "offset is not on a UTF-8 character boundary",
        ));
    }
    let col = source[line_start..byte_offset].chars().count() as u32;
    Ok(PyLineColumn { lineno, col })
}

// ─── SyntaxKind ──────────────────────────────────────────────────────────────

/// Syntax node/token kind enum.
#[pyclass(eq, eq_int, frozen, hash, name = "SyntaxKind")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
enum PySyntaxKind {
    // Common tokens
    NAME,
    TYPE,
    COLON,
    DESCRIPTION,
    OPEN_BRACKET,
    CLOSE_BRACKET,
    OPTIONAL,
    BODY_TEXT,
    SUMMARY,
    EXTENDED_SUMMARY,
    STRAY_LINE,
    // Google-specific tokens
    WARNING_TYPE,
    // NumPy-specific tokens
    UNDERLINE,
    DIRECTIVE_MARKER,
    KEYWORD,
    DOUBLE_COLON,
    VERSION,
    RETURN_TYPE,
    DEFAULT_KEYWORD,
    DEFAULT_SEPARATOR,
    DEFAULT_VALUE,
    NUMBER,
    CONTENT,
    // Google nodes
    GOOGLE_DOCSTRING,
    GOOGLE_SECTION,
    GOOGLE_SECTION_HEADER,
    GOOGLE_ARG,
    GOOGLE_RETURNS,
    GOOGLE_YIELDS,
    GOOGLE_EXCEPTION,
    GOOGLE_WARNING,
    GOOGLE_SEE_ALSO_ITEM,
    GOOGLE_ATTRIBUTE,
    GOOGLE_METHOD,
    // NumPy nodes
    NUMPY_DOCSTRING,
    NUMPY_SECTION,
    NUMPY_SECTION_HEADER,
    NUMPY_DEPRECATION,
    NUMPY_PARAMETER,
    NUMPY_RETURNS,
    NUMPY_YIELDS,
    NUMPY_EXCEPTION,
    NUMPY_WARNING,
    NUMPY_SEE_ALSO_ITEM,
    NUMPY_REFERENCE,
    NUMPY_ATTRIBUTE,
    NUMPY_METHOD,
    // Plain node
    PLAIN_DOCSTRING,
}

#[pymethods]
impl PySyntaxKind {
    fn __repr__(&self) -> String {
        format!("SyntaxKind.{}", self.to_core().name())
    }

    fn __str__(&self) -> &'static str {
        self.to_core().name()
    }
}

impl PySyntaxKind {
    fn from_core(kind: SyntaxKind) -> Self {
        match kind {
            SyntaxKind::NAME => Self::NAME,
            SyntaxKind::TYPE => Self::TYPE,
            SyntaxKind::COLON => Self::COLON,
            SyntaxKind::DESCRIPTION => Self::DESCRIPTION,
            SyntaxKind::OPEN_BRACKET => Self::OPEN_BRACKET,
            SyntaxKind::CLOSE_BRACKET => Self::CLOSE_BRACKET,
            SyntaxKind::OPTIONAL => Self::OPTIONAL,
            SyntaxKind::BODY_TEXT => Self::BODY_TEXT,
            SyntaxKind::SUMMARY => Self::SUMMARY,
            SyntaxKind::EXTENDED_SUMMARY => Self::EXTENDED_SUMMARY,
            SyntaxKind::STRAY_LINE => Self::STRAY_LINE,
            SyntaxKind::WARNING_TYPE => Self::WARNING_TYPE,
            SyntaxKind::UNDERLINE => Self::UNDERLINE,
            SyntaxKind::DIRECTIVE_MARKER => Self::DIRECTIVE_MARKER,
            SyntaxKind::KEYWORD => Self::KEYWORD,
            SyntaxKind::DOUBLE_COLON => Self::DOUBLE_COLON,
            SyntaxKind::VERSION => Self::VERSION,
            SyntaxKind::RETURN_TYPE => Self::RETURN_TYPE,
            SyntaxKind::DEFAULT_KEYWORD => Self::DEFAULT_KEYWORD,
            SyntaxKind::DEFAULT_SEPARATOR => Self::DEFAULT_SEPARATOR,
            SyntaxKind::DEFAULT_VALUE => Self::DEFAULT_VALUE,
            SyntaxKind::NUMBER => Self::NUMBER,
            SyntaxKind::CONTENT => Self::CONTENT,
            SyntaxKind::GOOGLE_DOCSTRING => Self::GOOGLE_DOCSTRING,
            SyntaxKind::GOOGLE_SECTION => Self::GOOGLE_SECTION,
            SyntaxKind::GOOGLE_SECTION_HEADER => Self::GOOGLE_SECTION_HEADER,
            SyntaxKind::GOOGLE_ARG => Self::GOOGLE_ARG,
            SyntaxKind::GOOGLE_RETURNS => Self::GOOGLE_RETURNS,
            SyntaxKind::GOOGLE_YIELDS => Self::GOOGLE_YIELDS,
            SyntaxKind::GOOGLE_EXCEPTION => Self::GOOGLE_EXCEPTION,
            SyntaxKind::GOOGLE_WARNING => Self::GOOGLE_WARNING,
            SyntaxKind::GOOGLE_SEE_ALSO_ITEM => Self::GOOGLE_SEE_ALSO_ITEM,
            SyntaxKind::GOOGLE_ATTRIBUTE => Self::GOOGLE_ATTRIBUTE,
            SyntaxKind::GOOGLE_METHOD => Self::GOOGLE_METHOD,
            SyntaxKind::NUMPY_DOCSTRING => Self::NUMPY_DOCSTRING,
            SyntaxKind::NUMPY_SECTION => Self::NUMPY_SECTION,
            SyntaxKind::NUMPY_SECTION_HEADER => Self::NUMPY_SECTION_HEADER,
            SyntaxKind::NUMPY_DEPRECATION => Self::NUMPY_DEPRECATION,
            SyntaxKind::NUMPY_PARAMETER => Self::NUMPY_PARAMETER,
            SyntaxKind::NUMPY_RETURNS => Self::NUMPY_RETURNS,
            SyntaxKind::NUMPY_YIELDS => Self::NUMPY_YIELDS,
            SyntaxKind::NUMPY_EXCEPTION => Self::NUMPY_EXCEPTION,
            SyntaxKind::NUMPY_WARNING => Self::NUMPY_WARNING,
            SyntaxKind::NUMPY_SEE_ALSO_ITEM => Self::NUMPY_SEE_ALSO_ITEM,
            SyntaxKind::NUMPY_REFERENCE => Self::NUMPY_REFERENCE,
            SyntaxKind::NUMPY_ATTRIBUTE => Self::NUMPY_ATTRIBUTE,
            SyntaxKind::NUMPY_METHOD => Self::NUMPY_METHOD,
            SyntaxKind::PLAIN_DOCSTRING => Self::PLAIN_DOCSTRING,
        }
    }

    fn to_core(self) -> SyntaxKind {
        match self {
            Self::NAME => SyntaxKind::NAME,
            Self::TYPE => SyntaxKind::TYPE,
            Self::COLON => SyntaxKind::COLON,
            Self::DESCRIPTION => SyntaxKind::DESCRIPTION,
            Self::OPEN_BRACKET => SyntaxKind::OPEN_BRACKET,
            Self::CLOSE_BRACKET => SyntaxKind::CLOSE_BRACKET,
            Self::OPTIONAL => SyntaxKind::OPTIONAL,
            Self::BODY_TEXT => SyntaxKind::BODY_TEXT,
            Self::SUMMARY => SyntaxKind::SUMMARY,
            Self::EXTENDED_SUMMARY => SyntaxKind::EXTENDED_SUMMARY,
            Self::STRAY_LINE => SyntaxKind::STRAY_LINE,
            Self::WARNING_TYPE => SyntaxKind::WARNING_TYPE,
            Self::UNDERLINE => SyntaxKind::UNDERLINE,
            Self::DIRECTIVE_MARKER => SyntaxKind::DIRECTIVE_MARKER,
            Self::KEYWORD => SyntaxKind::KEYWORD,
            Self::DOUBLE_COLON => SyntaxKind::DOUBLE_COLON,
            Self::VERSION => SyntaxKind::VERSION,
            Self::RETURN_TYPE => SyntaxKind::RETURN_TYPE,
            Self::DEFAULT_KEYWORD => SyntaxKind::DEFAULT_KEYWORD,
            Self::DEFAULT_SEPARATOR => SyntaxKind::DEFAULT_SEPARATOR,
            Self::DEFAULT_VALUE => SyntaxKind::DEFAULT_VALUE,
            Self::NUMBER => SyntaxKind::NUMBER,
            Self::CONTENT => SyntaxKind::CONTENT,
            Self::GOOGLE_DOCSTRING => SyntaxKind::GOOGLE_DOCSTRING,
            Self::GOOGLE_SECTION => SyntaxKind::GOOGLE_SECTION,
            Self::GOOGLE_SECTION_HEADER => SyntaxKind::GOOGLE_SECTION_HEADER,
            Self::GOOGLE_ARG => SyntaxKind::GOOGLE_ARG,
            Self::GOOGLE_RETURNS => SyntaxKind::GOOGLE_RETURNS,
            Self::GOOGLE_YIELDS => SyntaxKind::GOOGLE_YIELDS,
            Self::GOOGLE_EXCEPTION => SyntaxKind::GOOGLE_EXCEPTION,
            Self::GOOGLE_WARNING => SyntaxKind::GOOGLE_WARNING,
            Self::GOOGLE_SEE_ALSO_ITEM => SyntaxKind::GOOGLE_SEE_ALSO_ITEM,
            Self::GOOGLE_ATTRIBUTE => SyntaxKind::GOOGLE_ATTRIBUTE,
            Self::GOOGLE_METHOD => SyntaxKind::GOOGLE_METHOD,
            Self::NUMPY_DOCSTRING => SyntaxKind::NUMPY_DOCSTRING,
            Self::NUMPY_SECTION => SyntaxKind::NUMPY_SECTION,
            Self::NUMPY_SECTION_HEADER => SyntaxKind::NUMPY_SECTION_HEADER,
            Self::NUMPY_DEPRECATION => SyntaxKind::NUMPY_DEPRECATION,
            Self::NUMPY_PARAMETER => SyntaxKind::NUMPY_PARAMETER,
            Self::NUMPY_RETURNS => SyntaxKind::NUMPY_RETURNS,
            Self::NUMPY_YIELDS => SyntaxKind::NUMPY_YIELDS,
            Self::NUMPY_EXCEPTION => SyntaxKind::NUMPY_EXCEPTION,
            Self::NUMPY_WARNING => SyntaxKind::NUMPY_WARNING,
            Self::NUMPY_SEE_ALSO_ITEM => SyntaxKind::NUMPY_SEE_ALSO_ITEM,
            Self::NUMPY_REFERENCE => SyntaxKind::NUMPY_REFERENCE,
            Self::NUMPY_ATTRIBUTE => SyntaxKind::NUMPY_ATTRIBUTE,
            Self::NUMPY_METHOD => SyntaxKind::NUMPY_METHOD,
            Self::PLAIN_DOCSTRING => SyntaxKind::PLAIN_DOCSTRING,
        }
    }
}

// ─── Token ──────────────────────────────────────────────────────────────────

#[pyclass(name = "Token", frozen)]
struct PyToken {
    text: String,
    range: Py<PyTextRange>,
    kind: PySyntaxKind,
}

#[pymethods]
impl PyToken {
    #[getter]
    fn text(&self) -> &str {
        &self.text
    }
    #[getter]
    fn range(&self, py: Python<'_>) -> Py<PyTextRange> {
        self.range.clone_ref(py)
    }
    #[getter]
    fn kind(&self) -> PySyntaxKind {
        self.kind
    }
    fn __repr__(&self) -> String {
        format!("Token(SyntaxKind.{}, {:?})", self.kind.to_core().name(), self.text)
    }
}

fn to_py_token(py: Python<'_>, token: &SyntaxToken, source: &str) -> PyResult<Py<PyToken>> {
    Py::new(
        py,
        PyToken {
            kind: PySyntaxKind::from_core(token.kind()),
            text: token.text(source).to_string(),
            range: Py::new(py, PyTextRange::from(token.range()))?,
        },
    )
}

fn to_py_token_opt(py: Python<'_>, token: Option<&SyntaxToken>, source: &str) -> PyResult<Option<Py<PyToken>>> {
    token.map(|t| to_py_token(py, t, source)).transpose()
}

// ─── Node ───────────────────────────────────────────────────────────────────

#[pyclass(name = "Node", frozen)]
struct PyNode {
    kind: PySyntaxKind,
    range: Py<PyTextRange>,
    children: Vec<PyObject>,
}

#[pymethods]
impl PyNode {
    #[getter]
    fn kind(&self) -> PySyntaxKind {
        self.kind
    }
    #[getter]
    fn range(&self, py: Python<'_>) -> Py<PyTextRange> {
        self.range.clone_ref(py)
    }
    #[getter]
    fn children(&self, py: Python<'_>) -> Vec<PyObject> {
        self.children.iter().map(|c| c.clone_ref(py)).collect()
    }
    fn __repr__(&self) -> String {
        format!(
            "Node(SyntaxKind.{}, {} children)",
            self.kind.to_core().name(),
            self.children.len()
        )
    }
}

fn to_py_node(py: Python<'_>, node: &SyntaxNode, source: &str) -> PyResult<Py<PyNode>> {
    let children: Vec<PyObject> = node
        .children()
        .iter()
        .map(|child| match child {
            pydocstring_core::syntax::SyntaxElement::Node(n) => Ok(to_py_node(py, n, source)?.into_any()),
            pydocstring_core::syntax::SyntaxElement::Token(t) => Ok(to_py_token(py, t, source)?.into_any()),
        })
        .collect::<PyResult<Vec<_>>>()?;

    Py::new(
        py,
        PyNode {
            kind: PySyntaxKind::from_core(node.kind()),
            range: Py::new(py, PyTextRange::from(node.range()))?,
            children,
        },
    )
}

// ─── Walker ─────────────────────────────────────────────────────────────────

#[pyclass(name = "DocstringWalker")]
struct PyDocstringWalker {
    items: Vec<PyObject>,
    index: usize,
}

#[pymethods]
impl PyDocstringWalker {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        if slf.index < slf.items.len() {
            let item = slf.items[slf.index].clone_ref(slf.py());
            slf.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// ─── Google typed wrappers ──────────────────────────────────────────────────

#[pyclass(name = "GoogleArg", frozen)]
struct PyGoogleArg {
    node: Py<PyNode>,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
    optional: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleArg {
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.r#type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn optional(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.optional.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleArg({})", self.name.borrow(py).text)
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleArg>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_ARG {
            return Ok(None);
        }
        let name = match token_child(py, &node_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => return Ok(None),
        };
        let r#type = token_child(py, &node_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        let optional = token_child(py, &node_ref, PySyntaxKind::OPTIONAL);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleArg {
                node,
                name,
                r#type,
                description,
                optional,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleReturns", frozen)]
struct PyGoogleReturns {
    node: Py<PyNode>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleReturns {
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleReturns>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_RETURNS {
            return Ok(None);
        }
        let return_type = token_child(py, &node_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleReturns {
                node,
                return_type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleYields", frozen)]
struct PyGoogleYields {
    node: Py<PyNode>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleYields {
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleYields>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_YIELDS {
            return Ok(None);
        }
        let return_type = token_child(py, &node_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleYields {
                node,
                return_type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleException", frozen)]
struct PyGoogleException {
    node: Py<PyNode>,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleException {
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleException>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_EXCEPTION {
            return Ok(None);
        }
        let r#type = match token_child(py, &node_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleException {
                node,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleWarning", frozen)]
struct PyGoogleWarning {
    node: Py<PyNode>,
    warning_type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleWarning {
    #[getter]
    fn warning_type(&self, py: Python<'_>) -> Py<PyToken> {
        self.warning_type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleWarning>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_WARNING {
            return Ok(None);
        }
        let warning_type = match token_child(py, &node_ref, PySyntaxKind::WARNING_TYPE) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleWarning {
                node,
                warning_type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleSeeAlsoItem", frozen)]
struct PyGoogleSeeAlsoItem {
    node: Py<PyNode>,
    names: Vec<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleSeeAlsoItem {
    #[getter]
    fn names(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.names.iter().map(|n| n.clone_ref(py)).collect()
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleSeeAlsoItem>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_SEE_ALSO_ITEM {
            return Ok(None);
        }
        let names = token_children(py, &node_ref, PySyntaxKind::NAME);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleSeeAlsoItem {
                node,
                names,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleAttribute", frozen)]
struct PyGoogleAttribute {
    node: Py<PyNode>,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleAttribute {
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.r#type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleAttribute({})", self.name.borrow(py).text)
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleAttribute>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_ATTRIBUTE {
            return Ok(None);
        }
        let name = match token_child(py, &node_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => return Ok(None),
        };
        let r#type = token_child(py, &node_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleAttribute {
                node,
                name,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "GoogleMethod", frozen)]
struct PyGoogleMethod {
    node: Py<PyNode>,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleMethod {
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.r#type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleMethod({})", self.name.borrow(py).text)
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyGoogleMethod>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::GOOGLE_METHOD {
            return Ok(None);
        }
        let name = match token_child(py, &node_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => return Ok(None),
        };
        let r#type = token_child(py, &node_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyGoogleMethod {
                node,
                name,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

// ─── Google section kind enum ──────────────────────────────────────────────

#[pyclass(eq, eq_int, frozen, hash, name = "GoogleSectionKind")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PyGoogleSectionKind {
    #[pyo3(name = "ARGS")]
    Args,
    #[pyo3(name = "KEYWORD_ARGS")]
    KeywordArgs,
    #[pyo3(name = "OTHER_PARAMETERS")]
    OtherParameters,
    #[pyo3(name = "RECEIVES")]
    Receives,
    #[pyo3(name = "RETURNS")]
    Returns,
    #[pyo3(name = "YIELDS")]
    Yields,
    #[pyo3(name = "RAISES")]
    Raises,
    #[pyo3(name = "WARNS")]
    Warns,
    #[pyo3(name = "ATTRIBUTES")]
    Attributes,
    #[pyo3(name = "METHODS")]
    Methods,
    #[pyo3(name = "SEE_ALSO")]
    SeeAlso,
    #[pyo3(name = "NOTES")]
    Notes,
    #[pyo3(name = "EXAMPLES")]
    Examples,
    #[pyo3(name = "TODO")]
    Todo,
    #[pyo3(name = "REFERENCES")]
    References,
    #[pyo3(name = "WARNINGS")]
    Warnings,
    #[pyo3(name = "ATTENTION")]
    Attention,
    #[pyo3(name = "CAUTION")]
    Caution,
    #[pyo3(name = "DANGER")]
    Danger,
    #[pyo3(name = "ERROR")]
    Error,
    #[pyo3(name = "HINT")]
    Hint,
    #[pyo3(name = "IMPORTANT")]
    Important,
    #[pyo3(name = "TIP")]
    Tip,
    #[pyo3(name = "UNKNOWN")]
    Unknown,
}

#[pymethods]
impl PyGoogleSectionKind {
    fn __repr__(&self) -> String {
        format!(
            "GoogleSectionKind.{}",
            match self {
                Self::Args => "ARGS",
                Self::KeywordArgs => "KEYWORD_ARGS",
                Self::OtherParameters => "OTHER_PARAMETERS",
                Self::Receives => "RECEIVES",
                Self::Returns => "RETURNS",
                Self::Yields => "YIELDS",
                Self::Raises => "RAISES",
                Self::Warns => "WARNS",
                Self::Attributes => "ATTRIBUTES",
                Self::Methods => "METHODS",
                Self::SeeAlso => "SEE_ALSO",
                Self::Notes => "NOTES",
                Self::Examples => "EXAMPLES",
                Self::Todo => "TODO",
                Self::References => "REFERENCES",
                Self::Warnings => "WARNINGS",
                Self::Attention => "ATTENTION",
                Self::Caution => "CAUTION",
                Self::Danger => "DANGER",
                Self::Error => "ERROR",
                Self::Hint => "HINT",
                Self::Important => "IMPORTANT",
                Self::Tip => "TIP",
                Self::Unknown => "UNKNOWN",
            }
        )
    }
}

fn google_section_kind_to_py(kind: GoogleSectionKind) -> PyGoogleSectionKind {
    match kind {
        GoogleSectionKind::Args => PyGoogleSectionKind::Args,
        GoogleSectionKind::KeywordArgs => PyGoogleSectionKind::KeywordArgs,
        GoogleSectionKind::OtherParameters => PyGoogleSectionKind::OtherParameters,
        GoogleSectionKind::Receives => PyGoogleSectionKind::Receives,
        GoogleSectionKind::Returns => PyGoogleSectionKind::Returns,
        GoogleSectionKind::Yields => PyGoogleSectionKind::Yields,
        GoogleSectionKind::Raises => PyGoogleSectionKind::Raises,
        GoogleSectionKind::Warns => PyGoogleSectionKind::Warns,
        GoogleSectionKind::Attributes => PyGoogleSectionKind::Attributes,
        GoogleSectionKind::Methods => PyGoogleSectionKind::Methods,
        GoogleSectionKind::SeeAlso => PyGoogleSectionKind::SeeAlso,
        GoogleSectionKind::Notes => PyGoogleSectionKind::Notes,
        GoogleSectionKind::Examples => PyGoogleSectionKind::Examples,
        GoogleSectionKind::Todo => PyGoogleSectionKind::Todo,
        GoogleSectionKind::References => PyGoogleSectionKind::References,
        GoogleSectionKind::Warnings => PyGoogleSectionKind::Warnings,
        GoogleSectionKind::Attention => PyGoogleSectionKind::Attention,
        GoogleSectionKind::Caution => PyGoogleSectionKind::Caution,
        GoogleSectionKind::Danger => PyGoogleSectionKind::Danger,
        GoogleSectionKind::Error => PyGoogleSectionKind::Error,
        GoogleSectionKind::Hint => PyGoogleSectionKind::Hint,
        GoogleSectionKind::Important => PyGoogleSectionKind::Important,
        GoogleSectionKind::Tip => PyGoogleSectionKind::Tip,
        GoogleSectionKind::Unknown => PyGoogleSectionKind::Unknown,
    }
}

// ─── Google typed sections ──────────────────────────────────────────────────

#[pyclass(name = "GoogleSection", frozen)]
struct PyGoogleSection {
    node: Py<PyNode>,
    section_kind: PyGoogleSectionKind,
    header_name: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleSection {
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[getter]
    fn section_kind(&self) -> PyGoogleSectionKind {
        self.section_kind
    }
    #[getter]
    fn header_name(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.header_name.as_ref().map(|t| t.clone_ref(py))
    }
    fn args(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleArg>>> {
        collect_google_args(py, &*self.node.borrow(py))
    }
    fn returns(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleReturns>>> {
        collect_google_returns_entries(py, &*self.node.borrow(py))
    }
    fn yields(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleYields>>> {
        collect_google_yields_entries(py, &*self.node.borrow(py))
    }
    fn exceptions(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleException>>> {
        collect_google_exceptions(py, &*self.node.borrow(py))
    }
    fn warnings(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleWarning>>> {
        collect_google_warnings(py, &*self.node.borrow(py))
    }
    fn see_also_items(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleSeeAlsoItem>>> {
        collect_google_see_also_items(py, &*self.node.borrow(py))
    }
    fn attributes(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleAttribute>>> {
        collect_google_attributes(py, &*self.node.borrow(py))
    }
    fn methods(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleMethod>>> {
        collect_google_methods(py, &*self.node.borrow(py))
    }
    fn body_text(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        token_child(py, &*self.node.borrow(py), PySyntaxKind::BODY_TEXT)
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        match &self.header_name {
            Some(t) => format!("GoogleSection({})", t.borrow(py).text),
            None => "GoogleSection(?)".to_string(),
        }
    }
}

#[pyclass(name = "GoogleDocstring", frozen)]
struct PyGoogleDocstring {
    node: Py<PyNode>,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    source: String,
}

#[pymethods]
impl PyGoogleDocstring {
    #[getter]
    fn summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn extended_summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.extended_summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    fn sections(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGoogleSection>>> {
        let section_nodes = node_children(py, &*self.node.borrow(py), PySyntaxKind::GOOGLE_SECTION);
        let mut result = Vec::with_capacity(section_nodes.len());
        for section_py in section_nodes {
            let section_kind = {
                let section_ref = section_py.borrow(py);
                get_google_section_kind(py, &*section_ref)
                    .map(google_section_kind_to_py)
                    .unwrap_or(PyGoogleSectionKind::Unknown)
            };
            let header_name = {
                let section_ref = section_py.borrow(py);
                node_child(py, &*section_ref, PySyntaxKind::GOOGLE_SECTION_HEADER).and_then(|header_py| {
                    let header_ref = header_py.borrow(py);
                    token_child(py, &*header_ref, PySyntaxKind::NAME)
                })
            };
            result.push(Py::new(
                py,
                PyGoogleSection {
                    node: section_py,
                    section_kind,
                    header_name,
                },
            )?);
        }
        Ok(result)
    }
    fn pretty_print(&self) -> String {
        let parsed = google::parse_google(&self.source);
        parsed.pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = google::parse_google(&self.source);
        let doc = pydocstring_core::parse::google::to_model::to_model(&parsed)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))?;
        Ok(PyModelDocstring { inner: doc })
    }
    /// Convert a byte offset to a `LineColumn` with codepoint-based `col`.
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        let lc = byte_offset_to_line_col(&self.source, offset as usize)?;
        Py::new(py, lc)
    }
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::Google
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    fn walk(&self, py: Python<'_>) -> PyResult<Py<PyDocstringWalker>> {
        let mut items: Vec<PyObject> = Vec::new();
        collect_walk_items(py, self.node.clone_ref(py).into_any(), &mut items);
        Py::new(py, PyDocstringWalker { items, index: 0 })
    }
    fn __repr__(&self) -> String {
        "GoogleDocstring(...)".to_string()
    }
}

// ─── NumPy typed wrappers ───────────────────────────────────────────────────

#[pyclass(name = "NumPyParameter", frozen)]
struct PyNumPyParameter {
    node: Py<PyNode>,
    names: Vec<Py<PyToken>>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
    optional: Option<Py<PyToken>>,
    default_value: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyParameter {
    #[getter]
    fn names(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.names.iter().map(|n| n.clone_ref(py)).collect()
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.r#type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn optional(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.optional.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn default_value(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.default_value.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        let name_texts: Vec<String> = self.names.iter().map(|n| n.borrow(py).text.clone()).collect();
        format!("NumPyParameter({})", name_texts.join(", "))
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyParameter>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_PARAMETER {
            return Ok(None);
        }
        let names = token_children(py, &node_ref, PySyntaxKind::NAME);
        if names.is_empty() {
            return Ok(None);
        }
        let r#type = token_child(py, &node_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        let optional = token_child(py, &node_ref, PySyntaxKind::OPTIONAL);
        let default_value = token_child(py, &node_ref, PySyntaxKind::DEFAULT_VALUE);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyParameter {
                node,
                names,
                r#type,
                description,
                optional,
                default_value,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyReturns", frozen)]
struct PyNumPyReturns {
    node: Py<PyNode>,
    name: Option<Py<PyToken>>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyReturns {
    #[getter]
    fn name(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.name.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyReturns>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_RETURNS {
            return Ok(None);
        }
        let name = token_child(py, &node_ref, PySyntaxKind::NAME);
        let return_type = token_child(py, &node_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyReturns {
                node,
                name,
                return_type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyYields", frozen)]
struct PyNumPyYields {
    node: Py<PyNode>,
    name: Option<Py<PyToken>>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyYields {
    #[getter]
    fn name(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.name.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyYields>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_YIELDS {
            return Ok(None);
        }
        let name = token_child(py, &node_ref, PySyntaxKind::NAME);
        let return_type = token_child(py, &node_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyYields {
                node,
                name,
                return_type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyException", frozen)]
struct PyNumPyException {
    node: Py<PyNode>,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyException {
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyException>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_EXCEPTION {
            return Ok(None);
        }
        let r#type = match token_child(py, &node_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyException {
                node,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyDeprecation", frozen)]
struct PyNumPyDeprecation {
    node: Py<PyNode>,
    version: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyDeprecation {
    #[getter]
    fn version(&self, py: Python<'_>) -> Py<PyToken> {
        self.version.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyDeprecation>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_DEPRECATION {
            return Ok(None);
        }
        let version = match token_child(py, &node_ref, PySyntaxKind::VERSION) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyDeprecation {
                node,
                version,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyWarning", frozen)]
struct PyNumPyWarning {
    node: Py<PyNode>,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyWarning {
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyWarning>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_WARNING {
            return Ok(None);
        }
        let r#type = match token_child(py, &node_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyWarning {
                node,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPySeeAlsoItem", frozen)]
struct PyNumPySeeAlsoItem {
    node: Py<PyNode>,
    names: Vec<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPySeeAlsoItem {
    #[getter]
    fn names(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.names.iter().map(|n| n.clone_ref(py)).collect()
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPySeeAlsoItem>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_SEE_ALSO_ITEM {
            return Ok(None);
        }
        let names = token_children(py, &node_ref, PySyntaxKind::NAME);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPySeeAlsoItem {
                node,
                names,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyReference", frozen)]
struct PyNumPyReference {
    node: Py<PyNode>,
    number: Option<Py<PyToken>>,
    content: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyReference {
    #[getter]
    fn number(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.number.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn content(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.content.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyReference>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_REFERENCE {
            return Ok(None);
        }
        let number = token_child(py, &node_ref, PySyntaxKind::NUMBER);
        let content = token_child(py, &node_ref, PySyntaxKind::CONTENT);
        drop(node_ref);
        Py::new(py, PyNumPyReference { node, number, content }).map(Some)
    }
}

#[pyclass(name = "NumPyAttribute", frozen)]
struct PyNumPyAttribute {
    node: Py<PyNode>,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyAttribute {
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.r#type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyAttribute({})", self.name.borrow(py).text)
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyAttribute>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_ATTRIBUTE {
            return Ok(None);
        }
        let name = match token_child(py, &node_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => return Ok(None),
        };
        let r#type = token_child(py, &node_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyAttribute {
                node,
                name,
                r#type,
                description,
            },
        )
        .map(Some)
    }
}

#[pyclass(name = "NumPyMethod", frozen)]
struct PyNumPyMethod {
    node: Py<PyNode>,
    name: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyMethod {
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyMethod({})", self.name.borrow(py).text)
    }
    #[staticmethod]
    fn cast(py: Python<'_>, node: Py<PyNode>) -> PyResult<Option<Py<PyNumPyMethod>>> {
        let node_ref = node.borrow(py);
        if node_ref.kind != PySyntaxKind::NUMPY_METHOD {
            return Ok(None);
        }
        let name = match token_child(py, &node_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => return Ok(None),
        };
        let description = token_child(py, &node_ref, PySyntaxKind::DESCRIPTION);
        drop(node_ref);
        Py::new(
            py,
            PyNumPyMethod {
                node,
                name,
                description,
            },
        )
        .map(Some)
    }
}

// ─── NumPy section kind enum ───────────────────────────────────────────────

#[pyclass(eq, eq_int, frozen, hash, name = "NumPySectionKind")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PyNumPySectionKind {
    #[pyo3(name = "PARAMETERS")]
    Parameters,
    #[pyo3(name = "RETURNS")]
    Returns,
    #[pyo3(name = "YIELDS")]
    Yields,
    #[pyo3(name = "RECEIVES")]
    Receives,
    #[pyo3(name = "OTHER_PARAMETERS")]
    OtherParameters,
    #[pyo3(name = "RAISES")]
    Raises,
    #[pyo3(name = "WARNS")]
    Warns,
    #[pyo3(name = "WARNINGS")]
    Warnings,
    #[pyo3(name = "SEE_ALSO")]
    SeeAlso,
    #[pyo3(name = "NOTES")]
    Notes,
    #[pyo3(name = "REFERENCES")]
    References,
    #[pyo3(name = "EXAMPLES")]
    Examples,
    #[pyo3(name = "ATTRIBUTES")]
    Attributes,
    #[pyo3(name = "METHODS")]
    Methods,
    #[pyo3(name = "UNKNOWN")]
    Unknown,
}

#[pymethods]
impl PyNumPySectionKind {
    fn __repr__(&self) -> String {
        format!(
            "NumPySectionKind.{}",
            match self {
                Self::Parameters => "PARAMETERS",
                Self::Returns => "RETURNS",
                Self::Yields => "YIELDS",
                Self::Receives => "RECEIVES",
                Self::OtherParameters => "OTHER_PARAMETERS",
                Self::Raises => "RAISES",
                Self::Warns => "WARNS",
                Self::Warnings => "WARNINGS",
                Self::SeeAlso => "SEE_ALSO",
                Self::Notes => "NOTES",
                Self::References => "REFERENCES",
                Self::Examples => "EXAMPLES",
                Self::Attributes => "ATTRIBUTES",
                Self::Methods => "METHODS",
                Self::Unknown => "UNKNOWN",
            }
        )
    }
}

fn numpy_section_kind_to_py(kind: NumPySectionKind) -> PyNumPySectionKind {
    match kind {
        NumPySectionKind::Parameters => PyNumPySectionKind::Parameters,
        NumPySectionKind::Returns => PyNumPySectionKind::Returns,
        NumPySectionKind::Yields => PyNumPySectionKind::Yields,
        NumPySectionKind::Receives => PyNumPySectionKind::Receives,
        NumPySectionKind::OtherParameters => PyNumPySectionKind::OtherParameters,
        NumPySectionKind::Raises => PyNumPySectionKind::Raises,
        NumPySectionKind::Warns => PyNumPySectionKind::Warns,
        NumPySectionKind::Warnings => PyNumPySectionKind::Warnings,
        NumPySectionKind::SeeAlso => PyNumPySectionKind::SeeAlso,
        NumPySectionKind::Notes => PyNumPySectionKind::Notes,
        NumPySectionKind::References => PyNumPySectionKind::References,
        NumPySectionKind::Examples => PyNumPySectionKind::Examples,
        NumPySectionKind::Attributes => PyNumPySectionKind::Attributes,
        NumPySectionKind::Methods => PyNumPySectionKind::Methods,
        NumPySectionKind::Unknown => PyNumPySectionKind::Unknown,
    }
}

// ─── NumPy typed sections ───────────────────────────────────────────────────

#[pyclass(name = "NumPySection", frozen)]
struct PyNumPySection {
    node: Py<PyNode>,
    section_kind: PyNumPySectionKind,
    header_name: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPySection {
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    #[getter]
    fn section_kind(&self) -> PyNumPySectionKind {
        self.section_kind
    }
    #[getter]
    fn header_name(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.header_name.as_ref().map(|t| t.clone_ref(py))
    }
    fn parameters(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyParameter>>> {
        collect_numpy_parameters(py, &*self.node.borrow(py))
    }
    fn returns(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyReturns>>> {
        collect_numpy_returns_entries(py, &*self.node.borrow(py))
    }
    fn yields(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyYields>>> {
        collect_numpy_yields_entries(py, &*self.node.borrow(py))
    }
    fn exceptions(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyException>>> {
        collect_numpy_exceptions(py, &*self.node.borrow(py))
    }
    fn warnings(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyWarning>>> {
        collect_numpy_warnings(py, &*self.node.borrow(py))
    }
    fn see_also_items(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPySeeAlsoItem>>> {
        collect_numpy_see_also_items(py, &*self.node.borrow(py))
    }
    fn references(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyReference>>> {
        collect_numpy_references(py, &*self.node.borrow(py))
    }
    fn attributes(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyAttribute>>> {
        collect_numpy_attributes(py, &*self.node.borrow(py))
    }
    fn methods(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPyMethod>>> {
        collect_numpy_methods(py, &*self.node.borrow(py))
    }
    fn body_text(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        token_child(py, &*self.node.borrow(py), PySyntaxKind::BODY_TEXT)
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        match &self.header_name {
            Some(t) => format!("NumPySection({})", t.borrow(py).text),
            None => "NumPySection(?)".to_string(),
        }
    }
}

#[pyclass(name = "NumPyDocstring", frozen)]
struct PyNumPyDocstring {
    node: Py<PyNode>,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    source: String,
}

#[pymethods]
impl PyNumPyDocstring {
    #[getter]
    fn summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn extended_summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.extended_summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    fn sections(&self, py: Python<'_>) -> PyResult<Vec<Py<PyNumPySection>>> {
        let section_nodes = node_children(py, &*self.node.borrow(py), PySyntaxKind::NUMPY_SECTION);
        let mut result = Vec::with_capacity(section_nodes.len());
        for section_py in section_nodes {
            let section_kind = {
                let section_ref = section_py.borrow(py);
                get_numpy_section_kind(py, &*section_ref)
                    .map(numpy_section_kind_to_py)
                    .unwrap_or(PyNumPySectionKind::Unknown)
            };
            let header_name = {
                let section_ref = section_py.borrow(py);
                node_child(py, &*section_ref, PySyntaxKind::NUMPY_SECTION_HEADER).and_then(|header_py| {
                    let header_ref = header_py.borrow(py);
                    token_child(py, &*header_ref, PySyntaxKind::NAME)
                })
            };
            result.push(Py::new(
                py,
                PyNumPySection {
                    node: section_py,
                    section_kind,
                    header_name,
                },
            )?);
        }
        Ok(result)
    }
    fn deprecation(&self, py: Python<'_>) -> PyResult<Option<Py<PyNumPyDeprecation>>> {
        let node_ref = self.node.borrow(py);
        match node_child(py, &*node_ref, PySyntaxKind::NUMPY_DEPRECATION) {
            None => Ok(None),
            Some(dep_py) => {
                let dep_ref = dep_py.borrow(py);
                let version = match token_child(py, &*dep_ref, PySyntaxKind::VERSION) {
                    Some(t) => t,
                    None => return Ok(None),
                };
                let description = token_child(py, &*dep_ref, PySyntaxKind::DESCRIPTION);
                drop(dep_ref);
                Py::new(
                    py,
                    PyNumPyDeprecation {
                        node: dep_py,
                        version,
                        description,
                    },
                )
                .map(Some)
            }
        }
    }
    fn pretty_print(&self) -> String {
        let parsed = pydocstring_core::parse::numpy::parse_numpy(&self.source);
        parsed.pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = pydocstring_core::parse::numpy::parse_numpy(&self.source);
        let doc = pydocstring_core::parse::numpy::to_model::to_model(&parsed)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))?;
        Ok(PyModelDocstring { inner: doc })
    }
    /// Convert a byte offset to a `LineColumn` with codepoint-based `col`.
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        let lc = byte_offset_to_line_col(&self.source, offset as usize)?;
        Py::new(py, lc)
    }
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::NumPy
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn walk(&self, py: Python<'_>) -> PyResult<Py<PyDocstringWalker>> {
        let mut items: Vec<PyObject> = Vec::new();
        collect_walk_items(py, self.node.clone_ref(py).into_any(), &mut items);
        Py::new(py, PyDocstringWalker { items, index: 0 })
    }
    fn __repr__(&self) -> String {
        "NumPyDocstring(...)".to_string()
    }
}

// ─── Plain typed wrapper ────────────────────────────────────────────────────

#[pyclass(name = "PlainDocstring", frozen)]
struct PyPlainDocstring {
    node: Py<PyNode>,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    source: String,
}

#[pymethods]
impl PyPlainDocstring {
    #[getter]
    fn summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn extended_summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.extended_summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    fn pretty_print(&self) -> String {
        pydocstring_core::parse::plain::parse_plain(&self.source).pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = pydocstring_core::parse::plain::parse_plain(&self.source);
        let doc = pydocstring_core::parse::plain::to_model::to_model(&parsed)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))?;
        Ok(PyModelDocstring { inner: doc })
    }
    /// Convert a byte offset to a `LineColumn` with codepoint-based `col`.
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        let lc = byte_offset_to_line_col(&self.source, offset as usize)?;
        Py::new(py, lc)
    }
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::Plain
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn kind(&self, py: Python<'_>) -> PySyntaxKind {
        self.node.borrow(py).kind
    }
    fn walk(&self, py: Python<'_>) -> PyResult<Py<PyDocstringWalker>> {
        let mut items: Vec<PyObject> = Vec::new();
        collect_walk_items(py, self.node.clone_ref(py).into_any(), &mut items);
        Py::new(py, PyDocstringWalker { items, index: 0 })
    }
    fn __repr__(&self) -> String {
        "PlainDocstring(...)".to_string()
    }
}

// ─── Walk / cast helpers ────────────────────────────────────────────────────

/// Walk the `PyNode` tree depth-first, collecting every Node and Token item.
fn collect_walk_items(py: Python<'_>, obj: PyObject, items: &mut Vec<PyObject>) {
    // Extract children before consuming obj (bind borrows obj).
    let children: Option<Vec<PyObject>> = {
        let bound = obj.bind(py);
        bound.downcast::<PyNode>().ok().map(|n| {
            let nr = n.borrow();
            nr.children.iter().map(|c| c.clone_ref(py)).collect()
        })
    }; // bound dropped here — obj borrow released
    items.push(obj);
    if let Some(childs) = children {
        for child in childs {
            collect_walk_items(py, child, items);
        }
    }
}

/// Find the first direct token child of `node` with the given kind.
fn token_child(py: Python<'_>, node: &PyNode, kind: PySyntaxKind) -> Option<Py<PyToken>> {
    for child in &node.children {
        let bound = child.bind(py);
        if let Ok(tok) = bound.downcast::<PyToken>() {
            let tok_kind = tok.borrow().kind;
            if tok_kind == kind {
                return Some(tok.as_unbound().clone_ref(py));
            }
        }
    }
    None
}

/// Collect all direct token children of `node` with the given kind.
fn token_children(py: Python<'_>, node: &PyNode, kind: PySyntaxKind) -> Vec<Py<PyToken>> {
    node.children
        .iter()
        .filter_map(|child| {
            let bound = child.bind(py);
            if let Ok(tok) = bound.downcast::<PyToken>() {
                let tok_kind = tok.borrow().kind;
                if tok_kind == kind {
                    return Some(tok.as_unbound().clone_ref(py));
                }
            }
            None
        })
        .collect()
}

/// Find the first direct node child of `node` with the given kind.
fn node_child(py: Python<'_>, node: &PyNode, kind: PySyntaxKind) -> Option<Py<PyNode>> {
    for child in &node.children {
        let bound = child.bind(py);
        if let Ok(n) = bound.downcast::<PyNode>() {
            let n_kind = n.borrow().kind;
            if n_kind == kind {
                return Some(n.as_unbound().clone_ref(py));
            }
        }
    }
    None
}

/// Collect all direct node children of `node` with the given kind.
fn node_children(py: Python<'_>, node: &PyNode, kind: PySyntaxKind) -> Vec<Py<PyNode>> {
    node.children
        .iter()
        .filter_map(|child| {
            let bound = child.bind(py);
            if let Ok(n) = bound.downcast::<PyNode>() {
                let n_kind = n.borrow().kind;
                if n_kind == kind {
                    return Some(n.as_unbound().clone_ref(py));
                }
            }
            None
        })
        .collect()
}

/// Extract the `GoogleSectionKind` from a `GOOGLE_SECTION` `PyNode`.
fn get_google_section_kind(py: Python<'_>, section_node: &PyNode) -> Option<GoogleSectionKind> {
    let header_py = node_child(py, section_node, PySyntaxKind::GOOGLE_SECTION_HEADER)?;
    let header_ref = header_py.borrow(py);
    let name_tok = token_child(py, &*header_ref, PySyntaxKind::NAME)?;
    let kind = {
        let name_ref = name_tok.borrow(py);
        GoogleSectionKind::from_name(&name_ref.text.to_lowercase())
    };
    Some(kind)
}

/// Extract the `NumPySectionKind` from a `NUMPY_SECTION` `PyNode`.
fn get_numpy_section_kind(py: Python<'_>, section_node: &PyNode) -> Option<NumPySectionKind> {
    let header_py = node_child(py, section_node, PySyntaxKind::NUMPY_SECTION_HEADER)?;
    let header_ref = header_py.borrow(py);
    let name_tok = token_child(py, &*header_ref, PySyntaxKind::NAME)?;
    let kind = {
        let name_ref = name_tok.borrow(py);
        NumPySectionKind::from_name(&name_ref.text.to_lowercase())
    };
    Some(kind)
}

// ─── Entry collectors ────────────────────────────────────────────────────────

fn collect_google_args(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleArg>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_ARG);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = match token_child(py, &*child_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => continue,
        };
        let r#type = token_child(py, &*child_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        let optional = token_child(py, &*child_ref, PySyntaxKind::OPTIONAL);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleArg {
                node: child_py,
                name,
                r#type,
                description,
                optional,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_returns_entries(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleReturns>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_RETURNS);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let return_type = token_child(py, &*child_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleReturns {
                node: child_py,
                return_type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_yields_entries(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleYields>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_YIELDS);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let return_type = token_child(py, &*child_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleYields {
                node: child_py,
                return_type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_exceptions(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleException>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_EXCEPTION);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let r#type = match token_child(py, &*child_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => continue,
        };
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleException {
                node: child_py,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_warnings(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleWarning>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_WARNING);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let warning_type = match token_child(py, &*child_ref, PySyntaxKind::WARNING_TYPE) {
            Some(t) => t,
            None => continue,
        };
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleWarning {
                node: child_py,
                warning_type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_see_also_items(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleSeeAlsoItem>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_SEE_ALSO_ITEM);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let names = token_children(py, &*child_ref, PySyntaxKind::NAME);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleSeeAlsoItem {
                node: child_py,
                names,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_attributes(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleAttribute>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_ATTRIBUTE);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = match token_child(py, &*child_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => continue,
        };
        let r#type = token_child(py, &*child_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleAttribute {
                node: child_py,
                name,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_google_methods(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyGoogleMethod>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::GOOGLE_METHOD);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = match token_child(py, &*child_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => continue,
        };
        let r#type = token_child(py, &*child_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyGoogleMethod {
                node: child_py,
                name,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_parameters(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyParameter>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_PARAMETER);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let names = token_children(py, &*child_ref, PySyntaxKind::NAME);
        if names.is_empty() {
            continue;
        }
        let r#type = token_child(py, &*child_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        let optional = token_child(py, &*child_ref, PySyntaxKind::OPTIONAL);
        let default_value = token_child(py, &*child_ref, PySyntaxKind::DEFAULT_VALUE);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyParameter {
                node: child_py,
                names,
                r#type,
                description,
                optional,
                default_value,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_returns_entries(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyReturns>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_RETURNS);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = token_child(py, &*child_ref, PySyntaxKind::NAME);
        let return_type = token_child(py, &*child_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyReturns {
                node: child_py,
                name,
                return_type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_yields_entries(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyYields>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_YIELDS);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = token_child(py, &*child_ref, PySyntaxKind::NAME);
        let return_type = token_child(py, &*child_ref, PySyntaxKind::RETURN_TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyYields {
                node: child_py,
                name,
                return_type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_exceptions(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyException>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_EXCEPTION);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let r#type = match token_child(py, &*child_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => continue,
        };
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyException {
                node: child_py,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_warnings(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyWarning>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_WARNING);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let r#type = match token_child(py, &*child_ref, PySyntaxKind::TYPE) {
            Some(t) => t,
            None => continue,
        };
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyWarning {
                node: child_py,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_see_also_items(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPySeeAlsoItem>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_SEE_ALSO_ITEM);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let names = token_children(py, &*child_ref, PySyntaxKind::NAME);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPySeeAlsoItem {
                node: child_py,
                names,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_references(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyReference>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_REFERENCE);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let number = token_child(py, &*child_ref, PySyntaxKind::NUMBER);
        let content = token_child(py, &*child_ref, PySyntaxKind::CONTENT);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyReference {
                node: child_py,
                number,
                content,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_attributes(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyAttribute>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_ATTRIBUTE);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = match token_child(py, &*child_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => continue,
        };
        let r#type = token_child(py, &*child_ref, PySyntaxKind::TYPE);
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyAttribute {
                node: child_py,
                name,
                r#type,
                description,
            },
        )?);
    }
    Ok(result)
}

fn collect_numpy_methods(py: Python<'_>, section: &PyNode) -> PyResult<Vec<Py<PyNumPyMethod>>> {
    let child_nodes = node_children(py, section, PySyntaxKind::NUMPY_METHOD);
    let mut result = Vec::with_capacity(child_nodes.len());
    for child_py in child_nodes {
        let child_ref = child_py.borrow(py);
        let name = match token_child(py, &*child_ref, PySyntaxKind::NAME) {
            Some(t) => t,
            None => continue,
        };
        let description = token_child(py, &*child_ref, PySyntaxKind::DESCRIPTION);
        drop(child_ref);
        result.push(Py::new(
            py,
            PyNumPyMethod {
                node: child_py,
                name,
                description,
            },
        )?);
    }
    Ok(result)
}

// ─── Conversion helpers ─────────────────────────────────────────────────────

fn build_google_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyGoogleDocstring>> {
    let source = parsed.source();
    let doc = gn::GoogleDocstring::cast(parsed.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root node is not a GOOGLE_DOCSTRING"))?;
    let summary = to_py_token_opt(py, doc.summary(), source)?;
    let extended_summary = to_py_token_opt(py, doc.extended_summary(), source)?;
    let node = to_py_node(py, parsed.root(), source)?;
    Py::new(
        py,
        PyGoogleDocstring {
            summary,
            extended_summary,
            node,
            source: source.to_string(),
        },
    )
}

fn build_numpy_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyNumPyDocstring>> {
    let source = parsed.source();
    let doc = nn::NumPyDocstring::cast(parsed.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root node is not a NUMPY_DOCSTRING"))?;
    let summary = to_py_token_opt(py, doc.summary(), source)?;
    let extended_summary = to_py_token_opt(py, doc.extended_summary(), source)?;
    let node = to_py_node(py, parsed.root(), source)?;
    Py::new(
        py,
        PyNumPyDocstring {
            summary,
            extended_summary,
            node,
            source: source.to_string(),
        },
    )
}

fn build_plain_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyPlainDocstring>> {
    let source = parsed.source();
    let doc = pydocstring_core::parse::plain::nodes::PlainDocstring::cast(parsed.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root node is not a PLAIN_DOCSTRING"))?;

    let summary = to_py_token_opt(py, doc.summary(), source)?;
    let extended_summary = to_py_token_opt(py, doc.extended_summary(), source)?;
    let node = to_py_node(py, parsed.root(), source)?;

    Py::new(
        py,
        PyPlainDocstring {
            summary,
            extended_summary,
            node,
            source: source.to_string(),
        },
    )
}

// ─── Model IR types ─────────────────────────────────────────────────────────

#[pyclass(name = "Deprecation")]
#[derive(Clone)]
struct PyModelDeprecation {
    #[pyo3(get, set)]
    version: String,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelDeprecation {
    #[new]
    #[pyo3(signature = (version, *, description=None))]
    fn new(version: String, description: Option<String>) -> Self {
        Self { version, description }
    }
    fn __repr__(&self) -> String {
        format!("Deprecation(version={:?})", self.version)
    }
}

#[pyclass(name = "Parameter")]
#[derive(Clone)]
struct PyModelParameter {
    #[pyo3(get, set)]
    names: Vec<String>,
    #[pyo3(get, set)]
    type_annotation: Option<String>,
    #[pyo3(get, set)]
    description: Option<String>,
    #[pyo3(get, set)]
    is_optional: bool,
    #[pyo3(get, set)]
    default_value: Option<String>,
}

#[pymethods]
impl PyModelParameter {
    #[new]
    #[pyo3(signature = (names, *, type_annotation=None, description=None, is_optional=false, default_value=None))]
    fn new(
        names: Vec<String>,
        type_annotation: Option<String>,
        description: Option<String>,
        is_optional: bool,
        default_value: Option<String>,
    ) -> Self {
        Self {
            names,
            type_annotation,
            description,
            is_optional,
            default_value,
        }
    }
    fn __repr__(&self) -> String {
        format!("Parameter({})", self.names.join(", "))
    }
}

#[pyclass(name = "Return")]
#[derive(Clone)]
struct PyModelReturn {
    #[pyo3(get, set)]
    name: Option<String>,
    #[pyo3(get, set)]
    type_annotation: Option<String>,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelReturn {
    #[new]
    #[pyo3(signature = (*, name=None, type_annotation=None, description=None))]
    fn new(name: Option<String>, type_annotation: Option<String>, description: Option<String>) -> Self {
        Self {
            name,
            type_annotation,
            description,
        }
    }
    fn __repr__(&self) -> String {
        if let Some(ref name) = self.name {
            format!("Return({})", name)
        } else {
            "Return(...)".to_string()
        }
    }
}

#[pyclass(name = "ExceptionEntry")]
#[derive(Clone)]
struct PyModelExceptionEntry {
    #[pyo3(get, set)]
    type_name: String,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelExceptionEntry {
    #[new]
    #[pyo3(signature = (type_name, *, description=None))]
    fn new(type_name: String, description: Option<String>) -> Self {
        Self { type_name, description }
    }
    fn __repr__(&self) -> String {
        format!("ExceptionEntry({})", self.type_name)
    }
}

#[pyclass(name = "SeeAlsoEntry")]
#[derive(Clone)]
struct PyModelSeeAlsoEntry {
    #[pyo3(get, set)]
    names: Vec<String>,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelSeeAlsoEntry {
    #[new]
    #[pyo3(signature = (names, *, description=None))]
    fn new(names: Vec<String>, description: Option<String>) -> Self {
        Self { names, description }
    }
    fn __repr__(&self) -> String {
        format!("SeeAlsoEntry({})", self.names.join(", "))
    }
}

#[pyclass(name = "Reference")]
#[derive(Clone)]
struct PyModelReference {
    #[pyo3(get, set)]
    number: Option<String>,
    #[pyo3(get, set)]
    content: Option<String>,
}

#[pymethods]
impl PyModelReference {
    #[new]
    #[pyo3(signature = (*, number=None, content=None))]
    fn new(number: Option<String>, content: Option<String>) -> Self {
        Self { number, content }
    }
    fn __repr__(&self) -> String {
        if let Some(ref num) = self.number {
            format!("Reference({})", num)
        } else {
            "Reference(...)".to_string()
        }
    }
}

#[pyclass(name = "Attribute")]
#[derive(Clone)]
struct PyModelAttribute {
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    type_annotation: Option<String>,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelAttribute {
    #[new]
    #[pyo3(signature = (name, *, type_annotation=None, description=None))]
    fn new(name: String, type_annotation: Option<String>, description: Option<String>) -> Self {
        Self {
            name,
            type_annotation,
            description,
        }
    }
    fn __repr__(&self) -> String {
        format!("Attribute({})", self.name)
    }
}

#[pyclass(name = "Method")]
#[derive(Clone)]
struct PyModelMethod {
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    type_annotation: Option<String>,
    #[pyo3(get, set)]
    description: Option<String>,
}

#[pymethods]
impl PyModelMethod {
    #[new]
    #[pyo3(signature = (name, *, type_annotation=None, description=None))]
    fn new(name: String, type_annotation: Option<String>, description: Option<String>) -> Self {
        Self {
            name,
            type_annotation,
            description,
        }
    }
    fn __repr__(&self) -> String {
        format!("Method({})", self.name)
    }
}

// ─── Section ────────────────────────────────────────────────────────────────

fn free_section_kind_to_str(kind: &model::FreeSectionKind) -> &str {
    match kind {
        model::FreeSectionKind::Notes => "notes",
        model::FreeSectionKind::Examples => "examples",
        model::FreeSectionKind::Warnings => "warnings",
        model::FreeSectionKind::Todo => "todo",
        model::FreeSectionKind::Attention => "attention",
        model::FreeSectionKind::Caution => "caution",
        model::FreeSectionKind::Danger => "danger",
        model::FreeSectionKind::Error => "error",
        model::FreeSectionKind::Hint => "hint",
        model::FreeSectionKind::Important => "important",
        model::FreeSectionKind::Tip => "tip",
        model::FreeSectionKind::Unknown(name) => name.as_str(),
    }
}

fn str_to_free_section_kind(s: &str) -> model::FreeSectionKind {
    match s {
        "notes" => model::FreeSectionKind::Notes,
        "examples" => model::FreeSectionKind::Examples,
        "warnings" => model::FreeSectionKind::Warnings,
        "todo" => model::FreeSectionKind::Todo,
        "attention" => model::FreeSectionKind::Attention,
        "caution" => model::FreeSectionKind::Caution,
        "danger" => model::FreeSectionKind::Danger,
        "error" => model::FreeSectionKind::Error,
        "hint" => model::FreeSectionKind::Hint,
        "important" => model::FreeSectionKind::Important,
        "tip" => model::FreeSectionKind::Tip,
        other => model::FreeSectionKind::Unknown(other.to_string()),
    }
}

fn extract_parameters(py: Python<'_>, entries: &[Py<PyModelParameter>]) -> Vec<model::Parameter> {
    entries
        .iter()
        .map(|p| {
            let p = p.borrow(py);
            model::Parameter {
                names: p.names.clone(),
                type_annotation: p.type_annotation.clone(),
                description: p.description.clone(),
                is_optional: p.is_optional,
                default_value: p.default_value.clone(),
            }
        })
        .collect()
}

fn extract_returns(py: Python<'_>, entries: &[Py<PyModelReturn>]) -> Vec<model::Return> {
    entries
        .iter()
        .map(|r| {
            let r = r.borrow(py);
            model::Return {
                name: r.name.clone(),
                type_annotation: r.type_annotation.clone(),
                description: r.description.clone(),
            }
        })
        .collect()
}

fn extract_exceptions(py: Python<'_>, entries: &[Py<PyModelExceptionEntry>]) -> Vec<model::ExceptionEntry> {
    entries
        .iter()
        .map(|e| {
            let e = e.borrow(py);
            model::ExceptionEntry {
                type_name: e.type_name.clone(),
                description: e.description.clone(),
            }
        })
        .collect()
}

fn extract_attributes(py: Python<'_>, entries: &[Py<PyModelAttribute>]) -> Vec<model::Attribute> {
    entries
        .iter()
        .map(|a| {
            let a = a.borrow(py);
            model::Attribute {
                name: a.name.clone(),
                type_annotation: a.type_annotation.clone(),
                description: a.description.clone(),
            }
        })
        .collect()
}

fn extract_methods(py: Python<'_>, entries: &[Py<PyModelMethod>]) -> Vec<model::Method> {
    entries
        .iter()
        .map(|m| {
            let m = m.borrow(py);
            model::Method {
                name: m.name.clone(),
                type_annotation: m.type_annotation.clone(),
                description: m.description.clone(),
            }
        })
        .collect()
}

fn extract_see_also(py: Python<'_>, entries: &[Py<PyModelSeeAlsoEntry>]) -> Vec<model::SeeAlsoEntry> {
    entries
        .iter()
        .map(|s| {
            let s = s.borrow(py);
            model::SeeAlsoEntry {
                names: s.names.clone(),
                description: s.description.clone(),
            }
        })
        .collect()
}

fn extract_references(py: Python<'_>, entries: &[Py<PyModelReference>]) -> Vec<model::Reference> {
    entries
        .iter()
        .map(|r| {
            let r = r.borrow(py);
            model::Reference {
                number: r.number.clone(),
                content: r.content.clone(),
            }
        })
        .collect()
}

#[pyclass(name = "Section")]
#[derive(Clone)]
struct PyModelSection {
    inner: model::Section,
}

#[pymethods]
impl PyModelSection {
    #[new]
    #[pyo3(signature = (kind, *, parameters=None, returns=None, exceptions=None, attributes=None, methods=None, see_also_entries=None, references=None, body=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        kind: &str,
        parameters: Option<Vec<Py<PyModelParameter>>>,
        returns: Option<Vec<Py<PyModelReturn>>>,
        exceptions: Option<Vec<Py<PyModelExceptionEntry>>>,
        attributes: Option<Vec<Py<PyModelAttribute>>>,
        methods: Option<Vec<Py<PyModelMethod>>>,
        see_also_entries: Option<Vec<Py<PyModelSeeAlsoEntry>>>,
        references: Option<Vec<Py<PyModelReference>>>,
        body: Option<String>,
    ) -> PyResult<Self> {
        let inner = match kind {
            "parameters" => model::Section::Parameters(extract_parameters(py, &parameters.unwrap_or_default())),
            "keyword_parameters" => {
                model::Section::KeywordParameters(extract_parameters(py, &parameters.unwrap_or_default()))
            }
            "other_parameters" => {
                model::Section::OtherParameters(extract_parameters(py, &parameters.unwrap_or_default()))
            }
            "receives" => model::Section::Receives(extract_parameters(py, &parameters.unwrap_or_default())),
            "returns" => model::Section::Returns(extract_returns(py, &returns.unwrap_or_default())),
            "yields" => model::Section::Yields(extract_returns(py, &returns.unwrap_or_default())),
            "raises" => model::Section::Raises(extract_exceptions(py, &exceptions.unwrap_or_default())),
            "warns" => model::Section::Warns(extract_exceptions(py, &exceptions.unwrap_or_default())),
            "attributes" => model::Section::Attributes(extract_attributes(py, &attributes.unwrap_or_default())),
            "methods" => model::Section::Methods(extract_methods(py, &methods.unwrap_or_default())),
            "see_also" => model::Section::SeeAlso(extract_see_also(py, &see_also_entries.unwrap_or_default())),
            "references" => model::Section::References(extract_references(py, &references.unwrap_or_default())),
            other => model::Section::FreeText {
                kind: str_to_free_section_kind(other),
                body: body.unwrap_or_default(),
            },
        };
        Ok(Self { inner })
    }

    #[getter]
    fn kind(&self) -> &str {
        match &self.inner {
            model::Section::Parameters(_) => "parameters",
            model::Section::KeywordParameters(_) => "keyword_parameters",
            model::Section::OtherParameters(_) => "other_parameters",
            model::Section::Receives(_) => "receives",
            model::Section::Returns(_) => "returns",
            model::Section::Yields(_) => "yields",
            model::Section::Raises(_) => "raises",
            model::Section::Warns(_) => "warns",
            model::Section::Attributes(_) => "attributes",
            model::Section::Methods(_) => "methods",
            model::Section::SeeAlso(_) => "see_also",
            model::Section::References(_) => "references",
            model::Section::FreeText { kind, .. } => free_section_kind_to_str(kind),
        }
    }

    #[getter]
    fn parameters(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelParameter>>> {
        match &self.inner {
            model::Section::Parameters(ps)
            | model::Section::KeywordParameters(ps)
            | model::Section::OtherParameters(ps)
            | model::Section::Receives(ps) => ps
                .iter()
                .map(|p| {
                    Py::new(
                        py,
                        PyModelParameter {
                            names: p.names.clone(),
                            type_annotation: p.type_annotation.clone(),
                            description: p.description.clone(),
                            is_optional: p.is_optional,
                            default_value: p.default_value.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn returns(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelReturn>>> {
        match &self.inner {
            model::Section::Returns(rs) | model::Section::Yields(rs) => rs
                .iter()
                .map(|r| {
                    Py::new(
                        py,
                        PyModelReturn {
                            name: r.name.clone(),
                            type_annotation: r.type_annotation.clone(),
                            description: r.description.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn exceptions(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelExceptionEntry>>> {
        match &self.inner {
            model::Section::Raises(es) | model::Section::Warns(es) => es
                .iter()
                .map(|e| {
                    Py::new(
                        py,
                        PyModelExceptionEntry {
                            type_name: e.type_name.clone(),
                            description: e.description.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn attributes(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelAttribute>>> {
        match &self.inner {
            model::Section::Attributes(attrs) => attrs
                .iter()
                .map(|a| {
                    Py::new(
                        py,
                        PyModelAttribute {
                            name: a.name.clone(),
                            type_annotation: a.type_annotation.clone(),
                            description: a.description.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn methods(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelMethod>>> {
        match &self.inner {
            model::Section::Methods(ms) => ms
                .iter()
                .map(|m| {
                    Py::new(
                        py,
                        PyModelMethod {
                            name: m.name.clone(),
                            type_annotation: m.type_annotation.clone(),
                            description: m.description.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn see_also_entries(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelSeeAlsoEntry>>> {
        match &self.inner {
            model::Section::SeeAlso(items) => items
                .iter()
                .map(|item| {
                    Py::new(
                        py,
                        PyModelSeeAlsoEntry {
                            names: item.names.clone(),
                            description: item.description.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn references(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelReference>>> {
        match &self.inner {
            model::Section::References(refs) => refs
                .iter()
                .map(|r| {
                    Py::new(
                        py,
                        PyModelReference {
                            number: r.number.clone(),
                            content: r.content.clone(),
                        },
                    )
                })
                .collect(),
            _ => Ok(vec![]),
        }
    }

    #[getter]
    fn body(&self) -> Option<String> {
        match &self.inner {
            model::Section::FreeText { body, .. } => Some(body.clone()),
            _ => None,
        }
    }

    fn __repr__(&self) -> String {
        format!("Section({})", self.kind())
    }
}

// ─── Model Docstring ────────────────────────────────────────────────────────

#[pyclass(name = "Docstring")]
#[derive(Clone)]
struct PyModelDocstring {
    inner: model::Docstring,
}

#[pymethods]
impl PyModelDocstring {
    #[new]
    #[pyo3(signature = (*, summary=None, extended_summary=None, deprecation=None, sections=None))]
    fn new(
        py: Python<'_>,
        summary: Option<String>,
        extended_summary: Option<String>,
        deprecation: Option<Py<PyModelDeprecation>>,
        sections: Option<Vec<Py<PyModelSection>>>,
    ) -> Self {
        Self {
            inner: model::Docstring {
                summary,
                extended_summary,
                deprecation: deprecation.map(|d| {
                    let d = d.borrow(py);
                    model::Deprecation {
                        version: d.version.clone(),
                        description: d.description.clone(),
                    }
                }),
                sections: sections
                    .map(|ss| ss.iter().map(|s| s.borrow(py).inner.clone()).collect())
                    .unwrap_or_default(),
            },
        }
    }

    #[getter]
    fn summary(&self) -> Option<&str> {
        self.inner.summary.as_deref()
    }

    #[setter]
    fn set_summary(&mut self, v: Option<String>) {
        self.inner.summary = v;
    }

    #[getter]
    fn extended_summary(&self) -> Option<&str> {
        self.inner.extended_summary.as_deref()
    }

    #[setter]
    fn set_extended_summary(&mut self, v: Option<String>) {
        self.inner.extended_summary = v;
    }

    #[getter]
    fn deprecation(&self, py: Python<'_>) -> PyResult<Option<Py<PyModelDeprecation>>> {
        self.inner
            .deprecation
            .as_ref()
            .map(|d| {
                Py::new(
                    py,
                    PyModelDeprecation {
                        version: d.version.clone(),
                        description: d.description.clone(),
                    },
                )
            })
            .transpose()
    }

    #[setter]
    fn set_deprecation(&mut self, dep: Option<Py<PyModelDeprecation>>) {
        Python::with_gil(|py| {
            self.inner.deprecation = dep.map(|d| {
                let d = d.borrow(py);
                model::Deprecation {
                    version: d.version.clone(),
                    description: d.description.clone(),
                }
            });
        });
    }

    #[getter]
    fn sections(&self, py: Python<'_>) -> PyResult<Vec<Py<PyModelSection>>> {
        self.inner
            .sections
            .iter()
            .map(|s| Py::new(py, PyModelSection { inner: s.clone() }))
            .collect()
    }

    #[setter]
    fn set_sections(&mut self, sections: Vec<Py<PyModelSection>>) {
        Python::with_gil(|py| {
            self.inner.sections = sections.iter().map(|s| s.borrow(py).inner.clone()).collect();
        });
    }

    fn __repr__(&self) -> String {
        format!("Docstring(summary={:?})", self.inner.summary)
    }
}

// ─── Module functions ───────────────────────────────────────────────────────

/// Parse a Google-style docstring and return a GoogleDocstring object.
#[pyfunction]
fn parse_google(py: Python<'_>, input: &str) -> PyResult<Py<PyGoogleDocstring>> {
    let parsed = google::parse_google(input);
    build_google_docstring(py, &parsed)
}

/// Parse a NumPy-style docstring and return a NumPyDocstring object.
#[pyfunction]
fn parse_numpy(py: Python<'_>, input: &str) -> PyResult<Py<PyNumPyDocstring>> {
    let parsed = pydocstring_core::parse::numpy::parse_numpy(input);
    build_numpy_docstring(py, &parsed)
}

/// Parse a plain docstring (no NumPy or Google section markers) and return a PlainDocstring object.
#[pyfunction]
fn parse_plain(py: Python<'_>, input: &str) -> PyResult<Py<PyPlainDocstring>> {
    let parsed = pydocstring_core::parse::plain::parse_plain(input);
    build_plain_docstring(py, &parsed)
}

/// Auto-detect the docstring style and parse it, returning a GoogleDocstring,
/// NumPyDocstring, or PlainDocstring. Use `.style` on the result to distinguish
/// between them without `isinstance` checks.
#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<pyo3::PyObject> {
    use pydocstring_core::syntax::SyntaxKind;
    let parsed = pydocstring_core::parse::parse(input);
    match parsed.root().kind() {
        SyntaxKind::GOOGLE_DOCSTRING => Ok(build_google_docstring(py, &parsed)?.into_any()),
        SyntaxKind::NUMPY_DOCSTRING => Ok(build_numpy_docstring(py, &parsed)?.into_any()),
        _ => Ok(build_plain_docstring(py, &parsed)?.into_any()),
    }
}

/// Docstring style enum.
#[pyclass(eq, eq_int, frozen, name = "Style")]
#[derive(Clone, PartialEq)]
enum PyStyle {
    #[pyo3(name = "GOOGLE")]
    Google,
    #[pyo3(name = "NUMPY")]
    NumPy,
    #[pyo3(name = "PLAIN")]
    Plain,
}

#[pymethods]
impl PyStyle {
    fn __repr__(&self) -> &'static str {
        match self {
            PyStyle::Google => "Style.GOOGLE",
            PyStyle::NumPy => "Style.NUMPY",
            PyStyle::Plain => "Style.PLAIN",
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            PyStyle::Google => "google",
            PyStyle::NumPy => "numpy",
            PyStyle::Plain => "plain",
        }
    }
}

/// Detect the docstring style.
#[pyfunction]
fn detect_style(input: &str) -> PyStyle {
    match pydocstring_core::parse::detect_style(input) {
        pydocstring_core::parse::Style::Google => PyStyle::Google,
        pydocstring_core::parse::Style::NumPy => PyStyle::NumPy,
        pydocstring_core::parse::Style::Plain => PyStyle::Plain,
    }
}

/// Emit a model Docstring as a Google-style docstring string.
#[pyfunction]
#[pyo3(name = "emit_google", signature = (doc, base_indent=0))]
fn py_emit_google(py: Python<'_>, doc: Py<PyModelDocstring>, base_indent: usize) -> String {
    let doc = doc.borrow(py);
    pydocstring_core::emit::google::emit_google(&doc.inner, base_indent)
}

/// Emit a model Docstring as a NumPy-style docstring string.
#[pyfunction]
#[pyo3(name = "emit_numpy", signature = (doc, base_indent=0))]
fn py_emit_numpy(py: Python<'_>, doc: Py<PyModelDocstring>, base_indent: usize) -> String {
    let doc = doc.borrow(py);
    pydocstring_core::emit::numpy::emit_numpy(&doc.inner, base_indent)
}

// ─── Module ─────────────────────────────────────────────────────────────────

#[pymodule]
fn pydocstring(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_google, m)?)?;
    m.add_function(wrap_pyfunction!(parse_numpy, m)?)?;
    m.add_function(wrap_pyfunction!(parse_plain, m)?)?;
    m.add_function(wrap_pyfunction!(detect_style, m)?)?;
    m.add_function(wrap_pyfunction!(py_emit_google, m)?)?;
    m.add_function(wrap_pyfunction!(py_emit_numpy, m)?)?;
    m.add_class::<PyStyle>()?;
    m.add_class::<PySyntaxKind>()?;
    m.add_class::<PyGoogleSectionKind>()?;
    m.add_class::<PyNumPySectionKind>()?;
    m.add_class::<PyTextRange>()?;
    m.add_class::<PyLineColumn>()?;
    m.add_class::<PyToken>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyDocstringWalker>()?;
    m.add_class::<PyGoogleDocstring>()?;
    m.add_class::<PyGoogleSection>()?;
    m.add_class::<PyGoogleArg>()?;
    m.add_class::<PyGoogleReturns>()?;
    m.add_class::<PyGoogleYields>()?;
    m.add_class::<PyGoogleException>()?;
    m.add_class::<PyGoogleWarning>()?;
    m.add_class::<PyGoogleSeeAlsoItem>()?;
    m.add_class::<PyGoogleAttribute>()?;
    m.add_class::<PyGoogleMethod>()?;
    m.add_class::<PyNumPyDocstring>()?;
    m.add_class::<PyNumPySection>()?;
    m.add_class::<PyNumPyParameter>()?;
    m.add_class::<PyNumPyReturns>()?;
    m.add_class::<PyNumPyYields>()?;
    m.add_class::<PyNumPyException>()?;
    m.add_class::<PyNumPyDeprecation>()?;
    m.add_class::<PyNumPyWarning>()?;
    m.add_class::<PyNumPySeeAlsoItem>()?;
    m.add_class::<PyNumPyReference>()?;
    m.add_class::<PyNumPyAttribute>()?;
    m.add_class::<PyNumPyMethod>()?;
    m.add_class::<PyPlainDocstring>()?;
    m.add_class::<PyModelDocstring>()?;
    m.add_class::<PyModelSection>()?;
    m.add_class::<PyModelParameter>()?;
    m.add_class::<PyModelReturn>()?;
    m.add_class::<PyModelExceptionEntry>()?;
    m.add_class::<PyModelSeeAlsoEntry>()?;
    m.add_class::<PyModelReference>()?;
    m.add_class::<PyModelAttribute>()?;
    m.add_class::<PyModelMethod>()?;
    m.add_class::<PyModelDeprecation>()?;

    // Add walk as a pure-Python function via exec
    let globals = pyo3::types::PyDict::new(m.py());
    m.py().run(
        pyo3::ffi::c_str!(
            "def walk(node):\n\
             \x20   \"\"\"Walk the syntax tree depth-first, yielding every Node and Token.\"\"\"\n\
             \x20   yield node\n\
             \x20   if hasattr(node, 'children'):\n\
             \x20       for child in node.children:\n\
             \x20           yield from walk(child)\n"
        ),
        Some(&globals),
        None,
    )?;
    m.add("walk", globals.get_item("walk")?.unwrap())?;

    Ok(())
}
