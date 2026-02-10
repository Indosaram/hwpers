"""Tests for Table creation, from_data, cell operations."""

from hwpers import Table, Writer


class TestTable:
    def test_create_empty(self):
        t = Table(3, 4)
        assert "rows=3" in repr(t)
        assert "cols=4" in repr(t)

    def test_set_cell(self):
        t = Table(2, 2)
        t.set_cell(0, 0, "A")
        t.set_cell(0, 1, "B")
        t.set_cell(1, 0, "C")
        t.set_cell(1, 1, "D")
        rows = t.rows
        assert rows[0][0] == "A"
        assert rows[0][1] == "B"
        assert rows[1][0] == "C"
        assert rows[1][1] == "D"

    def test_set_cell_out_of_bounds(self):
        """Out-of-bounds set_cell should be silently ignored (matching Rust API)."""
        t = Table(2, 2)
        t.set_cell(5, 5, "X")
        rows = t.rows
        assert rows[0][0] == ""

    def test_from_data(self):
        data = [["A", "B", "C"], ["1", "2", "3"]]
        t = Table.from_data(data)
        assert t.rows == data

    def test_from_data_empty(self):
        t = Table.from_data([])
        assert t.rows == []

    def test_single_cell(self):
        t = Table(1, 1)
        t.set_cell(0, 0, "Only cell")
        rows = t.rows
        assert rows[0][0] == "Only cell"


class TestTableInWriter:
    def test_basic_table(self):
        w = Writer()
        t = Table.from_data([["Name", "Age"], ["Alice", "30"], ["Bob", "25"]])
        w.add_table(t)
        data = w.to_bytes()
        assert len(data) > 0

    def test_multiple_tables(self):
        w = Writer()
        w.add_paragraph("Table 1:")
        w.add_table(Table.from_data([["A", "B"], ["C", "D"]]))
        w.add_paragraph("Table 2:")
        w.add_table(Table.from_data([["1", "2"], ["3", "4"]]))
        data = w.to_bytes()
        assert len(data) > 0

    def test_table_with_special_chars(self):
        w = Writer()
        t = Table.from_data([["<html>", "&amp;"], ['"quote"', "normal"]])
        w.add_table(t)
        data = w.to_bytes()
        assert len(data) > 0

    def test_table_with_korean(self):
        w = Writer()
        t = Table.from_data([["이름", "나이"], ["홍길동", "30"]])
        w.add_table(t)
        data = w.to_bytes()
        assert len(data) > 0
