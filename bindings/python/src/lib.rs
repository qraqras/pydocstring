use pyo3::prelude::*;

use pydocstring_core::model;
use pydocstring_core::parse::google;
use pydocstring_core::parse::google::kind::GoogleSectionKind;
use pydocstring_core::parse::google::nodes as gn;
use pydocstring_core::parse::numpy::kind::NumPySectionKind;
use pydocstring_core::parse::numpy::nodes as nn;
use pydocstring_core::parse::visitor::{DocstringVisitor, walk_node as core_walk_node};
use pydocstring_core::syntax::{Parsed, SyntaxToken};
use pydocstring_core::text::TextRange;
use std::sync::Arc;

// ─── TextRange ──────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "TextRange")]
#[derive(Clone, Copy)]
struct PyTextRange {
    #[pyo3(get)]
    start: u32,
    #[pyo3(get)]
    end: u32,
}

impl From<TextRange> for PyTextRange {
    fn from(r: TextRange) -> Self {
        Self {
            start: r.start().raw(),
            end: r.end().raw(),
        }
    }
}

#[pymethods]
impl PyTextRange {
    fn __repr__(&self) -> String {
        format!("TextRange({}..{})", self.start, self.end)
    }
}

// ─── LineColumn ─────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "LineColumn")]
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
    if !source.is_char_boundary(byte_offset) || !source.is_char_boundary(line_start) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "offset is not on a UTF-8 character boundary",
        ));
    }
    let col = source[line_start..byte_offset].chars().count() as u32;
    Ok(PyLineColumn { lineno, col })
}

// ─── Token ──────────────────────────────────────────────────────────────────

/// A typed token: a text fragment plus its byte range in the source.
///
/// The field name on the parent object (e.g. `.name`, `.description`) implies
/// the semantic kind; no redundant `kind` field is exposed.
#[pyclass(frozen, name = "Token")]
struct PyToken {
    text: String,
    range: TextRange,
}

#[pymethods]
impl PyToken {
    #[getter]
    fn text(&self) -> &str {
        &self.text
    }
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    fn __repr__(&self) -> String {
        format!("Token({:?})", self.text)
    }
}

// ─── Token helpers ───────────────────────────────────────────────────────────

fn mk_token(py: Python<'_>, token: &SyntaxToken, source: &str) -> PyResult<Py<PyToken>> {
    Py::new(
        py,
        PyToken {
            text: token.text(source).to_string(),
            range: *token.range(),
        },
    )
}

fn mk_token_opt(py: Python<'_>, token: Option<&SyntaxToken>, source: &str) -> PyResult<Option<Py<PyToken>>> {
    token.map(|t| mk_token(py, t, source)).transpose()
}

fn mk_tokens<'a>(
    py: Python<'_>,
    tokens: impl Iterator<Item = &'a SyntaxToken>,
    source: &str,
) -> PyResult<Vec<Py<PyToken>>> {
    tokens.map(|t| mk_token(py, t, source)).collect()
}

// ─── Style ──────────────────────────────────────────────────────────────────

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

// ─── GoogleSectionKind ───────────────────────────────────────────────────────

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

// ─── NumPySectionKind ────────────────────────────────────────────────────────

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

// =============================================================================
// Google typed wrappers
// =============================================================================

// ─── GoogleArg ───────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleArg")]
struct PyGoogleArg {
    range: TextRange,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
    optional: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleArg {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
        format!("GoogleArg({:?})", self.name.borrow(py).text)
    }
}

fn build_google_arg(py: Python<'_>, arg: &gn::GoogleArg<'_>, source: &str) -> PyResult<Py<PyGoogleArg>> {
    Py::new(
        py,
        PyGoogleArg {
            range: *arg.syntax().range(),
            name: mk_token(py, arg.name(), source)?,
            r#type: mk_token_opt(py, arg.r#type(), source)?,
            description: mk_token_opt(py, arg.description(), source)?,
            optional: mk_token_opt(py, arg.optional(), source)?,
        },
    )
}

// ─── GoogleReturn ────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleReturn")]
struct PyGoogleReturn {
    range: TextRange,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleReturn {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self) -> &'static str {
        "GoogleReturn(...)"
    }
}

fn build_google_return(py: Python<'_>, rtn: &gn::GoogleReturn<'_>, source: &str) -> PyResult<Py<PyGoogleReturn>> {
    Py::new(
        py,
        PyGoogleReturn {
            range: *rtn.syntax().range(),
            return_type: mk_token_opt(py, rtn.return_type(), source)?,
            description: mk_token_opt(py, rtn.description(), source)?,
        },
    )
}

// ─── GoogleYield ─────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleYield")]
struct PyGoogleYield {
    range: TextRange,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleYield {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn return_type(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.return_type.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self) -> &'static str {
        "GoogleYield(...)"
    }
}

fn build_google_yield(py: Python<'_>, yld: &gn::GoogleYield<'_>, source: &str) -> PyResult<Py<PyGoogleYield>> {
    Py::new(
        py,
        PyGoogleYield {
            range: *yld.syntax().range(),
            return_type: mk_token_opt(py, yld.return_type(), source)?,
            description: mk_token_opt(py, yld.description(), source)?,
        },
    )
}

// ─── GoogleException ─────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleException")]
struct PyGoogleException {
    range: TextRange,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleException {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleException({:?})", self.r#type.borrow(py).text)
    }
}

fn build_google_exception(
    py: Python<'_>,
    exc: &gn::GoogleException<'_>,
    source: &str,
) -> PyResult<Py<PyGoogleException>> {
    Py::new(
        py,
        PyGoogleException {
            range: *exc.syntax().range(),
            r#type: mk_token(py, exc.r#type(), source)?,
            description: mk_token_opt(py, exc.description(), source)?,
        },
    )
}

// ─── GoogleWarning ───────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleWarning")]
struct PyGoogleWarning {
    range: TextRange,
    warning_type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleWarning {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn warning_type(&self, py: Python<'_>) -> Py<PyToken> {
        self.warning_type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleWarning({:?})", self.warning_type.borrow(py).text)
    }
}

fn build_google_warning(py: Python<'_>, wrn: &gn::GoogleWarning<'_>, source: &str) -> PyResult<Py<PyGoogleWarning>> {
    Py::new(
        py,
        PyGoogleWarning {
            range: *wrn.syntax().range(),
            warning_type: mk_token(py, wrn.warning_type(), source)?,
            description: mk_token_opt(py, wrn.description(), source)?,
        },
    )
}

// ─── GoogleSeeAlsoItem ───────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleSeeAlsoItem")]
struct PyGoogleSeeAlsoItem {
    range: TextRange,
    names: Vec<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleSeeAlsoItem {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn names(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.names.iter().map(|n| n.clone_ref(py)).collect()
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self) -> &'static str {
        "GoogleSeeAlsoItem(...)"
    }
}

fn build_google_see_also_item(
    py: Python<'_>,
    sai: &gn::GoogleSeeAlsoItem<'_>,
    source: &str,
) -> PyResult<Py<PyGoogleSeeAlsoItem>> {
    Py::new(
        py,
        PyGoogleSeeAlsoItem {
            range: *sai.syntax().range(),
            names: mk_tokens(py, sai.names(), source)?,
            description: mk_token_opt(py, sai.description(), source)?,
        },
    )
}

// ─── GoogleAttribute ─────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleAttribute")]
struct PyGoogleAttribute {
    range: TextRange,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleAttribute {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleAttribute({:?})", self.name.borrow(py).text)
    }
}

fn build_google_attribute(
    py: Python<'_>,
    att: &gn::GoogleAttribute<'_>,
    source: &str,
) -> PyResult<Py<PyGoogleAttribute>> {
    Py::new(
        py,
        PyGoogleAttribute {
            range: *att.syntax().range(),
            name: mk_token(py, att.name(), source)?,
            r#type: mk_token_opt(py, att.r#type(), source)?,
            description: mk_token_opt(py, att.description(), source)?,
        },
    )
}

// ─── GoogleMethod ────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleMethod")]
struct PyGoogleMethod {
    range: TextRange,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyGoogleMethod {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleMethod({:?})", self.name.borrow(py).text)
    }
}

fn build_google_method(py: Python<'_>, mtd: &gn::GoogleMethod<'_>, source: &str) -> PyResult<Py<PyGoogleMethod>> {
    Py::new(
        py,
        PyGoogleMethod {
            range: *mtd.syntax().range(),
            name: mk_token(py, mtd.name(), source)?,
            r#type: mk_token_opt(py, mtd.r#type(), source)?,
            description: mk_token_opt(py, mtd.description(), source)?,
        },
    )
}

// ─── GoogleSection ───────────────────────────────────────────────────────────

/// A thin wrapper for a Google section node (no eager child allocation).
#[pyclass(frozen, name = "GoogleSection")]
struct PyGoogleSection {
    range: TextRange,
    section_kind: PyGoogleSectionKind,
    header_name: Py<PyToken>,
}

#[pymethods]
impl PyGoogleSection {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn section_kind(&self) -> PyGoogleSectionKind {
        self.section_kind
    }
    #[getter]
    fn header_name(&self, py: Python<'_>) -> Py<PyToken> {
        self.header_name.clone_ref(py)
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("GoogleSection({:?})", self.header_name.borrow(py).text)
    }
}

fn build_google_section(py: Python<'_>, sec: &gn::GoogleSection<'_>, source: &str) -> PyResult<Py<PyGoogleSection>> {
    Py::new(
        py,
        PyGoogleSection {
            range: *sec.syntax().range(),
            section_kind: google_section_kind_to_py(sec.section_kind(source)),
            header_name: mk_token(py, sec.header().name(), source)?,
        },
    )
}

// ─── GoogleDocstring ─────────────────────────────────────────────────────────

#[pyclass(frozen, name = "GoogleDocstring")]
struct PyGoogleDocstring {
    range: TextRange,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    stray_lines: Vec<Py<PyToken>>,
    sections: Vec<Py<PyGoogleSection>>,
    source: String,
    /// Cached CST — avoids re-parsing when `walk()` is called.
    parsed: Arc<Parsed>,
}

#[pymethods]
impl PyGoogleDocstring {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn extended_summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.extended_summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn stray_lines(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.stray_lines.iter().map(|t| t.clone_ref(py)).collect()
    }
    #[getter]
    fn sections(&self, py: Python<'_>) -> Vec<Py<PyGoogleSection>> {
        self.sections.iter().map(|s| s.clone_ref(py)).collect()
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::Google
    }
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        Py::new(py, byte_offset_to_line_col(&self.source, offset as usize)?)
    }
    fn pretty_print(&self) -> String {
        google::parse_google(&self.source).pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = google::parse_google(&self.source);
        pydocstring_core::parse::google::to_model::to_model(&parsed)
            .map(|doc| PyModelDocstring { inner: doc })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))
    }
    fn __repr__(&self) -> &'static str {
        "GoogleDocstring(...)"
    }
}

fn build_google_docstring_node(
    py: Python<'_>,
    doc: &gn::GoogleDocstring<'_>,
    source: &str,
    parsed: Arc<Parsed>,
) -> PyResult<Py<PyGoogleDocstring>> {
    let summary = mk_token_opt(py, doc.summary(), source)?;
    let extended_summary = mk_token_opt(py, doc.extended_summary(), source)?;
    let stray_lines = mk_tokens(py, doc.stray_lines(), source)?;
    let sections = doc
        .sections()
        .map(|sec| build_google_section(py, &sec, source))
        .collect::<PyResult<_>>()?;
    Py::new(
        py,
        PyGoogleDocstring {
            range: *doc.syntax().range(),
            summary,
            extended_summary,
            stray_lines,
            sections,
            source: source.to_string(),
            parsed,
        },
    )
}

fn build_google_docstring(py: Python<'_>, parsed: Parsed) -> PyResult<Py<PyGoogleDocstring>> {
    let arc = Arc::new(parsed);
    let arc2 = Arc::clone(&arc);
    let doc = gn::GoogleDocstring::cast(arc.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root is not GOOGLE_DOCSTRING"))?;
    build_google_docstring_node(py, &doc, arc.source(), arc2)
}

// =============================================================================
// NumPy typed wrappers
// =============================================================================

// ─── NumPyDeprecation ────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyDeprecation")]
struct PyNumPyDeprecation {
    range: TextRange,
    version: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyDeprecation {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn version(&self, py: Python<'_>) -> Py<PyToken> {
        self.version.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyDeprecation({:?})", self.version.borrow(py).text)
    }
}

fn build_numpy_deprecation(
    py: Python<'_>,
    dep: &nn::NumPyDeprecation<'_>,
    source: &str,
) -> PyResult<Py<PyNumPyDeprecation>> {
    Py::new(
        py,
        PyNumPyDeprecation {
            range: *dep.syntax().range(),
            version: mk_token(py, dep.version(), source)?,
            description: mk_token_opt(py, dep.description(), source)?,
        },
    )
}

// ─── NumPyParameter ──────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyParameter")]
struct PyNumPyParameter {
    range: TextRange,
    names: Vec<Py<PyToken>>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
    optional: Option<Py<PyToken>>,
    default_value: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyParameter {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
        let first = self
            .names
            .first()
            .map(|n| n.borrow(py).text.clone())
            .unwrap_or_default();
        format!("NumPyParameter({:?})", first)
    }
}

fn build_numpy_parameter(py: Python<'_>, prm: &nn::NumPyParameter<'_>, source: &str) -> PyResult<Py<PyNumPyParameter>> {
    Py::new(
        py,
        PyNumPyParameter {
            range: *prm.syntax().range(),
            names: mk_tokens(py, prm.names(), source)?,
            r#type: mk_token_opt(py, prm.r#type(), source)?,
            description: mk_token_opt(py, prm.description(), source)?,
            optional: mk_token_opt(py, prm.optional(), source)?,
            default_value: mk_token_opt(py, prm.default_value(), source)?,
        },
    )
}

// ─── NumPyReturns ────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyReturns")]
struct PyNumPyReturns {
    range: TextRange,
    name: Option<Py<PyToken>>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyReturns {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    fn __repr__(&self) -> &'static str {
        "NumPyReturns(...)"
    }
}

fn build_numpy_returns(py: Python<'_>, rtn: &nn::NumPyReturns<'_>, source: &str) -> PyResult<Py<PyNumPyReturns>> {
    Py::new(
        py,
        PyNumPyReturns {
            range: *rtn.syntax().range(),
            name: mk_token_opt(py, rtn.name(), source)?,
            return_type: mk_token_opt(py, rtn.return_type(), source)?,
            description: mk_token_opt(py, rtn.description(), source)?,
        },
    )
}

// ─── NumPyYields ─────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyYields")]
struct PyNumPyYields {
    range: TextRange,
    name: Option<Py<PyToken>>,
    return_type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyYields {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    fn __repr__(&self) -> &'static str {
        "NumPyYields(...)"
    }
}

fn build_numpy_yields(py: Python<'_>, yld: &nn::NumPyYields<'_>, source: &str) -> PyResult<Py<PyNumPyYields>> {
    Py::new(
        py,
        PyNumPyYields {
            range: *yld.syntax().range(),
            name: mk_token_opt(py, yld.name(), source)?,
            return_type: mk_token_opt(py, yld.return_type(), source)?,
            description: mk_token_opt(py, yld.description(), source)?,
        },
    )
}

// ─── NumPyException ──────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyException")]
struct PyNumPyException {
    range: TextRange,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyException {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyException({:?})", self.r#type.borrow(py).text)
    }
}

fn build_numpy_exception(py: Python<'_>, exc: &nn::NumPyException<'_>, source: &str) -> PyResult<Py<PyNumPyException>> {
    Py::new(
        py,
        PyNumPyException {
            range: *exc.syntax().range(),
            r#type: mk_token(py, exc.r#type(), source)?,
            description: mk_token_opt(py, exc.description(), source)?,
        },
    )
}

// ─── NumPyWarning ────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyWarning")]
struct PyNumPyWarning {
    range: TextRange,
    r#type: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyWarning {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn r#type(&self, py: Python<'_>) -> Py<PyToken> {
        self.r#type.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyWarning({:?})", self.r#type.borrow(py).text)
    }
}

fn build_numpy_warning(py: Python<'_>, wrn: &nn::NumPyWarning<'_>, source: &str) -> PyResult<Py<PyNumPyWarning>> {
    Py::new(
        py,
        PyNumPyWarning {
            range: *wrn.syntax().range(),
            r#type: mk_token(py, wrn.r#type(), source)?,
            description: mk_token_opt(py, wrn.description(), source)?,
        },
    )
}

// ─── NumPySeeAlsoItem ────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPySeeAlsoItem")]
struct PyNumPySeeAlsoItem {
    range: TextRange,
    names: Vec<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPySeeAlsoItem {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn names(&self, py: Python<'_>) -> Vec<Py<PyToken>> {
        self.names.iter().map(|n| n.clone_ref(py)).collect()
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self) -> &'static str {
        "NumPySeeAlsoItem(...)"
    }
}

fn build_numpy_see_also_item(
    py: Python<'_>,
    sai: &nn::NumPySeeAlsoItem<'_>,
    source: &str,
) -> PyResult<Py<PyNumPySeeAlsoItem>> {
    Py::new(
        py,
        PyNumPySeeAlsoItem {
            range: *sai.syntax().range(),
            names: mk_tokens(py, sai.names(), source)?,
            description: mk_token_opt(py, sai.description(), source)?,
        },
    )
}

// ─── NumPyReference ──────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyReference")]
struct PyNumPyReference {
    range: TextRange,
    number: Option<Py<PyToken>>,
    content: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyReference {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn number(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.number.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn content(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.content.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self) -> &'static str {
        "NumPyReference(...)"
    }
}

fn build_numpy_reference(py: Python<'_>, r: &nn::NumPyReference<'_>, source: &str) -> PyResult<Py<PyNumPyReference>> {
    Py::new(
        py,
        PyNumPyReference {
            range: *r.syntax().range(),
            number: mk_token_opt(py, r.number(), source)?,
            content: mk_token_opt(py, r.content(), source)?,
        },
    )
}

// ─── NumPyAttribute ──────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyAttribute")]
struct PyNumPyAttribute {
    range: TextRange,
    name: Py<PyToken>,
    r#type: Option<Py<PyToken>>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyAttribute {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyAttribute({:?})", self.name.borrow(py).text)
    }
}

fn build_numpy_attribute(py: Python<'_>, att: &nn::NumPyAttribute<'_>, source: &str) -> PyResult<Py<PyNumPyAttribute>> {
    Py::new(
        py,
        PyNumPyAttribute {
            range: *att.syntax().range(),
            name: mk_token(py, att.name(), source)?,
            r#type: mk_token_opt(py, att.r#type(), source)?,
            description: mk_token_opt(py, att.description(), source)?,
        },
    )
}

// ─── NumPyMethod ─────────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyMethod")]
struct PyNumPyMethod {
    range: TextRange,
    name: Py<PyToken>,
    description: Option<Py<PyToken>>,
}

#[pymethods]
impl PyNumPyMethod {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyToken> {
        self.name.clone_ref(py)
    }
    #[getter]
    fn description(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.description.as_ref().map(|t| t.clone_ref(py))
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPyMethod({:?})", self.name.borrow(py).text)
    }
}

fn build_numpy_method(py: Python<'_>, mtd: &nn::NumPyMethod<'_>, source: &str) -> PyResult<Py<PyNumPyMethod>> {
    Py::new(
        py,
        PyNumPyMethod {
            range: *mtd.syntax().range(),
            name: mk_token(py, mtd.name(), source)?,
            description: mk_token_opt(py, mtd.description(), source)?,
        },
    )
}

// ─── NumPySection ────────────────────────────────────────────────────────────

/// A thin wrapper for a NumPy section node (no eager child allocation).
#[pyclass(frozen, name = "NumPySection")]
struct PyNumPySection {
    range: TextRange,
    section_kind: PyNumPySectionKind,
    header_name: Py<PyToken>,
}

#[pymethods]
impl PyNumPySection {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn section_kind(&self) -> PyNumPySectionKind {
        self.section_kind
    }
    #[getter]
    fn header_name(&self, py: Python<'_>) -> Py<PyToken> {
        self.header_name.clone_ref(py)
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("NumPySection({:?})", self.header_name.borrow(py).text)
    }
}

fn build_numpy_section(py: Python<'_>, sec: &nn::NumPySection<'_>, source: &str) -> PyResult<Py<PyNumPySection>> {
    Py::new(
        py,
        PyNumPySection {
            range: *sec.syntax().range(),
            section_kind: numpy_section_kind_to_py(sec.section_kind(source)),
            header_name: mk_token(py, sec.header().name(), source)?,
        },
    )
}

// ─── NumPyDocstring ──────────────────────────────────────────────────────────

#[pyclass(frozen, name = "NumPyDocstring")]
struct PyNumPyDocstring {
    range: TextRange,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    deprecation: Option<Py<PyNumPyDeprecation>>,
    sections: Vec<Py<PyNumPySection>>,
    source: String,
    /// Cached CST — avoids re-parsing when `walk()` is called.
    parsed: Arc<Parsed>,
}

#[pymethods]
impl PyNumPyDocstring {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
    #[getter]
    fn summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn extended_summary(&self, py: Python<'_>) -> Option<Py<PyToken>> {
        self.extended_summary.as_ref().map(|t| t.clone_ref(py))
    }
    #[getter]
    fn deprecation(&self, py: Python<'_>) -> Option<Py<PyNumPyDeprecation>> {
        self.deprecation.as_ref().map(|d| d.clone_ref(py))
    }
    #[getter]
    fn sections(&self, py: Python<'_>) -> Vec<Py<PyNumPySection>> {
        self.sections.iter().map(|s| s.clone_ref(py)).collect()
    }
    #[getter]
    fn source(&self) -> &str {
        &self.source
    }
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::NumPy
    }
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        Py::new(py, byte_offset_to_line_col(&self.source, offset as usize)?)
    }
    fn pretty_print(&self) -> String {
        pydocstring_core::parse::numpy::parse_numpy(&self.source).pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = pydocstring_core::parse::numpy::parse_numpy(&self.source);
        pydocstring_core::parse::numpy::to_model::to_model(&parsed)
            .map(|doc| PyModelDocstring { inner: doc })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))
    }
    fn __repr__(&self) -> &'static str {
        "NumPyDocstring(...)"
    }
}

fn build_numpy_docstring_node(
    py: Python<'_>,
    doc: &nn::NumPyDocstring<'_>,
    source: &str,
    parsed: Arc<Parsed>,
) -> PyResult<Py<PyNumPyDocstring>> {
    let summary = mk_token_opt(py, doc.summary(), source)?;
    let extended_summary = mk_token_opt(py, doc.extended_summary(), source)?;
    let deprecation = doc
        .deprecation()
        .map(|dep| build_numpy_deprecation(py, &dep, source))
        .transpose()?;
    let sections = doc
        .sections()
        .map(|sec| build_numpy_section(py, &sec, source))
        .collect::<PyResult<_>>()?;
    Py::new(
        py,
        PyNumPyDocstring {
            range: *doc.syntax().range(),
            summary,
            extended_summary,
            deprecation,
            sections,
            source: source.to_string(),
            parsed,
        },
    )
}

fn build_numpy_docstring(py: Python<'_>, parsed: Parsed) -> PyResult<Py<PyNumPyDocstring>> {
    let arc = Arc::new(parsed);
    let arc2 = Arc::clone(&arc);
    let doc = nn::NumPyDocstring::cast(arc.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root is not NUMPY_DOCSTRING"))?;
    build_numpy_docstring_node(py, &doc, arc.source(), arc2)
}

// =============================================================================
// Plain docstring
// =============================================================================

#[pyclass(frozen, name = "PlainDocstring")]
struct PyPlainDocstring {
    range: TextRange,
    summary: Option<Py<PyToken>>,
    extended_summary: Option<Py<PyToken>>,
    source: String,
}

#[pymethods]
impl PyPlainDocstring {
    #[getter]
    fn range(&self, py: Python<'_>) -> PyResult<Py<PyTextRange>> {
        Py::new(py, PyTextRange::from(self.range))
    }
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
    #[getter]
    fn style(&self) -> PyStyle {
        PyStyle::Plain
    }
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        Py::new(py, byte_offset_to_line_col(&self.source, offset as usize)?)
    }
    fn pretty_print(&self) -> String {
        pydocstring_core::parse::plain::parse_plain(&self.source).pretty_print()
    }
    fn to_model(&self) -> PyResult<PyModelDocstring> {
        let parsed = pydocstring_core::parse::plain::parse_plain(&self.source);
        pydocstring_core::parse::plain::to_model::to_model(&parsed)
            .map(|doc| PyModelDocstring { inner: doc })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("failed to convert to model"))
    }
    fn __repr__(&self) -> &'static str {
        "PlainDocstring(...)"
    }
}

fn build_plain_docstring(py: Python<'_>, parsed: &Parsed) -> PyResult<Py<PyPlainDocstring>> {
    let source = parsed.source();
    let doc = pydocstring_core::parse::plain::nodes::PlainDocstring::cast(parsed.root())
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("root is not PLAIN_DOCSTRING"))?;
    let summary = mk_token_opt(py, doc.summary(), source)?;
    let extended_summary = mk_token_opt(py, doc.extended_summary(), source)?;
    Py::new(
        py,
        PyPlainDocstring {
            range: *parsed.root().range(),
            summary,
            extended_summary,
            source: source.to_string(),
        },
    )
}

// =============================================================================
// Model IR types
// =============================================================================

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
        self.name
            .as_deref()
            .map_or_else(|| "Return(...)".to_string(), |n| format!("Return({})", n))
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
        self.number
            .as_deref()
            .map_or_else(|| "Reference(...)".to_string(), |n| format!("Reference({})", n))
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

// ─── Model section helpers ───────────────────────────────────────────────────

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

// ─── Model Section ───────────────────────────────────────────────────────────

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

// ─── Model Docstring ─────────────────────────────────────────────────────────

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

// =============================================================================
// Module functions
// =============================================================================

/// Parse a Google-style docstring.
#[pyfunction]
fn parse_google(py: Python<'_>, input: &str) -> PyResult<Py<PyGoogleDocstring>> {
    build_google_docstring(py, google::parse_google(input))
}

/// Parse a NumPy-style docstring.
#[pyfunction]
fn parse_numpy(py: Python<'_>, input: &str) -> PyResult<Py<PyNumPyDocstring>> {
    build_numpy_docstring(py, pydocstring_core::parse::numpy::parse_numpy(input))
}

/// Parse a plain docstring (no section markers).
#[pyfunction]
fn parse_plain(py: Python<'_>, input: &str) -> PyResult<Py<PyPlainDocstring>> {
    build_plain_docstring(py, &pydocstring_core::parse::plain::parse_plain(input))
}

/// Auto-detect the docstring style and parse it.
///
/// Returns a `GoogleDocstring`, `NumPyDocstring`, or `PlainDocstring`.
/// Use `.style` on the result to distinguish them without `isinstance` checks.
#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<PyObject> {
    use pydocstring_core::syntax::SyntaxKind;
    let parsed = pydocstring_core::parse::parse(input);
    let kind = parsed.root().kind();
    match kind {
        SyntaxKind::GOOGLE_DOCSTRING => Ok(build_google_docstring(py, parsed)?.into_any()),
        SyntaxKind::NUMPY_DOCSTRING => Ok(build_numpy_docstring(py, parsed)?.into_any()),
        _ => Ok(build_plain_docstring(py, &parsed)?.into_any()),
    }
}

/// Detect the docstring style without fully parsing.
#[pyfunction]
fn detect_style(input: &str) -> PyStyle {
    match pydocstring_core::parse::detect_style(input) {
        pydocstring_core::parse::Style::Google => PyStyle::Google,
        pydocstring_core::parse::Style::NumPy => PyStyle::NumPy,
        pydocstring_core::parse::Style::Plain => PyStyle::Plain,
    }
}

/// Emit a model `Docstring` as Google-style text.
#[pyfunction]
#[pyo3(name = "emit_google", signature = (doc, base_indent=0))]
fn py_emit_google(py: Python<'_>, doc: Py<PyModelDocstring>, base_indent: usize) -> String {
    pydocstring_core::emit::google::emit_google(&doc.borrow(py).inner, base_indent)
}

/// Emit a model `Docstring` as NumPy-style text.
#[pyfunction]
#[pyo3(name = "emit_numpy", signature = (doc, base_indent=0))]
fn py_emit_numpy(py: Python<'_>, doc: Py<PyModelDocstring>, base_indent: usize) -> String {
    pydocstring_core::emit::numpy::emit_numpy(&doc.borrow(py).inner, base_indent)
}

// =============================================================================
// walk() — CST-direct Python dispatch
// =============================================================================

// ─── WalkContext ─────────────────────────────────────────────────────────────

/// Context passed to every ``visit_*`` method during a ``walk()`` call.
///
/// Provides source-location helpers for the docstring currently being walked.
#[pyclass(frozen, name = "WalkContext")]
struct PyWalkContext {
    source: String,
    style: PyStyle,
}

#[pymethods]
impl PyWalkContext {
    /// Detected docstring style.
    #[getter]
    fn style(&self) -> PyStyle {
        self.style.clone()
    }
    /// Convert a byte offset into a ``LineColumn``.
    ///
    /// Equivalent to calling ``doc.line_col(offset)`` on the owning docstring.
    fn line_col(&self, py: Python<'_>, offset: u32) -> PyResult<Py<PyLineColumn>> {
        Py::new(py, byte_offset_to_line_col(&self.source, offset as usize)?)
    }
    fn __repr__(&self) -> &'static str {
        "WalkContext(...)"
    }
}

/// Which `visit_*` / `leave_*` methods the Python visitor defines.
///
/// Collected **once per `walk()` call** by inspecting the visitor object,
/// so `hasattr` is never called per-node.
struct ActiveMethods {
    // Google (visit)
    google_docstring: bool,
    google_section: bool,
    google_arg: bool,
    google_return: bool,
    google_yield: bool,
    google_exception: bool,
    google_warning: bool,
    google_see_also_item: bool,
    google_attribute: bool,
    google_method: bool,
    // Google (leave)
    leave_google_docstring: bool,
    leave_google_section: bool,
    leave_google_arg: bool,
    leave_google_return: bool,
    leave_google_yield: bool,
    leave_google_exception: bool,
    leave_google_warning: bool,
    leave_google_see_also_item: bool,
    leave_google_attribute: bool,
    leave_google_method: bool,
    // NumPy (visit)
    numpy_docstring: bool,
    numpy_deprecation: bool,
    numpy_section: bool,
    numpy_parameter: bool,
    numpy_returns: bool,
    numpy_yields: bool,
    numpy_exception: bool,
    numpy_warning: bool,
    numpy_see_also_item: bool,
    numpy_reference: bool,
    numpy_attribute: bool,
    numpy_method: bool,
    // NumPy (leave)
    leave_numpy_docstring: bool,
    leave_numpy_deprecation: bool,
    leave_numpy_section: bool,
    leave_numpy_parameter: bool,
    leave_numpy_returns: bool,
    leave_numpy_yields: bool,
    leave_numpy_exception: bool,
    leave_numpy_warning: bool,
    leave_numpy_see_also_item: bool,
    leave_numpy_reference: bool,
    leave_numpy_attribute: bool,
    leave_numpy_method: bool,
    // Plain
    plain_docstring: bool,
}

/// Inspect `visitor` once and return which `visit_*` / `leave_*` methods it defines.
fn collect_active(py: Python<'_>, visitor: &Py<PyAny>) -> PyResult<ActiveMethods> {
    let b = visitor.bind(py);
    Ok(ActiveMethods {
        // Google (visit)
        google_docstring: b.hasattr("visit_google_docstring")?,
        google_section: b.hasattr("visit_google_section")?,
        google_arg: b.hasattr("visit_google_arg")?,
        google_return: b.hasattr("visit_google_return")?,
        google_yield: b.hasattr("visit_google_yield")?,
        google_exception: b.hasattr("visit_google_exception")?,
        google_warning: b.hasattr("visit_google_warning")?,
        google_see_also_item: b.hasattr("visit_google_see_also_item")?,
        google_attribute: b.hasattr("visit_google_attribute")?,
        google_method: b.hasattr("visit_google_method")?,
        // Google (leave)
        leave_google_docstring: b.hasattr("leave_google_docstring")?,
        leave_google_section: b.hasattr("leave_google_section")?,
        leave_google_arg: b.hasattr("leave_google_arg")?,
        leave_google_return: b.hasattr("leave_google_return")?,
        leave_google_yield: b.hasattr("leave_google_yield")?,
        leave_google_exception: b.hasattr("leave_google_exception")?,
        leave_google_warning: b.hasattr("leave_google_warning")?,
        leave_google_see_also_item: b.hasattr("leave_google_see_also_item")?,
        leave_google_attribute: b.hasattr("leave_google_attribute")?,
        leave_google_method: b.hasattr("leave_google_method")?,
        // NumPy (visit)
        numpy_docstring: b.hasattr("visit_numpy_docstring")?,
        numpy_deprecation: b.hasattr("visit_numpy_deprecation")?,
        numpy_section: b.hasattr("visit_numpy_section")?,
        numpy_parameter: b.hasattr("visit_numpy_parameter")?,
        numpy_returns: b.hasattr("visit_numpy_returns")?,
        numpy_yields: b.hasattr("visit_numpy_yields")?,
        numpy_exception: b.hasattr("visit_numpy_exception")?,
        numpy_warning: b.hasattr("visit_numpy_warning")?,
        numpy_see_also_item: b.hasattr("visit_numpy_see_also_item")?,
        numpy_reference: b.hasattr("visit_numpy_reference")?,
        numpy_attribute: b.hasattr("visit_numpy_attribute")?,
        numpy_method: b.hasattr("visit_numpy_method")?,
        // NumPy (leave)
        leave_numpy_docstring: b.hasattr("leave_numpy_docstring")?,
        leave_numpy_deprecation: b.hasattr("leave_numpy_deprecation")?,
        leave_numpy_section: b.hasattr("leave_numpy_section")?,
        leave_numpy_parameter: b.hasattr("leave_numpy_parameter")?,
        leave_numpy_returns: b.hasattr("leave_numpy_returns")?,
        leave_numpy_yields: b.hasattr("leave_numpy_yields")?,
        leave_numpy_exception: b.hasattr("leave_numpy_exception")?,
        leave_numpy_warning: b.hasattr("leave_numpy_warning")?,
        leave_numpy_see_also_item: b.hasattr("leave_numpy_see_also_item")?,
        leave_numpy_reference: b.hasattr("leave_numpy_reference")?,
        leave_numpy_attribute: b.hasattr("leave_numpy_attribute")?,
        leave_numpy_method: b.hasattr("leave_numpy_method")?,
        // Plain
        plain_docstring: b.hasattr("visit_plain_docstring")?,
    })
}

/// Call `visitor.method(arg, ctx)`.  The caller has already confirmed the method exists.
#[inline]
fn dispatch_with_ctx<T: pyo3::PyClass>(
    py: Python<'_>,
    visitor: &Py<PyAny>,
    method: &str,
    arg: Py<T>,
    ctx: &Py<PyWalkContext>,
) -> PyResult<()> {
    visitor.bind(py).call_method1(method, (arg.bind(py), ctx.bind(py)))?;
    Ok(())
}

/// Walk the typed children of a Google section, dispatching visitor methods.
///
/// Each child collection is built at most once and shared between
/// `visit_google_section` and per-child `visit_google_*` calls via `clone_ref`.
/// Walk the children of a section node, dispatching visitor methods.
///
/// Accepts either a `GOOGLE_SECTION` or `NUMPY_SECTION` node.  The section
/// kind is read from `node.kind()` — no per-style function needed.
/// Each child collection is built at most once and shared between the
/// section object and per-child dispatches via `clone_ref`.

// =============================================================================
// PyDispatcher — ANTLR-style Python dispatch via DocstringVisitor
// =============================================================================

/// Implements `DocstringVisitor` from the core crate.
///
/// For every node kind the pattern is:
/// 1. Call Python `visit_*` (enter) if the visitor defines it.
/// 2. Recurse into children via `core_walk_node`.
/// 3. Call Python `leave_*` (exit) if the visitor defines it.
struct PyDispatcher<'py> {
    py: Python<'py>,
    arc: Arc<Parsed>,
    visitor: Py<PyAny>,
    active: ActiveMethods,
    ctx: Py<PyWalkContext>,
}

impl<'py> DocstringVisitor for PyDispatcher<'py> {
    type Error = PyErr;

    // ── Google ────────────────────────────────────────────────────────────
    fn visit_google_docstring(&mut self, source: &str, doc: &gn::GoogleDocstring<'_>) -> Result<(), PyErr> {
        let need = self.active.google_docstring || self.active.leave_google_docstring;
        let obj = if need { Some(build_google_docstring_node(self.py, doc, source, Arc::clone(&self.arc))?) } else { None };
        if self.active.google_docstring {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_docstring", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, doc.syntax(), self)?;
        if self.active.leave_google_docstring {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_docstring", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_section(&mut self, source: &str, sec: &gn::GoogleSection<'_>) -> Result<(), PyErr> {
        let need = self.active.google_section || self.active.leave_google_section;
        let obj = if need { Some(build_google_section(self.py, sec, source)?) } else { None };
        if self.active.google_section {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_section", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, sec.syntax(), self)?;
        if self.active.leave_google_section {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_section", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_arg(&mut self, source: &str, arg: &gn::GoogleArg<'_>) -> Result<(), PyErr> {
        let need = self.active.google_arg || self.active.leave_google_arg;
        let obj = if need { Some(build_google_arg(self.py, arg, source)?) } else { None };
        if self.active.google_arg {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_arg", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, arg.syntax(), self)?;
        if self.active.leave_google_arg {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_arg", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_return(&mut self, source: &str, rtn: &gn::GoogleReturn<'_>) -> Result<(), PyErr> {
        let need = self.active.google_return || self.active.leave_google_return;
        let obj = if need { Some(build_google_return(self.py, rtn, source)?) } else { None };
        if self.active.google_return {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_return", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, rtn.syntax(), self)?;
        if self.active.leave_google_return {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_return", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_yield(&mut self, source: &str, yld: &gn::GoogleYield<'_>) -> Result<(), PyErr> {
        let need = self.active.google_yield || self.active.leave_google_yield;
        let obj = if need { Some(build_google_yield(self.py, yld, source)?) } else { None };
        if self.active.google_yield {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_yield", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, yld.syntax(), self)?;
        if self.active.leave_google_yield {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_yield", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_exception(&mut self, source: &str, exc: &gn::GoogleException<'_>) -> Result<(), PyErr> {
        let need = self.active.google_exception || self.active.leave_google_exception;
        let obj = if need { Some(build_google_exception(self.py, exc, source)?) } else { None };
        if self.active.google_exception {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_exception", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, exc.syntax(), self)?;
        if self.active.leave_google_exception {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_exception", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_warning(&mut self, source: &str, wrn: &gn::GoogleWarning<'_>) -> Result<(), PyErr> {
        let need = self.active.google_warning || self.active.leave_google_warning;
        let obj = if need { Some(build_google_warning(self.py, wrn, source)?) } else { None };
        if self.active.google_warning {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_warning", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, wrn.syntax(), self)?;
        if self.active.leave_google_warning {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_warning", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_see_also_item(&mut self, source: &str, sai: &gn::GoogleSeeAlsoItem<'_>) -> Result<(), PyErr> {
        let need = self.active.google_see_also_item || self.active.leave_google_see_also_item;
        let obj = if need { Some(build_google_see_also_item(self.py, sai, source)?) } else { None };
        if self.active.google_see_also_item {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_see_also_item", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, sai.syntax(), self)?;
        if self.active.leave_google_see_also_item {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_see_also_item", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_attribute(&mut self, source: &str, att: &gn::GoogleAttribute<'_>) -> Result<(), PyErr> {
        let need = self.active.google_attribute || self.active.leave_google_attribute;
        let obj = if need { Some(build_google_attribute(self.py, att, source)?) } else { None };
        if self.active.google_attribute {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_attribute", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, att.syntax(), self)?;
        if self.active.leave_google_attribute {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_attribute", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_google_method(&mut self, source: &str, mtd: &gn::GoogleMethod<'_>) -> Result<(), PyErr> {
        let need = self.active.google_method || self.active.leave_google_method;
        let obj = if need { Some(build_google_method(self.py, mtd, source)?) } else { None };
        if self.active.google_method {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_google_method", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, mtd.syntax(), self)?;
        if self.active.leave_google_method {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_google_method", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    // ── NumPy ─────────────────────────────────────────────────────────────
    fn visit_numpy_docstring(&mut self, source: &str, doc: &nn::NumPyDocstring<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_docstring || self.active.leave_numpy_docstring;
        let obj = if need { Some(build_numpy_docstring_node(self.py, doc, source, Arc::clone(&self.arc))?) } else { None };
        if self.active.numpy_docstring {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_docstring", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, doc.syntax(), self)?;
        if self.active.leave_numpy_docstring {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_docstring", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_deprecation(&mut self, source: &str, dep: &nn::NumPyDeprecation<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_deprecation || self.active.leave_numpy_deprecation;
        let obj = if need { Some(build_numpy_deprecation(self.py, dep, source)?) } else { None };
        if self.active.numpy_deprecation {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_deprecation", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, dep.syntax(), self)?;
        if self.active.leave_numpy_deprecation {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_deprecation", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_section(&mut self, source: &str, sec: &nn::NumPySection<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_section || self.active.leave_numpy_section;
        let obj = if need { Some(build_numpy_section(self.py, sec, source)?) } else { None };
        if self.active.numpy_section {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_section", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, sec.syntax(), self)?;
        if self.active.leave_numpy_section {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_section", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_parameter(&mut self, source: &str, prm: &nn::NumPyParameter<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_parameter || self.active.leave_numpy_parameter;
        let obj = if need { Some(build_numpy_parameter(self.py, prm, source)?) } else { None };
        if self.active.numpy_parameter {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_parameter", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, prm.syntax(), self)?;
        if self.active.leave_numpy_parameter {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_parameter", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_returns(&mut self, source: &str, rtn: &nn::NumPyReturns<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_returns || self.active.leave_numpy_returns;
        let obj = if need { Some(build_numpy_returns(self.py, rtn, source)?) } else { None };
        if self.active.numpy_returns {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_returns", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, rtn.syntax(), self)?;
        if self.active.leave_numpy_returns {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_returns", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_yields(&mut self, source: &str, yld: &nn::NumPyYields<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_yields || self.active.leave_numpy_yields;
        let obj = if need { Some(build_numpy_yields(self.py, yld, source)?) } else { None };
        if self.active.numpy_yields {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_yields", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, yld.syntax(), self)?;
        if self.active.leave_numpy_yields {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_yields", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_exception(&mut self, source: &str, exc: &nn::NumPyException<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_exception || self.active.leave_numpy_exception;
        let obj = if need { Some(build_numpy_exception(self.py, exc, source)?) } else { None };
        if self.active.numpy_exception {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_exception", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, exc.syntax(), self)?;
        if self.active.leave_numpy_exception {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_exception", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_warning(&mut self, source: &str, wrn: &nn::NumPyWarning<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_warning || self.active.leave_numpy_warning;
        let obj = if need { Some(build_numpy_warning(self.py, wrn, source)?) } else { None };
        if self.active.numpy_warning {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_warning", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, wrn.syntax(), self)?;
        if self.active.leave_numpy_warning {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_warning", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_see_also_item(&mut self, source: &str, sai: &nn::NumPySeeAlsoItem<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_see_also_item || self.active.leave_numpy_see_also_item;
        let obj = if need { Some(build_numpy_see_also_item(self.py, sai, source)?) } else { None };
        if self.active.numpy_see_also_item {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_see_also_item", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, sai.syntax(), self)?;
        if self.active.leave_numpy_see_also_item {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_see_also_item", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_reference(&mut self, source: &str, r#ref: &nn::NumPyReference<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_reference || self.active.leave_numpy_reference;
        let obj = if need { Some(build_numpy_reference(self.py, r#ref, source)?) } else { None };
        if self.active.numpy_reference {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_reference", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, r#ref.syntax(), self)?;
        if self.active.leave_numpy_reference {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_reference", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_attribute(&mut self, source: &str, att: &nn::NumPyAttribute<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_attribute || self.active.leave_numpy_attribute;
        let obj = if need { Some(build_numpy_attribute(self.py, att, source)?) } else { None };
        if self.active.numpy_attribute {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_attribute", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, att.syntax(), self)?;
        if self.active.leave_numpy_attribute {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_attribute", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }

    fn visit_numpy_method(&mut self, source: &str, mtd: &nn::NumPyMethod<'_>) -> Result<(), PyErr> {
        let need = self.active.numpy_method || self.active.leave_numpy_method;
        let obj = if need { Some(build_numpy_method(self.py, mtd, source)?) } else { None };
        if self.active.numpy_method {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "visit_numpy_method", o.clone_ref(self.py), &self.ctx)?; }
        }
        core_walk_node(source, mtd.syntax(), self)?;
        if self.active.leave_numpy_method {
            if let Some(ref o) = obj { dispatch_with_ctx(self.py, &self.visitor, "leave_numpy_method", o.clone_ref(self.py), &self.ctx)?; }
        }
        Ok(())
    }
}

/// Walk a plain docstring, dispatching ``visit_plain_docstring`` if defined.
fn walk_plain_cst(
    py: Python<'_>,
    plain_doc: Py<PyPlainDocstring>,
    visitor: &Py<PyAny>,
    active: &ActiveMethods,
) -> PyResult<()> {
    if !active.plain_docstring {
        return Ok(());
    }
    let source = plain_doc.borrow(py).source.clone();
    let ctx = Py::new(
        py,
        PyWalkContext {
            source,
            style: PyStyle::Plain,
        },
    )?;
    dispatch_with_ctx(py, visitor, "visit_plain_docstring", plain_doc, &ctx)?;
    Ok(())
}

/// Walk any docstring depth-first, calling typed methods on ``visitor`` for each node.
///
/// Accepts a `GoogleDocstring`, `NumPyDocstring`, or `PlainDocstring`.
/// The visitor defines only the methods it needs; all others are silently skipped.
/// Returns ``visitor`` so results can be collected inline.
///
/// Every ``visit_*`` method receives ``(node, ctx: WalkContext)`` where
/// ``ctx.line_col(offset)`` converts byte offsets to line/column positions.
///
/// ```python
/// class TypeAnnotationChecker:
///     def visit_google_arg(self, arg, ctx): ...
///     def visit_numpy_parameter(self, param, ctx): ...
///
/// for docstring_text in all_docstrings:
///     doc = pydocstring.parse(docstring_text)   # auto-detects style
///     checker = pydocstring.walk(doc, checker)  # returns visitor
/// ```
///
/// Google `visit_*` methods:
/// `visit_google_docstring`, `visit_google_section`, `visit_google_arg`,
/// `visit_google_return`, `visit_google_yield`, `visit_google_exception`,
/// `visit_google_warning`, `visit_google_see_also_item`,
/// `visit_google_attribute`, `visit_google_method`
///
/// NumPy `visit_*` methods:
/// `visit_numpy_docstring`, `visit_numpy_section`, `visit_numpy_deprecation`,
/// `visit_numpy_parameter`, `visit_numpy_returns`, `visit_numpy_yields`,
/// `visit_numpy_exception`, `visit_numpy_warning`, `visit_numpy_see_also_item`,
/// `visit_numpy_reference`, `visit_numpy_attribute`, `visit_numpy_method`
///
/// Plain `visit_*` methods:
/// `visit_plain_docstring`
#[pyfunction]
fn walk(py: Python<'_>, doc: PyObject, visitor: PyObject) -> PyResult<PyObject> {
    let bound = doc.bind(py);
    let active = collect_active(py, &visitor)?;
    if let Ok(google_doc) = bound.downcast::<PyGoogleDocstring>() {
        let arc = google_doc.borrow().parsed.clone();
        let source = arc.source().to_string();
        let ctx = Py::new(py, PyWalkContext { source: source.clone(), style: PyStyle::Google })?;
        let root = arc.root();
        if let Some(doc_node) = gn::GoogleDocstring::cast(root) {
            let mut dispatcher = PyDispatcher { py, arc: Arc::clone(&arc), visitor: visitor.clone_ref(py), active, ctx };
            dispatcher.visit_google_docstring(&source, &doc_node)?;
        }
    } else if let Ok(numpy_doc) = bound.downcast::<PyNumPyDocstring>() {
        let arc = numpy_doc.borrow().parsed.clone();
        let source = arc.source().to_string();
        let ctx = Py::new(py, PyWalkContext { source: source.clone(), style: PyStyle::NumPy })?;
        let root = arc.root();
        if let Some(doc_node) = nn::NumPyDocstring::cast(root) {
            let mut dispatcher = PyDispatcher { py, arc: Arc::clone(&arc), visitor: visitor.clone_ref(py), active, ctx };
            dispatcher.visit_numpy_docstring(&source, &doc_node)?;
        }
    } else if let Ok(plain_doc) = bound.downcast::<PyPlainDocstring>() {
        walk_plain_cst(py, plain_doc.clone().unbind(), &visitor, &active)?;
    } else {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "expected GoogleDocstring, NumPyDocstring, or PlainDocstring",
        ));
    }
    Ok(visitor)
}

// =============================================================================
// Module
// =============================================================================

#[pymodule]
fn pydocstring(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_google, m)?)?;
    m.add_function(wrap_pyfunction!(parse_numpy, m)?)?;
    m.add_function(wrap_pyfunction!(parse_plain, m)?)?;
    m.add_function(wrap_pyfunction!(detect_style, m)?)?;
    m.add_function(wrap_pyfunction!(py_emit_google, m)?)?;
    m.add_function(wrap_pyfunction!(py_emit_numpy, m)?)?;
    m.add_function(wrap_pyfunction!(walk, m)?)?;
    // Core types
    m.add_class::<PyStyle>()?;
    m.add_class::<PyGoogleSectionKind>()?;
    m.add_class::<PyNumPySectionKind>()?;
    m.add_class::<PyTextRange>()?;
    m.add_class::<PyLineColumn>()?;
    m.add_class::<PyToken>()?;
    m.add_class::<PyWalkContext>()?;
    // Google CST wrappers
    m.add_class::<PyGoogleDocstring>()?;
    m.add_class::<PyGoogleSection>()?;
    m.add_class::<PyGoogleArg>()?;
    m.add_class::<PyGoogleReturn>()?;
    m.add_class::<PyGoogleYield>()?;
    m.add_class::<PyGoogleException>()?;
    m.add_class::<PyGoogleWarning>()?;
    m.add_class::<PyGoogleSeeAlsoItem>()?;
    m.add_class::<PyGoogleAttribute>()?;
    m.add_class::<PyGoogleMethod>()?;
    // NumPy CST wrappers
    m.add_class::<PyNumPyDocstring>()?;
    m.add_class::<PyNumPySection>()?;
    m.add_class::<PyNumPyDeprecation>()?;
    m.add_class::<PyNumPyParameter>()?;
    m.add_class::<PyNumPyReturns>()?;
    m.add_class::<PyNumPyYields>()?;
    m.add_class::<PyNumPyException>()?;
    m.add_class::<PyNumPyWarning>()?;
    m.add_class::<PyNumPySeeAlsoItem>()?;
    m.add_class::<PyNumPyReference>()?;
    m.add_class::<PyNumPyAttribute>()?;
    m.add_class::<PyNumPyMethod>()?;
    // Plain CST wrapper
    m.add_class::<PyPlainDocstring>()?;
    // Model IR
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
    Ok(())
}
