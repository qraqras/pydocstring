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
        assert sections[0].section_kind == pydocstring.GoogleSectionKind.ARGS
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
        assert section.section_kind == pydocstring.GoogleSectionKind.RETURNS
        ret = section.returns
        assert ret is not None
        assert ret.return_type.text == "bool"
        assert ret.description.text == "True if successful."

    def test_raises(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nRaises:\n    ValueError: If x is negative."
        )
        section = doc.sections[0]
        assert section.section_kind == pydocstring.GoogleSectionKind.RAISES
        excepts = section.exceptions
        assert len(excepts) == 1
        assert excepts[0].type.text == "ValueError"
        assert excepts[0].description.text == "If x is negative."

    def test_extended_summary(self):
        doc = pydocstring.parse_google("Summary.\n\nExtended description here.")
        assert doc.extended_summary is not None
        assert doc.extended_summary.text == "Extended description here."

    def test_body_text_section(self):
        doc = pydocstring.parse_google("Summary.\n\nNotes:\n    Some free text.")
        section = doc.sections[0]
        assert section.section_kind == pydocstring.GoogleSectionKind.NOTES
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

    def test_yields_is_optional(self):
        doc = pydocstring.parse_google(
            "Summary.\n\nYields:\n    int: The next value."
        )
        section = doc.sections[0]
        assert section.section_kind == pydocstring.GoogleSectionKind.YIELDS
        yld = section.yields
        assert yld is not None
        assert yld.return_type.text == "int"

    def test_section_kind_repr(self):
        assert repr(pydocstring.GoogleSectionKind.ARGS) == "GoogleSectionKind.ARGS"
        assert repr(pydocstring.GoogleSectionKind.RETURNS) == "GoogleSectionKind.RETURNS"

    def test_range_on_token(self):
        doc = pydocstring.parse_google("Summary.")
        r = doc.summary.range
        assert r.start == 0
        assert r.end == 8


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
        assert sections[0].section_kind == pydocstring.NumPySectionKind.PARAMETERS
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
        assert section.section_kind == pydocstring.NumPySectionKind.RETURNS
        returns = section.returns
        assert len(returns) == 1
        assert returns[0].return_type.text == "bool"
        assert returns[0].description.text == "True if successful."

    def test_raises(self):
        doc = pydocstring.parse_numpy(
            "Summary.\n\nRaises\n------\nValueError\n    If x is negative."
        )
        section = doc.sections[0]
        assert section.section_kind == pydocstring.NumPySectionKind.RAISES
        excepts = section.exceptions
        assert len(excepts) == 1
        assert excepts[0].type.text == "ValueError"

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

    def test_section_kind_repr(self):
        assert repr(pydocstring.NumPySectionKind.PARAMETERS) == "NumPySectionKind.PARAMETERS"
        assert repr(pydocstring.NumPySectionKind.RETURNS) == "NumPySectionKind.RETURNS"


class TestToken:
    def test_text_and_range(self):
        doc = pydocstring.parse_google("Summary.")
        token = doc.summary
        assert token.text == "Summary."
        assert token.range.start == 0
        assert token.range.end == 8

    def test_repr(self):
        doc = pydocstring.parse_google("Hello.")
        assert repr(doc.summary) == 'Token("Hello.")'

    def test_no_kind_field(self):
        doc = pydocstring.parse_google("Summary.")
        assert not hasattr(doc.summary, "kind"), "Token must not expose a 'kind' field"


class TestTextRange:
    def test_range_repr(self):
        doc = pydocstring.parse_google("Summary.")
        r = doc.summary.range
        assert repr(r) == "TextRange(0..8)"

    def test_section_range(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.")
        section = doc.sections[0]
        r = section.range
        assert r.start < r.end


class TestLineColumn:
    def test_summary_start(self):
        doc = pydocstring.parse_plain("Summary.")
        lc = doc.line_col(doc.summary.range.start)
        assert lc.lineno == 1
        assert lc.col == 0

    def test_extended_summary_start(self):
        doc = pydocstring.parse_plain("Summary.\n\nExtended.")
        lc = doc.line_col(doc.extended_summary.range.start)
        assert lc.lineno == 3
        assert lc.col == 0

    def test_repr(self):
        doc = pydocstring.parse_plain("Summary.")
        lc = doc.line_col(0)
        assert repr(lc) == "LineColumn(lineno=1, col=0)"


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
        doc = pydocstring.parse_plain(
            "Summary.\n\n:param x: A value.\n:returns: Something."
        )
        model = doc.to_model()
        assert model.sections == []

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

    def test_no_node_attribute(self):
        doc = pydocstring.parse_plain("Summary.")
        assert not hasattr(doc, "node"), "Docstring must not expose a 'node' attribute"


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


class TestWalk:
    def test_google_walk_collects_args(self):
        source = "Summary.\n\nArgs:\n    x (int): The x value.\n    y (str): The y value."
        doc = pydocstring.parse_google(source)

        class Collector:
            def __init__(self):
                self.arg_names = []

            def visit_google_arg(self, arg):
                self.arg_names.append(arg.name.text)

        collector = Collector()
        pydocstring.walk(doc, collector)
        assert collector.arg_names == ["x", "y"]

    def test_numpy_walk_collects_parameters(self):
        source = "Summary.\n\nParameters\n----------\nx : int\n    Desc x.\ny : str\n    Desc y."
        doc = pydocstring.parse_numpy(source)

        class Collector:
            def __init__(self):
                self.names = []

            def visit_numpy_parameter(self, param):
                self.names.append(param.names[0].text)

        collector = Collector()
        pydocstring.walk(doc, collector)
        assert collector.names == ["x", "y"]

    def test_walk_plain_is_noop(self):
        doc = pydocstring.parse_plain("Just a summary.")

        class Collector:
            def __init__(self):
                self.called = False

            def visit_google_arg(self, arg):
                self.called = True

            def visit_numpy_parameter(self, param):
                self.called = True

        collector = Collector()
        pydocstring.walk(doc, collector)
        assert not collector.called

    def test_walk_rejects_wrong_type(self):
        import pytest
        with pytest.raises(TypeError):
            pydocstring.walk("not a docstring", object())

    def test_walk_via_parse_google(self):
        """walk() dispatches correctly when doc comes from auto-detect parse()."""
        source = "Summary.\n\nArgs:\n    z (float): A float."
        doc = pydocstring.parse(source)
        assert isinstance(doc, pydocstring.GoogleDocstring)

        names = []
        class V:
            def visit_google_arg(self, arg):
                names.append(arg.name.text)

        pydocstring.walk(doc, V())
        assert names == ["z"]

    def test_walk_via_parse_numpy(self):
        """walk() dispatches correctly when doc comes from auto-detect parse()."""
        source = "Summary.\n\nParameters\n----------\na : int\n    Desc."
        doc = pydocstring.parse(source)
        assert isinstance(doc, pydocstring.NumPyDocstring)

        names = []
        class V:
            def visit_numpy_parameter(self, param):
                names.append(param.names[0].text)

        pydocstring.walk(doc, V())
        assert names == ["a"]

    def test_walk_visitor_without_methods_is_safe(self):
        """A visitor with no visit_* methods should not raise."""
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.")
        pydocstring.walk(doc, object())
