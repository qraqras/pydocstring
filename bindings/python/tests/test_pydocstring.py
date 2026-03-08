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

    def test_fallback_to_google(self):
        assert pydocstring.detect_style("Just a summary.") == pydocstring.Style.GOOGLE

    def test_str(self):
        assert str(pydocstring.Style.GOOGLE) == "google"
        assert str(pydocstring.Style.NUMPY) == "numpy"

    def test_repr(self):
        assert repr(pydocstring.Style.GOOGLE) == "Style.GOOGLE"
        assert repr(pydocstring.Style.NUMPY) == "Style.NUMPY"


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
        assert token.kind == "SUMMARY"
        assert token.text == "Summary."
        assert token.range.start == 0
        assert token.range.end == 8


class TestNode:
    def test_properties(self):
        doc = pydocstring.parse_google("Summary.")
        node = doc.node
        assert node.kind == "GOOGLE_DOCSTRING"
        assert len(node.children) > 0
        assert node.range.start == 0


class TestWalk:
    def test_walk_yields_all(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x (int): Value.")
        items = list(pydocstring.walk(doc.node))
        kinds = [item.kind for item in items]
        assert "GOOGLE_DOCSTRING" in kinds
        assert "SUMMARY" in kinds
        assert "NAME" in kinds

    def test_walk_collects_names(self):
        doc = pydocstring.parse_google("Summary.\n\nArgs:\n    x: Desc.\n    y: Desc.")
        names = [
            item.text
            for item in pydocstring.walk(doc.node)
            if isinstance(item, pydocstring.Token) and item.kind == "NAME"
        ]
        assert names == ["Args", "x", "y"]
