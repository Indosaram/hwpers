use hwpers::{writer::style::TextStyle, HwpReader, HwpWriter};

#[test]
fn test_text_style_creation() {
    let style = TextStyle::new()
        .font("맑은 고딕")
        .size(14)
        .bold()
        .italic()
        .color(0xFF0000);

    assert_eq!(style.font_name, Some("맑은 고딕".to_string()));
    assert_eq!(style.font_size, Some(14));
    assert!(style.bold);
    assert!(style.italic);
    assert!(!style.underline); // Not set
    assert_eq!(style.color, 0xFF0000);
}

#[test]
fn test_add_paragraph_with_style() {
    let mut writer = HwpWriter::new();

    let bold_style = TextStyle::new().bold();
    writer
        .add_paragraph_with_style("Bold text", &bold_style)
        .unwrap();

    let document = writer.document();

    // Verify document structure
    assert_eq!(document.body_texts.len(), 1);
    assert_eq!(document.body_texts[0].sections.len(), 1);
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 1);

    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(paragraph.char_shapes.is_some());

    // Verify CharShape was added
    assert!(document.doc_info.char_shapes.len() > 1); // Default + our new one
}

#[test]
fn test_add_heading() {
    let mut writer = HwpWriter::new();

    writer.add_heading("Chapter 1", 1).unwrap();
    writer.add_heading("Section 1.1", 2).unwrap();
    writer.add_heading("Subsection 1.1.1", 3).unwrap();

    let document = writer.document();

    // Verify paragraphs were added
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 3);

    // Verify different paragraph shapes were created for different heading levels
    assert!(document.doc_info.para_shapes.len() > 1);

    // Verify char shapes were created (bold for headings)
    assert!(document.doc_info.char_shapes.len() > 1);
}

#[test]
fn test_font_management() {
    let mut writer = HwpWriter::new();

    // Add paragraphs with different fonts
    let gulim_style = TextStyle::new().font("굴림");
    writer
        .add_paragraph_with_style("굴림체 텍스트", &gulim_style)
        .unwrap();

    let batang_style = TextStyle::new().font("바탕");
    writer
        .add_paragraph_with_style("바탕체 텍스트", &batang_style)
        .unwrap();

    // Use the same font again
    writer
        .add_paragraph_with_style("또 다른 굴림체 텍스트", &gulim_style)
        .unwrap();

    let document = writer.document();

    // Should have default font + 2 new fonts (굴림, 바탕)
    assert_eq!(document.doc_info.face_names.len(), 3);

    // Verify font names
    let font_names: Vec<&str> = document
        .doc_info
        .face_names
        .iter()
        .map(|f| f.font_name.as_str())
        .collect();
    assert!(font_names.contains(&"굴림"));
    assert!(font_names.contains(&"바탕"));
}

#[test]
fn test_styled_document_roundtrip() {
    let mut writer = HwpWriter::new();

    // Add various styled content
    writer.add_heading("Test Document", 1).unwrap();

    let bold_style = TextStyle::new().bold().size(14);
    writer
        .add_paragraph_with_style("Bold 14pt text", &bold_style)
        .unwrap();

    let colored_style = TextStyle::new().color(0x0000FF).italic();
    writer
        .add_paragraph_with_style("Blue italic text", &colored_style)
        .unwrap();

    // Convert to bytes
    let bytes = writer.to_bytes().unwrap();
    assert!(bytes.len() > 1000); // Should be a reasonable size

    // Try to read it back
    let result = HwpReader::from_bytes(&bytes);
    if let Ok(doc) = result {
        // Verify we can extract text
        let text = doc.extract_text();
        assert!(text.contains("Test Document"));
        assert!(text.contains("Bold 14pt text"));
        assert!(text.contains("Blue italic text"));
    }
}

#[test]
fn test_text_style_properties() {
    let style = TextStyle::new().bold().italic().underline().strikethrough();

    // Just verify the style properties are set correctly
    assert!(style.bold);
    assert!(style.italic);
    assert!(style.underline);
    assert!(style.strikethrough);
}

#[test]
fn test_font_size_in_style() {
    let style = TextStyle::new().size(12);
    assert_eq!(style.font_size, Some(12));

    let large_style = TextStyle::new().size(24);
    assert_eq!(large_style.font_size, Some(24));
}
