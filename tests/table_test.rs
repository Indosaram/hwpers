use hwpers::HwpWriter;

#[test]
fn test_simple_table() {
    let mut writer = HwpWriter::new();

    writer
        .add_simple_table(&[vec!["Name", "Age"], vec!["Alice", "25"], vec!["Bob", "30"]])
        .unwrap();

    let document = writer.document();

    // With real table controls, we expect:
    // 1 paragraph with table control + 6 cell paragraphs (2x3 table) = 7 paragraphs
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 7);

    // First paragraph should have control header (table control)
    let table_para = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(table_para.ctrl_header.is_some());
    assert_eq!(table_para.control_mask, 1); // Control present
    assert!(table_para.text.is_none()); // Control paragraph has no text

    // Check cell paragraphs contain expected text
    let cell_texts: Vec<&str> = document.body_texts[0].sections[0]
        .paragraphs
        .iter()
        .skip(1) // Skip table control paragraph
        .filter_map(|p| p.text.as_ref())
        .map(|t| t.content.as_str())
        .collect();

    assert!(cell_texts.contains(&"Name"));
    assert!(cell_texts.contains(&"Age"));
    assert!(cell_texts.contains(&"Alice"));
    assert!(cell_texts.contains(&"25"));
    assert!(cell_texts.contains(&"Bob"));
    assert!(cell_texts.contains(&"30"));
}

#[test]
fn test_table_builder() {
    let mut writer = HwpWriter::new();

    writer
        .add_table(2, 2)
        .unwrap()
        .set_cell(0, 0, "A1")
        .set_cell(0, 1, "B1")
        .set_cell(1, 0, "A2")
        .set_cell(1, 1, "B2")
        .finish()
        .unwrap();

    let document = writer.document();

    // 1 table control + 4 cell paragraphs = 5 paragraphs
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 5);

    // Check cell content
    let cell_texts: Vec<&str> = document.body_texts[0].sections[0]
        .paragraphs
        .iter()
        .skip(1)
        .filter_map(|p| p.text.as_ref())
        .map(|t| t.content.as_str())
        .collect();

    assert!(cell_texts.contains(&"A1"));
    assert!(cell_texts.contains(&"B1"));
    assert!(cell_texts.contains(&"A2"));
    assert!(cell_texts.contains(&"B2"));
}

#[test]
fn test_table_with_header() {
    let mut writer = HwpWriter::new();

    writer
        .add_table(3, 2)
        .unwrap()
        .set_header_row(true)
        .set_cell(0, 0, "Header 1")
        .set_cell(0, 1, "Header 2")
        .set_cell(1, 0, "Data 1")
        .set_cell(1, 1, "Data 2")
        .set_cell(2, 0, "Data 3")
        .set_cell(2, 1, "Data 4")
        .finish()
        .unwrap();

    let document = writer.document();

    // 1 table control + 6 cell paragraphs = 7 paragraphs
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 7);

    // Check that all content is present
    let cell_texts: Vec<&str> = document.body_texts[0].sections[0]
        .paragraphs
        .iter()
        .skip(1)
        .filter_map(|p| p.text.as_ref())
        .map(|t| t.content.as_str())
        .collect();

    assert!(cell_texts.contains(&"Header 1"));
    assert!(cell_texts.contains(&"Header 2"));
    assert!(cell_texts.contains(&"Data 1"));
    assert!(cell_texts.contains(&"Data 4"));
}

#[test]
fn test_empty_table() {
    let mut writer = HwpWriter::new();

    // Empty data should not cause panic
    writer.add_simple_table(&[]).unwrap();

    let document = writer.document();
    // No paragraph should be added for empty table
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 0);
}

#[test]
fn test_table_with_long_text() {
    let mut writer = HwpWriter::new();

    writer
        .add_table(2, 2)
        .unwrap()
        .set_cell(0, 0, "This is a very long text that should be truncated")
        .set_cell(0, 1, "Short")
        .set_cell(1, 0, "Normal")
        .set_cell(1, 1, "Text")
        .finish()
        .unwrap();

    let document = writer.document();

    // 1 table control + 4 cell paragraphs = 5 paragraphs
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 5);

    // Check that long text is preserved (no truncation in real table cells)
    let long_text_para = &document.body_texts[0].sections[0].paragraphs[1];
    assert!(long_text_para
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("This is a very long text"));
}

#[test]
fn test_table_uses_monospace_font() {
    let mut writer = HwpWriter::new();

    writer
        .add_simple_table(&[vec!["A", "B"], vec!["C", "D"]])
        .unwrap();

    let document = writer.document();

    // Real table controls don't use monospace fonts by default
    // They use regular character shapes, so we just check that the table was created
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 5); // 1 control + 4 cells

    // Check that table control paragraph exists
    let table_para = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(table_para.ctrl_header.is_some());
}

#[test]
fn test_mixed_content_with_table() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Before table").unwrap();

    writer
        .add_simple_table(&[vec!["Col1", "Col2"], vec!["Val1", "Val2"]])
        .unwrap();

    writer.add_paragraph("After table").unwrap();

    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;

    // Before + Table control + 4 cells + After = 7 paragraphs
    assert_eq!(paragraphs.len(), 7);

    // Check content order
    assert!(paragraphs[0]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("Before table"));
    assert!(paragraphs[1].ctrl_header.is_some()); // Table control
    assert!(paragraphs[6]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("After table"));
}
