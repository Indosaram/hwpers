"""Tests for Writer: basic creation, paragraphs, headings, images, hyperlinks, text boxes, page settings, save/bytes."""

import os
import tempfile

import pytest
from hwpers import (
    Writer,
    TextStyle,
    StyledText,
    Table,
    Image,
    Hyperlink,
    Header,
    Footer,
    ImageFormat,
    HeaderFooterApplyTo,
    PageNumberFormat,
)


class TestWriterBasic:
    def test_create_writer(self):
        w = Writer()
        assert repr(w) == "Writer()"

    def test_to_bytes_empty(self):
        w = Writer()
        data = w.to_bytes()
        assert isinstance(data, bytes)
        assert len(data) > 0

    def test_add_paragraph(self):
        w = Writer()
        w.add_paragraph("Hello")
        data = w.to_bytes()
        assert len(data) > 0

    def test_add_multiple_paragraphs(self):
        w = Writer()
        w.add_paragraph("First")
        w.add_paragraph("Second")
        w.add_paragraph("Third")
        data = w.to_bytes()
        assert len(data) > 0

    def test_save_to_file(self):
        w = Writer()
        w.add_paragraph("Test content")
        with tempfile.NamedTemporaryFile(suffix=".hwpx", delete=False) as f:
            path = f.name
        try:
            w.save_to_file(path)
            assert os.path.exists(path)
            assert os.path.getsize(path) > 0
        finally:
            os.unlink(path)

    def test_save_to_file_invalid_path(self):
        w = Writer()
        w.add_paragraph("Test")
        with pytest.raises(OSError):
            w.save_to_file("/nonexistent/dir/test.hwpx")


class TestWriterTable:
    def test_add_table(self):
        w = Writer()
        t = Table(2, 3)
        t.set_cell(0, 0, "A1")
        t.set_cell(0, 1, "B1")
        t.set_cell(1, 0, "A2")
        w.add_table(t)
        data = w.to_bytes()
        assert len(data) > 0

    def test_add_table_from_data(self):
        w = Writer()
        t = Table.from_data([["Header", "Value"], ["Name", "Test"]])
        w.add_table(t)
        data = w.to_bytes()
        assert len(data) > 0

    def test_table_repr(self):
        t = Table(3, 4)
        assert "rows=3" in repr(t)
        assert "cols=4" in repr(t)

    def test_table_rows_getter(self):
        t = Table.from_data([["a", "b"], ["c", "d"]])
        rows = t.rows
        assert rows == [["a", "b"], ["c", "d"]]


class TestWriterImage:
    @staticmethod
    def make_png_bytes():
        # Minimal valid PNG header
        return (
            b"\x89PNG\r\n\x1a\n"
            b"\x00\x00\x00\rIHDR"
            b"\x00\x00\x00\x01"
            b"\x00\x00\x00\x01"
            b"\x08\x02\x00\x00\x00\x90wS\xde"
        )

    def test_image_from_bytes(self):
        img = Image.from_bytes(self.make_png_bytes())
        assert img.format == ImageFormat.Png

    def test_image_with_size(self):
        img = Image.from_bytes(self.make_png_bytes()).with_size(100, 50)
        assert img.width_mm == 100
        assert img.height_mm == 50

    def test_image_invalid_bytes(self):
        with pytest.raises(ValueError):
            Image.from_bytes(b"not an image")

    def test_add_image(self):
        w = Writer()
        img = Image.from_bytes(self.make_png_bytes()).with_size(50, 50)
        w.add_image(img)
        data = w.to_bytes()
        assert len(data) > 0


class TestWriterHyperlink:
    def test_add_hyperlink(self):
        w = Writer()
        w.add_hyperlink("Click here", "https://example.com")
        data = w.to_bytes()
        assert len(data) > 0

    def test_add_paragraph_with_hyperlinks(self):
        w = Writer()
        links = [
            Hyperlink("Google", "https://google.com"),
            Hyperlink("GitHub", "https://github.com"),
        ]
        w.add_paragraph_with_hyperlinks("Visit Google and GitHub", links)
        data = w.to_bytes()
        assert len(data) > 0

    def test_hyperlink_properties(self):
        h = Hyperlink("Click", "https://example.com")
        assert h.text == "Click"
        assert h.url == "https://example.com"


class TestWriterHeaderFooter:
    def test_add_header(self):
        w = Writer()
        w.add_paragraph("Content")
        w.add_header("My Header")
        data = w.to_bytes()
        assert len(data) > 0

    def test_add_header_config(self):
        w = Writer()
        w.add_paragraph("Content")
        header = Header("Odd Header")
        w.add_header_config(header)
        data = w.to_bytes()
        assert len(data) > 0

    def test_header_for_odd_pages(self):
        h = Header.for_odd_pages("Odd page header")
        assert h.text == "Odd page header"

    def test_header_for_even_pages(self):
        h = Header.for_even_pages("Even page header")
        assert h.text == "Even page header"

    def test_add_footer(self):
        w = Writer()
        w.add_paragraph("Content")
        w.add_footer("My Footer")
        data = w.to_bytes()
        assert len(data) > 0

    def test_add_footer_with_page_number(self):
        w = Writer()
        w.add_paragraph("Content")
        w.add_footer_with_page_number("Page ")
        data = w.to_bytes()
        assert len(data) > 0

    def test_footer_chaining(self):
        f = Footer("Footer").with_page_number()
        assert f.include_page_number is True
        assert f.text == "Footer"

    def test_footer_page_number_format(self):
        f = Footer("Page ").with_page_number_format(PageNumberFormat.RomanUpper)
        assert f.include_page_number is True

    def test_footer_for_odd_pages(self):
        f = Footer("Odd footer").for_odd_pages()
        assert f.text == "Odd footer"

    def test_footer_config(self):
        w = Writer()
        w.add_paragraph("Content")
        footer = Footer("Custom").with_page_number().for_even_pages()
        w.add_footer_config(footer)
        data = w.to_bytes()
        assert len(data) > 0
