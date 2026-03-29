from ._pydocstring import *  # noqa: F401, F403
from ._visitor import Visitor  # noqa: F401

__all__ = [
    # ── Core types ────────────────────────────────────────────────────────
    "TextRange",
    "LineColumn",
    "WalkContext",
    "Token",
    "Style",
    # ── Section kinds ─────────────────────────────────────────────────────
    "SectionKind",
    "GoogleSectionKind",
    "NumPySectionKind",
    # ── Google CST wrappers ───────────────────────────────────────────────
    "GoogleDocstring",
    "GoogleSection",
    "GoogleArg",
    "GoogleReturn",
    "GoogleYield",
    "GoogleException",
    "GoogleWarning",
    "GoogleSeeAlsoItem",
    "GoogleAttribute",
    "GoogleMethod",
    # ── NumPy CST wrappers ────────────────────────────────────────────────
    "NumPyDocstring",
    "NumPySection",
    "NumPyDeprecation",
    "NumPyParameter",
    "NumPyReturns",
    "NumPyYields",
    "NumPyException",
    "NumPyWarning",
    "NumPySeeAlsoItem",
    "NumPyReference",
    "NumPyAttribute",
    "NumPyMethod",
    # ── Plain CST wrapper ─────────────────────────────────────────────────
    "PlainDocstring",
    # ── Model IR ──────────────────────────────────────────────────────────
    "Docstring",
    "Section",
    "Parameter",
    "Return",
    "ExceptionEntry",
    "SeeAlsoEntry",
    "Reference",
    "Attribute",
    "Method",
    "Deprecation",
    # ── Visitor ───────────────────────────────────────────────────────────
    "Visitor",
    # ── Functions ─────────────────────────────────────────────────────────
    "parse",
    "parse_google",
    "parse_numpy",
    "parse_plain",
    "detect_style",
    "emit_google",
    "emit_numpy",
    "walk",
]
