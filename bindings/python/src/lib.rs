use pyo3::prelude::*;

use pydocstring_core::parse::google;
use pydocstring_core::parse::google::nodes as gn;
use pydocstring_core::parse::numpy::nodes as nn;
use pydocstring_core::syntax::{Parsed, SyntaxNode, SyntaxToken};
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

// ─── Token ──────────────────────────────────────────────────────────────────

#[pyclass(name = "Token", frozen)]
struct PyToken {
    kind: String,
    text: String,
    range: Py<PyTextRange>,
}

#[pymethods]
impl PyToken {
    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }
    #[getter]
    fn text(&self) -> &str {
        &self.text
    }
    #[getter]
    fn range(&self, py: Python<'_>) -> Py<PyTextRange> {
        self.range.clone_ref(py)
    }
    fn __repr__(&self) -> String {
        format!("Token({}, {:?})", self.kind, self.text)
    }
}

fn to_py_token(py: Python<'_>, token: &SyntaxToken, source: &str) -> PyResult<Py<PyToken>> {
    Py::new(
        py,
        PyToken {
            kind: token.kind().name().to_string(),
            text: token.text(source).to_string(),
            range: Py::new(py, PyTextRange::from(token.range()))?,
        },
    )
}

fn to_py_token_opt(
    py: Python<'_>,
    token: Option<&SyntaxToken>,
    source: &str,
) -> PyResult<Option<Py<PyToken>>> {
    token.map(|t| to_py_token(py, t, source)).transpose()
}

// ─── Node ───────────────────────────────────────────────────────────────────

#[pyclass(name = "Node", frozen)]
struct PyNode {
    kind: String,
    range: Py<PyTextRange>,
    children: Vec<PyObject>,
}

#[pymethods]
impl PyNode {
    #[getter]
    fn kind(&self) -> &str {
        &self.kind
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
        format!("Node({}, {} children)", self.kind, self.children.len())
    }
}

fn to_py_node(py: Python<'_>, node: &SyntaxNode, source: &str) -> PyResult<Py<PyNode>> {
    let children: Vec<PyObject> = node
        .children()
        .iter()
        .map(|child| match child {
            pydocstring_core::syntax::SyntaxElement::Node(n) => {
                Ok(to_py_node(py, n, source)?.into_any().into())
            }
            pydocstring_core::syntax::SyntaxElement::Token(t) => {
                Ok(to_py_token(py, t, source)?.into_any().into())
            }
        })
        .collect::<PyResult<Vec<_>>>()?;

    Py::new(
        py,
        PyNode {
            kind: node.kind().name().to_string(),
            range: Py::new(py, PyTextRange::from(node.range()))?,
            children,
        },
    )
}

// ─── Google typed wrappers ──────────────────────────────────────────────────

#[pyclass(name = "GoogleArg", frozen)]
struct PyGoogleArg {
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
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleArg({})", self.name.borrow(py).text)
    }
}

#[pyclass(name = "GoogleReturns", frozen)]
struct PyGoogleReturns {
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
}

#[pyclass(name = "GoogleException", frozen)]
struct PyGoogleException {
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
}

#[pyclass(name = "GoogleSection", frozen)]
struct PyGoogleSection {
    kind: String,
    args: Vec<Py<PyGoogleArg>>,
    returns: Option<Py<PyGoogleReturns>>,
    exceptions: Vec<Py<PyGoogleException>>,
    body_text: Option<Py<PyToken>>,
    node: Py<PyNode>,
}

#[pymethods]
impl PyGoogleSection {
    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }
    #[getter]
    fn args(&self, py: Python<'_>) -> Vec<Py<PyGoogleArg>> {
        self.args.iter().map(|a| a.clone_ref(py)).collect()
    }
    #[getter]
    fn returns(&self, py: Python<'_>) -> Option<Py<PyGoogleReturns>> {
        self.returns.as_ref().map(|r| r.clone_ref(py))
    }
    #[getter]
    fn exceptions(&self, py: Python<'_>) -> Vec<Py<PyGoogleException>> {
        self.exceptions.iter().map(|e| e.clone_ref(py)).collect()
    }
    #[getter]
    fn body_text(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.body_text.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    fn __repr__(&self) -> String {
        format!("GoogleSection({})", self.kind)
    }
}

#[pyclass(name = "GoogleDocstring", frozen)]
struct PyGoogleDocstring {
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    sections: Vec<Py<PyGoogleSection>>,
    node: Py<PyNode>,
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
    fn sections(&self, py: Python<'_>) -> Vec<Py<PyGoogleSection>> {
        self.sections.iter().map(|s| s.clone_ref(py)).collect()
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    fn pretty_print(&self) -> String {
        let parsed = google::parse_google(&self.source);
        parsed.pretty_print()
    }
    fn __repr__(&self) -> String {
        "GoogleDocstring(...)".to_string()
    }
}

// ─── NumPy typed wrappers ───────────────────────────────────────────────────

#[pyclass(name = "NumPyParameter", frozen)]
struct PyNumPyParameter {
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
    fn __repr__(&self, py: Python<'_>) -> String {
        let name_texts: Vec<String> = self
            .names
            .iter()
            .map(|n| n.borrow(py).text.clone())
            .collect();
        format!("NumPyParameter({})", name_texts.join(", "))
    }
}

#[pyclass(name = "NumPyReturns", frozen)]
struct PyNumPyReturns {
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
}

#[pyclass(name = "NumPyException", frozen)]
struct PyNumPyException {
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
}

#[pyclass(name = "NumPySection", frozen)]
struct PyNumPySection {
    kind: String,
    parameters: Vec<Py<PyNumPyParameter>>,
    returns: Vec<Py<PyNumPyReturns>>,
    exceptions: Vec<Py<PyNumPyException>>,
    body_text: Option<Py<PyToken>>,
    node: Py<PyNode>,
}

#[pymethods]
impl PyNumPySection {
    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }
    #[getter]
    fn parameters(&self, py: Python<'_>) -> Vec<Py<PyNumPyParameter>> {
        self.parameters.iter().map(|p| p.clone_ref(py)).collect()
    }
    #[getter]
    fn returns(&self, py: Python<'_>) -> Vec<Py<PyNumPyReturns>> {
        self.returns.iter().map(|r| r.clone_ref(py)).collect()
    }
    #[getter]
    fn exceptions(&self, py: Python<'_>) -> Vec<Py<PyNumPyException>> {
        self.exceptions.iter().map(|e| e.clone_ref(py)).collect()
    }
    #[getter]
    fn body_text(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.body_text.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    fn __repr__(&self) -> String {
        format!("NumPySection({})", self.kind)
    }
}

#[pyclass(name = "NumPyDocstring", frozen)]
struct PyNumPyDocstring {
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    sections: Vec<Py<PyNumPySection>>,
    node: Py<PyNode>,
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
    fn sections(&self, py: Python<'_>) -> Vec<Py<PyNumPySection>> {
        self.sections.iter().map(|s| s.clone_ref(py)).collect()
    }
    #[getter]
    fn node(&self, py: Python<'_>) -> Py<PyNode> {
        self.node.clone_ref(py)
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    fn pretty_print(&self) -> String {
        let parsed = pydocstring_core::parse::numpy::parse_numpy(&self.source);
        parsed.pretty_print()
    }
    fn __repr__(&self) -> String {
        "NumPyDocstring(...)".to_string()
    }
}

// ─── Conversion helpers ─────────────────────────────────────────────────────

fn build_google_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyGoogleDocstring>> {
    let source = parsed.source();
    let doc = gn::GoogleDocstring::cast(parsed.root()).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("root node is not a GOOGLE_DOCSTRING")
    })?;

    let summary = to_py_token_opt(py, doc.summary(), source)?;
    let extended_summary = to_py_token_opt(py, doc.extended_summary(), source)?;

    let sections: Vec<Py<PyGoogleSection>> = doc
        .sections()
        .map(|sec| {
            let kind = format!("{}", sec.section_kind(source));
            let args: Vec<Py<PyGoogleArg>> = sec
                .args()
                .map(|a| {
                    Py::new(
                        py,
                        PyGoogleArg {
                            name: to_py_token(py, a.name(), source)?,
                            r#type: to_py_token_opt(py, a.r#type(), source)?,
                            description: to_py_token_opt(py, a.description(), source)?,
                            optional: to_py_token_opt(py, a.optional(), source)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<_>>>()?;
            let returns = sec
                .returns()
                .map(|r| {
                    Py::new(
                        py,
                        PyGoogleReturns {
                            return_type: to_py_token_opt(py, r.return_type(), source)?,
                            description: to_py_token_opt(py, r.description(), source)?,
                        },
                    )
                })
                .transpose()?;
            let exceptions: Vec<Py<PyGoogleException>> = sec
                .exceptions()
                .map(|e| {
                    Py::new(
                        py,
                        PyGoogleException {
                            r#type: to_py_token(py, e.r#type(), source)?,
                            description: to_py_token_opt(py, e.description(), source)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<_>>>()?;
            let body_text = to_py_token_opt(py, sec.body_text(), source)?;
            let node = to_py_node(py, sec.syntax(), source)?;
            Py::new(
                py,
                PyGoogleSection {
                    kind,
                    args,
                    returns,
                    exceptions,
                    body_text,
                    node,
                },
            )
        })
        .collect::<PyResult<Vec<_>>>()?;

    let node = to_py_node(py, parsed.root(), source)?;

    Py::new(
        py,
        PyGoogleDocstring {
            summary,
            extended_summary,
            sections,
            node,
            source: source.to_string(),
        },
    )
}

fn build_numpy_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyNumPyDocstring>> {
    let source = parsed.source();
    let doc = nn::NumPyDocstring::cast(parsed.root()).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("root node is not a NUMPY_DOCSTRING")
    })?;

    let summary = to_py_token_opt(py, doc.summary(), source)?;
    let extended_summary = to_py_token_opt(py, doc.extended_summary(), source)?;

    let sections: Vec<Py<PyNumPySection>> = doc
        .sections()
        .map(|sec| {
            let kind = format!("{}", sec.section_kind(source));
            let parameters: Vec<Py<PyNumPyParameter>> = sec
                .parameters()
                .map(|p| {
                    let names: Vec<Py<PyToken>> = p
                        .names()
                        .map(|n| to_py_token(py, n, source))
                        .collect::<PyResult<Vec<_>>>()?;
                    Py::new(
                        py,
                        PyNumPyParameter {
                            names,
                            r#type: to_py_token_opt(py, p.r#type(), source)?,
                            description: to_py_token_opt(py, p.description(), source)?,
                            optional: to_py_token_opt(py, p.optional(), source)?,
                            default_value: to_py_token_opt(py, p.default_value(), source)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<_>>>()?;
            let returns: Vec<Py<PyNumPyReturns>> = sec
                .returns()
                .map(|r| {
                    Py::new(
                        py,
                        PyNumPyReturns {
                            name: to_py_token_opt(py, r.name(), source)?,
                            return_type: to_py_token_opt(py, r.return_type(), source)?,
                            description: to_py_token_opt(py, r.description(), source)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<_>>>()?;
            let exceptions: Vec<Py<PyNumPyException>> = sec
                .exceptions()
                .map(|e| {
                    Py::new(
                        py,
                        PyNumPyException {
                            r#type: to_py_token(py, e.r#type(), source)?,
                            description: to_py_token_opt(py, e.description(), source)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<_>>>()?;
            let body_text = to_py_token_opt(py, sec.body_text(), source)?;
            let node = to_py_node(py, sec.syntax(), source)?;
            Py::new(
                py,
                PyNumPySection {
                    kind,
                    parameters,
                    returns,
                    exceptions,
                    body_text,
                    node,
                },
            )
        })
        .collect::<PyResult<Vec<_>>>()?;

    let node = to_py_node(py, parsed.root(), source)?;

    Py::new(
        py,
        PyNumPyDocstring {
            summary,
            extended_summary,
            sections,
            node,
            source: source.to_string(),
        },
    )
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

/// Docstring style enum.
#[pyclass(eq, eq_int, frozen, name = "Style")]
#[derive(Clone, PartialEq)]
enum PyStyle {
    #[pyo3(name = "GOOGLE")]
    Google,
    #[pyo3(name = "NUMPY")]
    NumPy,
}

#[pymethods]
impl PyStyle {
    fn __repr__(&self) -> &'static str {
        match self {
            PyStyle::Google => "Style.GOOGLE",
            PyStyle::NumPy => "Style.NUMPY",
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            PyStyle::Google => "google",
            PyStyle::NumPy => "numpy",
        }
    }
}

/// Detect the docstring style.
#[pyfunction]
fn detect_style(input: &str) -> PyStyle {
    match pydocstring_core::parse::detect_style(input) {
        pydocstring_core::parse::Style::Google => PyStyle::Google,
        pydocstring_core::parse::Style::NumPy => PyStyle::NumPy,
    }
}

// ─── Module ─────────────────────────────────────────────────────────────────

#[pymodule]
fn pydocstring(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_google, m)?)?;
    m.add_function(wrap_pyfunction!(parse_numpy, m)?)?;
    m.add_function(wrap_pyfunction!(detect_style, m)?)?;
    m.add_class::<PyStyle>()?;
    m.add_class::<PyTextRange>()?;
    m.add_class::<PyToken>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyGoogleDocstring>()?;
    m.add_class::<PyGoogleSection>()?;
    m.add_class::<PyGoogleArg>()?;
    m.add_class::<PyGoogleReturns>()?;
    m.add_class::<PyGoogleException>()?;
    m.add_class::<PyNumPyDocstring>()?;
    m.add_class::<PyNumPySection>()?;
    m.add_class::<PyNumPyParameter>()?;
    m.add_class::<PyNumPyReturns>()?;
    m.add_class::<PyNumPyException>()?;

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
