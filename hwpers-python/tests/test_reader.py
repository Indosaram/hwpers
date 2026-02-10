"""Tests for Reader roundtrip: Writer -> bytes -> Reader -> extract_text."""

from hwpers import Writer, Reader, TextStyle, StyledText, Table


class TestReaderRoundtrip:
    def test_basic_roundtrip(self):
        w = Writer()
        w.add_paragraph("Hello, World!")
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        assert "Hello, World!" in text

    def test_multiple_paragraphs_roundtrip(self):
        w = Writer()
        w.add_paragraph("First paragraph")
        w.add_paragraph("Second paragraph")
        w.add_paragraph("Third paragraph")
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        assert "First paragraph" in text
        assert "Second paragraph" in text
        assert "Third paragraph" in text

    def test_styled_roundtrip(self):
        w = Writer()
        w.add_styled_paragraph("Bold text", TextStyle(bold=True))
        w.add_paragraph("Normal text")
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        assert "Bold text" in text
        assert "Normal text" in text

    def test_mixed_styled_roundtrip(self):
        w = Writer()
        runs = [
            StyledText("Hello "),
            StyledText("world", style=TextStyle(bold=True)),
        ]
        w.add_mixed_styled_paragraph(runs)
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        # Reader merges runs; trailing space in "Hello " may or may not be preserved
        assert "Hello" in text
        assert "world" in text

    def test_korean_roundtrip(self):
        w = Writer()
        w.add_paragraph("안녕하세요")
        w.add_paragraph("한글 문서입니다")
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        assert "안녕하세요" in text
        assert "한글 문서입니다" in text

    def test_empty_document_roundtrip(self):
        w = Writer()
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        text = doc.extract_text()
        # Empty document may produce whitespace or empty string
        assert isinstance(text, str)

    def test_document_repr(self):
        w = Writer()
        w.add_paragraph("Test document content")
        data = w.to_bytes()

        doc = Reader.from_bytes(data)
        r = repr(doc)
        assert "Document(" in r


class TestReaderFromFile:
    def test_from_file_nonexistent(self):
        import pytest

        with pytest.raises(OSError):
            Reader.from_file("/nonexistent/path/test.hwpx")

    def test_from_bytes_invalid(self):
        import pytest

        with pytest.raises(RuntimeError):
            Reader.from_bytes(b"not a valid hwpx file")

    def test_roundtrip_via_file(self, tmp_path):
        w = Writer()
        w.add_paragraph("File roundtrip test")
        path = str(tmp_path / "test.hwpx")
        w.save_to_file(path)

        doc = Reader.from_file(path)
        text = doc.extract_text()
        assert "File roundtrip test" in text
