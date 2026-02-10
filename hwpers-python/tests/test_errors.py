"""Tests for error mapping: Rust errors -> Python exceptions."""

import pytest
from hwpers import Writer, Reader, Image


class TestErrorMapping:
    def test_save_to_nonexistent_dir_raises_oserror(self):
        w = Writer()
        w.add_paragraph("Test")
        with pytest.raises(OSError):
            w.save_to_file("/nonexistent/dir/test.hwpx")

    def test_reader_file_not_found_raises_oserror(self):
        with pytest.raises(OSError):
            Reader.from_file("/nonexistent/path/file.hwpx")

    def test_reader_invalid_bytes_raises_runtime_error(self):
        with pytest.raises(RuntimeError):
            Reader.from_bytes(b"invalid data")

    def test_image_invalid_format_raises_value_error(self):
        with pytest.raises(ValueError):
            Image.from_bytes(b"not an image")

    def test_image_empty_raises_value_error(self):
        with pytest.raises(ValueError):
            Image.from_bytes(b"")

    def test_image_too_short_raises_value_error(self):
        with pytest.raises(ValueError):
            Image.from_bytes(b"\x00\x01\x02")
