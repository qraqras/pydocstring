import pydocstring


class TestDetectStyle:
    def test_google(self):
        assert pydocstring.detect_style("Summary.\n\nArgs:\n    x: Desc.") == pydocstring.Style.GOOGLE

    def test_numpy(self):
        assert (
            pydocstring.detect_style(
                "Summary.\n\nParameters\n----------\nx : int\n    Desc."
            )
            == pydocstring.Style.NUMPY
        )

    def test_fallback_to_plain(self):
        assert pydocstring.detect_style("Just a summary.") == pydocstring.Style.PLAIN

    def test_str(self):
        assert str(pydocstring.Style.GOOGLE) == "google"
        assert str(pydocstring.Style.NUMPY) == "numpy"
        assert str(pydocstring.Style.PLAIN) == "plain"

    def test_repr(self):
        assert repr(pydocstring.Style.GOOGLE) == "Style.GOOGLE"
        assert repr(pydocstring.Style.NUMPY) == "Style.NUMPY"
        assert repr(pydocstring.Style.PLAIN) == "Style.PLAIN"


class TestParseGoogle:
    def test_summary(self):
        doc = pydocstring.parse_google("Summary line.")
        assert doc.summary is not None
        assert doc.summary.text == "Summary line."

    def test_args(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nArgs:\n    x (int): The value.\n    y (str): Another."
        )
        sections = doc.sections
        assert len(sections) == 1
        assert sections[0].kind == "Args"
        args = sections[0].args
        assert len(args) == 2
        assert args[0].name.text == "x"
        assert args[0].type.text == "int"
        assert args[0].description.text == "The value."
        assert args[1].name.text == "y"
        assert args[1].type.text == "str"

    def test_returns(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nReturns:\n    bool: True if successful."
        )
        section = doc.sections[0]
        assert section.kind == "Returns"
        assert section.returns is not None
        assert section.returns.return_type.text == "bool"
        assert section.returns.description.text == "True if successful."

    def test_raises(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nRaises:\n    ValueError: If x is negative."
        )
        section = doc.sections[0]
        assert section.kind == "Raises"
        assert len(section.exceptions) == 1
        assert section.exceptions[0].type.text == "ValueError"
        assert section.exceptions[0].description.text == "If x is negative."

    def test_extended_summary(self):
        doc = pydocstring.parse_google("Summary.\n\nExtended description here.")
        assert doc.extended_summary is not None
        assert doc.extended_summary.text == "Extended description here."

    def test_body_text_section(self):
        doc = pydocstring.parse_google("Summary.\n\nNotes:\n    Some free text.")
        section = doc.sections[0]
        assert section.kind == "Notes"
        assert section.body_text is not None

    def test_pretty_print(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.")
        output = doc.pretty_print()
        assert "GOOGLE_DOCSTRING" in output
        assert "SUMMARY" in output

    def test_source(self):
        text = "Summary.\n\nArgs:\n    x: Desc."
        doc = pydocstring.parse_google(text)
        assert doc.source == text

    def test_no_summary(self):
        doc = pydocstring.parse_google("")
        assert doc.summary is None


class TestParseNumPy:
    def test_summary(self):
        doc = pydocstring.parse_numpy("Summary line.")
        assert doc.summary is not None
        assert doc.summary.text == "Summary line."

    def test_parameters(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nParameters\n----------\nx : int\n    The first.\ny : str\n    The second."
        )
        sections = doc.sections
        assert len(sections) == 1
        assert sections[0].kind == "Parameters"
        params = sections[0].parameters
        assert len(params) == 2
        assert [n.text for n in params[0].names] == ["x"]
        assert params[0].type.text == "int"
        assert params[0].description.text == "The first."
        assert [n.text for n in params[1].names] == ["y"]

    def test_returns(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nReturns\n-------\nbool\n    True if successful."
        )
        section = doc.sections[0]
        assert section.kind == "Returns"
        assert len(section.returns) == 1
        assert section.returns[0].return_type.text == "bool"
        assert section.returns[0].description.text == "True if successful."

    def test_raises(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nRaises\n------\nValueError\n    If x is negative."
        )
        section = doc.sections[0]
        assert section.kind == "Raises"
        assert len(section.exceptions) == 1
        assert section.exceptions[0].type.text == "ValueError"

    def test_pretty_print(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nParameters\n----------\nx : int\n    Desc."
        )
        output = doc.pretty_print()
        assert "NUMPY_DOCSTRING" in output

    def test_source(self):
        text = "Summary.\n\nParameters\n----------\nx : int\n    Desc."
        doc = pydocstring.parse_numpy(text)
        assert doc.source == text


class TestToken:
    def test_properties(self):
        doc = pydocstring.parse_google("Summary.")
        token = doc.summary
        assert token.kind == pydocstring.SyntaxKind.SUMMARY
        assert token.text == "Summary."
        assert token.range.start == 0
        assert token.range.end == 8


class TestNode:
    def test_properties(self):
        doc = pydocstring.parse_google("Summary.")
        node = doc.node
        assert node.kind == pydocstring.SyntaxKind.GOOGLE_DOCSTRING
        assert len(node.children) > 0
        assert node.range.start == 0


class TestWalk:
    def test_walk_yields_all(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x (int): Value.")
        items = list(pydocstring.walk(doc.node))
        kinds = [item.kind for item in items]
        assert pydocstring.SyntaxKind.GOOGLE_DOCSTRING in kinds
        assert pydocstring.SyntaxKind.SUMMARY in kinds
        assert pydocstring.SyntaxKind.NAME in kinds

    def test_walk_collects_names(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.\n    y: Desc.")
        names = [
            item.text
            for item in pydocstring.walk(doc.node)
            if isinstance(item, pydocstring.Token) and item.kind == pydocstring.SyntaxKind.NAME
        ]
        assert names == ["Args", "x", "y"]


class TestModelTypes:
    def test_parameter_construction(self):
        p = pydocstring.Parameter(["x"], type_annotation="int", description="The value.")
        assert p.names == ["x"]
        assert p.type_annotation == "int"
        assert p.description == "The value."
        assert p.is_optional is False
        assert p.default_value is None

    def test_parameter_mutability(self):
        p = pydocstring.Parameter(["x"])
        p.names = ["x", "y"]
        p.type_annotation = "str"
        p.is_optional = True
        assert p.names == ["x", "y"]
        assert p.type_annotation == "str"
        assert p.is_optional is True

    def test_return_construction(self):
        r = pydocstring.Return(type_annotation="int", description="The result.")
        assert r.name is None
        assert r.type_annotation == "int"
        assert r.description == "The result."

    def test_exception_entry_construction(self):
        e = pydocstring.ExceptionEntry("ValueError", description="If x is negative.")
        assert e.type_name == "ValueError"
        assert e.description == "If x is negative."

    def test_deprecation_construction(self):
        d = pydocstring.Deprecation("1.6.0", description="Use new_func instead.")
        assert d.version == "1.6.0"
        assert d.description == "Use new_func instead."

    def test_attribute_construction(self):
        a = pydocstring.Attribute("name", type_annotation="str", description="The name.")
        assert a.name == "name"
        assert a.type_annotation == "str"

    def test_method_construction(self):
        m = pydocstring.Method("run", description="Run the task.")
        assert m.name == "run"
        assert m.type_annotation is None
        assert m.description == "Run the task."

    def test_see_also_entry_construction(self):
        s = pydocstring.SeeAlsoEntry(["foo", "bar"], description="Related functions.")
        assert s.names == ["foo", "bar"]
        assert s.description == "Related functions."

    def test_reference_construction(self):
        r = pydocstring.Reference(number="1", content="Doe et al. 2020")
        assert r.number == "1"
        assert r.content == "Doe et al. 2020"


class TestSection:
    def test_parameters_section(self):
        p = pydocstring.Parameter(["x"], type_annotation="int", description="Value.")
        sec = pydocstring.Section("parameters", parameters=[p])
        assert sec.kind == "parameters"
        params = sec.parameters
        assert len(params) == 1
        assert params[0].names == ["x"]
        assert params[0].type_annotation == "int"

    def test_returns_section(self):
        r = pydocstring.Return(type_annotation="bool", description="Success.")
        sec = pydocstring.Section("returns", returns=[r])
        assert sec.kind == "returns"
        rets = sec.returns
        assert len(rets) == 1
        assert rets[0].type_annotation == "bool"

    def test_raises_section(self):
        e = pydocstring.ExceptionEntry("ValueError", description="Bad value.")
        sec = pydocstring.Section("raises", exceptions=[e])
        assert sec.kind == "raises"
        assert len(sec.exceptions) == 1
        assert sec.exceptions[0].type_name == "ValueError"

    def test_free_text_section(self):
        sec = pydocstring.Section("notes", body="Some notes here.")
        assert sec.kind == "notes"
        assert sec.body == "Some notes here."

    def test_empty_accessors(self):
        sec = pydocstring.Section("parameters", parameters=[])
        assert sec.returns == []
        assert sec.exceptions == []
        assert sec.body is None


class TestDocstringModel:
    def test_construction(self):
        doc = pydocstring.Docstring(summary="Brief summary.")
        assert doc.summary == "Brief summary."
        assert doc.extended_summary is None
        assert doc.deprecation is None
        assert doc.sections == []

    def test_mutability(self):
        doc = pydocstring.Docstring(summary="Old.")
        doc.summary = "New."
        assert doc.summary == "New."

    def test_with_sections(self):
        p = pydocstring.Parameter(["x"], type_annotation="int")
        sec = pydocstring.Section("parameters", parameters=[p])
        doc = pydocstring.Docstring(summary="Brief.", sections=[sec])
        assert len(doc.sections) == 1
        assert doc.sections[0].kind == "parameters"

    def test_with_deprecation(self):
        dep = pydocstring.Deprecation("2.0", description="Removed.")
        doc = pydocstring.Docstring(deprecation=dep)
        assert doc.deprecation is not None
        assert doc.deprecation.version == "2.0"


class TestToModel:
    def test_google_to_model(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nArgs:\n    x (int): The value.\n"
        )
        model = doc.to_model()
        assert model.summary == "Summary."
        assert len(model.sections) == 1
        assert model.sections[0].kind == "parameters"
        params = model.sections[0].parameters
        assert len(params) == 1
        assert params[0].names == ["x"]
        assert params[0].type_annotation == "int"
        assert params[0].description == "The value."

    def test_numpy_to_model(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nParameters\n----------\nx : int\n    The value.\n"
        )
        model = doc.to_model()
        assert model.summary == "Summary."
        assert len(model.sections) == 1
        assert model.sections[0].kind == "parameters"
        params = model.sections[0].parameters
        assert len(params) == 1
        assert params[0].names == ["x"]
        assert params[0].type_annotation == "int"

    def test_google_to_model_raises(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nRaises:\n    ValueError: Bad input.\n"
        )
        model = doc.to_model()
        assert model.sections[0].kind == "raises"
        assert model.sections[0].exceptions[0].type_name == "ValueError"

    def test_google_to_model_returns(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nReturns:\n    int: The result.\n"
        )
        model = doc.to_model()
        assert model.sections[0].kind == "returns"
        rets = model.sections[0].returns
        assert len(rets) == 1
        assert rets[0].type_annotation == "int"

    def test_plain_to_model(self):
        doc = pydocstring.parse_plain("Brief summary.\n\nMore details.")
        model = doc.to_model()
        assert model.summary == "Brief summary."
        assert model.extended_summary == "More details."
        assert model.sections == []

    def test_plain_to_model_summary_only(self):
        doc = pydocstring.parse_plain("Just a summary.")
        model = doc.to_model()
        assert model.summary == "Just a summary."
        assert model.extended_summary is None


class TestParsePlain:
    def test_summary(self):
        doc = pydocstring.parse_plain("Summary line.")
        assert doc.summary is not None
        assert doc.summary.text == "Summary line."
        assert doc.extended_summary is None

    def test_empty(self):
        doc = pydocstring.parse_plain("")
        assert doc.summary is None
        assert doc.extended_summary is None

    def test_extended_summary(self):
        doc = pydocstring.parse_plain("Summary.\n\nMore details here.\nContinued.")
        assert doc.summary is not None
        assert doc.summary.text == "Summary."
        assert doc.extended_summary is not None
        assert "More details here." in doc.extended_summary.text

    def test_no_sections(self):
        # Plain docstrings never produce sections — Sphinx-like text stays plain
        doc = pydocstring.parse_plain(
            "Summary.\n\n:param x: A value.\n:returns: Something."
        )
        model = doc.to_model()
        assert model.sections == []

    def test_node_kind(self):
        doc = pydocstring.parse_plain("Summary.")
        assert doc.node.kind == pydocstring.SyntaxKind.PLAIN_DOCSTRING

    def test_source(self):
        text = "Summary.\n\nExtended."
        doc = pydocstring.parse_plain(text)
        assert doc.source == text

    def test_pretty_print(self):
        doc = pydocstring.parse_plain("Summary.\n\nExtended.")
        output = doc.pretty_print()
        assert "PLAIN_DOCSTRING" in output
        assert "SUMMARY" in output
        assert "EXTENDED_SUMMARY" in output

    def test_summary_token_kind(self):
        doc = pydocstring.parse_plain("Summary.")
        assert doc.summary.kind == pydocstring.SyntaxKind.SUMMARY

    def test_extended_summary_token_kind(self):
        doc = pydocstring.parse_plain("Summary.\n\nExtended.")
        assert doc.extended_summary.kind == pydocstring.SyntaxKind.EXTENDED_SUMMARY

    def test_repr(self):
        doc = pydocstring.parse_plain("Summary.")
        assert repr(doc) == "PlainDocstring(...)"

    def test_line_col_summary(self):
        doc = pydocstring.parse_plain("Summary.")
        lc = doc.line_col(doc.summary.range.start)
        assert lc.lineno == 1
        assert lc.col == 0

    def test_line_col_extended_summary(self):
        doc = pydocstring.parse_plain("Summary.\n\nExtended.")
        lc = doc.line_col(doc.extended_summary.range.start)
        assert lc.lineno == 3
        assert lc.col == 0

    def test_detect_style_dispatches_to_plain(self):
        assert pydocstring.detect_style("Just a summary.") == pydocstring.Style.PLAIN
        assert (
            pydocstring.detect_style("Summary.\n\n:param x: value.")
            == pydocstring.Style.PLAIN
        )

    def test_style_property(self):
        doc = pydocstring.parse_plain("Summary.")
        assert doc.style == pydocstring.Style.PLAIN


class TestParse:
    """Tests for the unified parse() entry point."""

    def test_google_returns_google_docstring(self):
        doc = pydocstring.parse("Summary.\n\nArgs:\n    x (int): Value.")
        assert isinstance(doc, pydocstring.GoogleDocstring)
        assert doc.style == pydocstring.Style.GOOGLE

    def test_numpy_returns_numpy_docstring(self):
        doc = pydocstring.parse(
            "Summary.\n\nParameters\n----------\nx : int\n    Value."
        )
        assert isinstance(doc, pydocstring.NumPyDocstring)
        assert doc.style == pydocstring.Style.NUMPY

    def test_plain_returns_plain_docstring(self):
        doc = pydocstring.parse("Just a summary.")
        assert isinstance(doc, pydocstring.PlainDocstring)
        assert doc.style == pydocstring.Style.PLAIN

    def test_empty_returns_plain_docstring(self):
        doc = pydocstring.parse("")
        assert isinstance(doc, pydocstring.PlainDocstring)
        assert doc.style == pydocstring.Style.PLAIN

    def test_sphinx_returns_plain_docstring(self):
        doc = pydocstring.parse("Summary.\n\n:param x: A value.\n:returns: Something.")
        assert isinstance(doc, pydocstring.PlainDocstring)
        assert doc.style == pydocstring.Style.PLAIN

    def test_google_style_property(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.")
        assert doc.style == pydocstring.Style.GOOGLE

    def test_numpy_style_property(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nParameters\n----------\nx : int\n    Desc."
        )
        assert doc.style == pydocstring.Style.NUMPY

    def test_parse_google_summary(self):
        doc = pydocstring.parse("Summary.\n\nArgs:\n    x (int): Value.")
        assert doc.summary.text == "Summary."

    def test_parse_numpy_summary(self):
        doc = pydocstring.parse(
            "Summary.\n\nParameters\n----------\nx : int\n    Value."
        )
        assert doc.summary.text == "Summary."

    def test_parse_plain_summary(self):
        doc = pydocstring.parse("Plain summary.")
        assert doc.summary.text == "Plain summary."

    def test_parse_to_model_google(self):
        doc = pydocstring.parse("Summary.\n\nArgs:\n    x (int): Value.")
        model = doc.to_model()
        assert model.summary == "Summary."
        assert model.sections[0].kind == "parameters"

    def test_parse_to_model_numpy(self):
        doc = pydocstring.parse(
            "Summary.\n\nParameters\n----------\nx : int\n    Value."
        )
        model = doc.to_model()
        assert model.summary == "Summary."
        assert model.sections[0].kind == "parameters"

    def test_parse_to_model_plain(self):
        doc = pydocstring.parse("Summary.\n\nExtended.")
        model = doc.to_model()
        assert model.summary == "Summary."
        assert model.sections == []

    def test_match_style(self):
        # Verify match-statement style dispatch works
        for src, expected_style in [
            ("Summary.\n\nArgs:\n    x: Desc.", pydocstring.Style.GOOGLE),
            ("Summary.\n\nParameters\n----------\nx : int\n    Desc.", pydocstring.Style.NUMPY),
            ("Just a summary.", pydocstring.Style.PLAIN),
        ]:
            doc = pydocstring.parse(src)
            assert doc.style == expected_style


class TestEmit:
    def test_emit_google(self):
        doc = pydocstring.Docstring(
            summary="Brief summary.",
            sections=[
                pydocstring.Section(
                    "parameters",
                    parameters=[
                        pydocstring.Parameter(
                            ["x"], type_annotation="int", description="The value."
                        )
                    ],
                )
            ],
        )
        text = pydocstring.emit_google(doc)
        assert "Brief summary." in text
        assert "Args:" in text
        assert "x (int):" in text

    def test_emit_numpy(self):
        doc = pydocstring.Docstring(
            summary="Brief summary.",
            sections=[
                pydocstring.Section(
                    "parameters",
                    parameters=[
                        pydocstring.Parameter(
                            ["x"], type_annotation="int", description="The value."
                        )
                    ],
                )
            ],
        )
        text = pydocstring.emit_numpy(doc)
        assert "Brief summary." in text
        assert "Parameters" in text
        assert "----------" in text
        assert "x : int" in text

    def test_roundtrip_google(self):
        original = "Summary.\n\nArgs:\n    x (int): The value.\n"
        doc = pydocstring.parse_google(original)
        model = doc.to_model()
        emitted = pydocstring.emit_google(model)
        assert "Summary." in emitted
        assert "Args:" in emitted
        assert "x (int):" in emitted

    def test_roundtrip_numpy(self):
        original = "Summary.\n\nParameters\n----------\nx : int\n    The value.\n"
        doc = pydocstring.parse_numpy(original)
        model = doc.to_model()
        emitted = pydocstring.emit_numpy(model)
        assert "Summary." in emitted
        assert "Parameters" in emitted
        assert "x : int" in emitted

    def test_convert_google_to_numpy(self):
        google_doc = pydocstring.parse_google(
            "Summary.\n\nArgs:\n    x (int): The value.\n"
        )
        model = google_doc.to_model()
        numpy_text = pydocstring.emit_numpy(model)
        assert "Summary." in numpy_text
        assert "Parameters" in numpy_text
        assert "----------" in numpy_text
        assert "x : int" in numpy_text


class TestLineCol:
    """Tests for GoogleDocstring.line_col() and NumPyDocstring.line_col()."""

    # ── Google ───────────────────────────────────────────────────────────────

    def test_google_summary_first_line(self):
        doc = pydocstring.parse_google("Summary.")
        lc = doc.line_col(doc.summary.range.start)
        assert lc.lineno == 1
        assert lc.col == 0

    def test_google_arg_name_lineno(self):
        src = "Summary.\n\nArgs:\n    x (int): Value."
        doc = pydocstring.parse_google(src)
        arg = doc.sections[0].args[0]
        lc = doc.line_col(arg.name.range.start)
        assert lc.lineno == 4   # "    x (int): Value." is on line 4
        assert lc.col == 4      # 4 spaces of indentation

    def test_google_col_is_codepoints_not_bytes(self):
        # "α" is 2 bytes in UTF-8 but 1 codepoint.
        # Source: "α.\n\nArgs:\n    x: V."
        # "x" starts at byte 4+4=... let's compute:
        # line 1: "α.\n"  → α=2bytes, .=1, \n=1  → line_start line4 = 2+1+1+1 = 5
        # line 2: "\n"    → 1byte
        # line 3: "Args:\n" → 6bytes
        # line 4: "    x: V.\n" → "    x" starts with 4 spaces + x
        # byte of "x" in line4 = 5+1+6+4 = 16
        src = "α.\n\nArgs:\n    x: V."
        doc = pydocstring.parse_google(src)
        arg = doc.sections[0].args[0]
        lc = doc.line_col(arg.name.range.start)
        assert lc.lineno == 4
        assert lc.col == 4  # 4 spaces → 4 codepoints (bytes == codepoints here)

    def test_google_multibyte_col(self):
        # Line with multibyte chars before the token.
        # "αβ: int" as the summary — check col of "int" token text
        # α=2bytes, β=2bytes, :=1, space=1 → "int" starts at byte 6
        # but codepoints: α=1, β=1, :=1, space=1 → col=4
        src = "αβ: int"
        doc = pydocstring.parse_google(src)
        # The whole line is treated as summary; check that line_col at byte 6
        # returns col 4 (codepoints), not 6 (bytes)
        lc = doc.line_col(6)
        assert lc.lineno == 1
        assert lc.col == 4

    def test_google_multiline_lineno(self):
        src = "Summary.\n\nExtended.\n\nArgs:\n    x: V."
        doc = pydocstring.parse_google(src)
        arg = doc.sections[0].args[0]
        lc = doc.line_col(arg.name.range.start)
        assert lc.lineno == 6

    def test_google_returns_class(self):
        lc = pydocstring.parse_google("S.").line_col(0)
        assert isinstance(lc, pydocstring.LineColumn)

    def test_google_out_of_bounds(self):
        import pytest
        doc = pydocstring.parse_google("S.")
        with pytest.raises(Exception):
            doc.line_col(9999)

    # ── NumPy ────────────────────────────────────────────────────────────────

    def test_numpy_summary_first_line(self):
        doc = pydocstring.parse_numpy("Summary.")
        lc = doc.line_col(doc.summary.range.start)
        assert lc.lineno == 1
        assert lc.col == 0

    def test_numpy_param_name_lineno(self):
        src = "Summary.\n\nParameters\n----------\nx : int\n    Desc."
        doc = pydocstring.parse_numpy(src)
        param = doc.sections[0].parameters[0]
        lc = doc.line_col(param.names[0].range.start)
        assert lc.lineno == 5   # "x : int" is on line 5
        assert lc.col == 0

    def test_numpy_multibyte_col(self):
        # Same multibyte check for NumPy path
        src = "αβ: int"
        doc = pydocstring.parse_numpy(src)
        lc = doc.line_col(6)
        assert lc.lineno == 1
        assert lc.col == 4

    def test_numpy_returns_class(self):
        lc = pydocstring.parse_numpy("S.").line_col(0)
        assert isinstance(lc, pydocstring.LineColumn)

    def test_numpy_out_of_bounds(self):
        import pytest
        doc = pydocstring.parse_numpy("S.")
        with pytest.raises(Exception):
            doc.line_col(9999)

    def test_emit_free_text_section(self):
        doc = pydocstring.Docstring(
            summary="Brief.",
            sections=[pydocstring.Section("notes", body="Some notes.")],
        )
        text = pydocstring.emit_google(doc)
        assert "Notes:" in text
        assert "Some notes." in text
