"""Tests for TextStyle, StyledText, mixed paragraph styling."""

from hwpers import TextStyle, StyledText, Writer


class TestTextStyle:
    def test_default(self):
        s = TextStyle()
        assert "bold=False" in repr(s)
        assert "italic=False" in repr(s)

    def test_kwargs_bold(self):
        s = TextStyle(bold=True)
        assert "bold=True" in repr(s)

    def test_kwargs_all(self):
        s = TextStyle(
            font_size=14,
            bold=True,
            italic=True,
            underline=True,
            strikethrough=True,
            color=0xFF0000,
        )
        r = repr(s)
        assert "font_size=14" in r
        assert "bold=True" in r
        assert "italic=True" in r
        assert "underline=True" in r
        assert "strikethrough=True" in r
        assert "color=0xFF0000" in r

    def test_chaining_bold(self):
        s = TextStyle().bold()
        assert "bold=True" in repr(s)

    def test_chaining_italic(self):
        s = TextStyle().italic()
        assert "italic=True" in repr(s)

    def test_chaining_underline(self):
        s = TextStyle().underline()
        assert "underline=True" in repr(s)

    def test_chaining_strikethrough(self):
        s = TextStyle().strikethrough()
        assert "strikethrough=True" in repr(s)

    def test_chaining_size(self):
        s = TextStyle().size(20)
        assert "font_size=20" in repr(s)

    def test_chaining_color(self):
        s = TextStyle().color(0x00FF00)
        assert "color=0x00FF00" in repr(s)

    def test_chaining_immutable(self):
        """Verify chaining returns a new object, not mutating self."""
        s1 = TextStyle()
        s2 = s1.bold()
        assert "bold=False" in repr(s1)
        assert "bold=True" in repr(s2)

    def test_chaining_combined(self):
        s = TextStyle().bold().italic().size(16).color(0x0000FF)
        r = repr(s)
        assert "bold=True" in r
        assert "italic=True" in r
        assert "font_size=16" in r
        assert "color=0x0000FF" in r


class TestStyledText:
    def test_default_style(self):
        st = StyledText("hello")
        assert st.text == "hello"

    def test_with_style(self):
        style = TextStyle(bold=True, font_size=18)
        st = StyledText("bold text", style=style)
        assert st.text == "bold text"

    def test_repr(self):
        st = StyledText("test")
        assert 'StyledText("test")' == repr(st)


class TestStyledParagraph:
    def test_styled_paragraph(self):
        w = Writer()
        w.add_styled_paragraph("Bold text", TextStyle(bold=True))
        data = w.to_bytes()
        assert len(data) > 0

    def test_mixed_styled_paragraph(self):
        w = Writer()
        runs = [
            StyledText("Normal "),
            StyledText("bold", style=TextStyle(bold=True)),
            StyledText(" and "),
            StyledText("italic", style=TextStyle(italic=True)),
        ]
        w.add_mixed_styled_paragraph(runs)
        data = w.to_bytes()
        assert len(data) > 0

    def test_multiple_styled_paragraphs(self):
        w = Writer()
        w.add_styled_paragraph("Red text", TextStyle(color=0xFF0000))
        w.add_styled_paragraph("Large text", TextStyle(font_size=24))
        w.add_styled_paragraph(
            "Combined",
            TextStyle(bold=True, italic=True, underline=True, font_size=16, color=0x0000FF),
        )
        data = w.to_bytes()
        assert len(data) > 0
