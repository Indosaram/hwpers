use hwpers::{
    model::{TextBoxAlignment, TextBoxBorderStyle, TextBoxFillType},
    HwpWriter,
};

#[test]
fn test_basic_text_box() {
    let mut writer = HwpWriter::new();

    // Add content
    writer
        .add_paragraph("This document contains a text box.")
        .unwrap();

    // Add basic text box
    writer.add_text_box("This is a basic text box").unwrap();

    let document = writer.document();

    // Check that we have paragraphs
    let section = &document.body_texts[0].sections[0];
    assert_eq!(section.paragraphs.len(), 2);

    // Check text box paragraph
    let text_box_para = &section.paragraphs[1];
    assert!(text_box_para.text_box_data.is_some());
    assert!(text_box_para.ctrl_header.is_some());
    assert_eq!(text_box_para.control_mask, 0x02); // Control header present

    let text_box = text_box_para.text_box_data.as_ref().unwrap();
    assert_eq!(text_box.text, "This is a basic text box");
    assert_eq!(text_box.alignment, TextBoxAlignment::Inline);
}

#[test]
fn test_positioned_text_box() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Document content").unwrap();
    writer
        .add_text_box_at_position("Positioned text box", 10, 20, 50, 30)
        .unwrap();

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let text_box_para = section
        .paragraphs
        .iter()
        .find(|p| p.text_box_data.is_some())
        .unwrap();
    let text_box = text_box_para.text_box_data.as_ref().unwrap();

    assert_eq!(text_box.text, "Positioned text box");
    assert_eq!(text_box.x, 1000); // 10mm in HWP units
    assert_eq!(text_box.y, 2000); // 20mm in HWP units
    assert_eq!(text_box.width, 5000); // 50mm in HWP units
    assert_eq!(text_box.height, 3000); // 30mm in HWP units
}

#[test]
fn test_styled_text_boxes() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph("Testing different text box styles")
        .unwrap();

    // Test different predefined styles
    writer.add_styled_text_box("Basic style", "basic").unwrap();
    writer
        .add_styled_text_box("Highlight style", "highlight")
        .unwrap();
    writer
        .add_styled_text_box("Warning style", "warning")
        .unwrap();
    writer.add_styled_text_box("Info style", "info").unwrap();
    writer
        .add_styled_text_box("Transparent style", "transparent")
        .unwrap();
    writer
        .add_styled_text_box("Bubble style", "bubble")
        .unwrap();

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];

    // Count text box paragraphs
    let text_box_count = section
        .paragraphs
        .iter()
        .filter(|p| p.text_box_data.is_some())
        .count();
    assert_eq!(text_box_count, 6);

    // Test specific styles
    let highlight_para = section
        .paragraphs
        .iter()
        .find(|p| {
            p.text_box_data
                .as_ref()
                .map_or(false, |tb| tb.text == "Highlight style")
        })
        .unwrap();
    let highlight_box = highlight_para.text_box_data.as_ref().unwrap();
    assert_eq!(highlight_box.background_color, 0xFFFF00); // Yellow background

    let warning_para = section
        .paragraphs
        .iter()
        .find(|p| {
            p.text_box_data
                .as_ref()
                .map_or(false, |tb| tb.text == "Warning style")
        })
        .unwrap();
    let warning_box = warning_para.text_box_data.as_ref().unwrap();
    assert_eq!(warning_box.border_color, 0xFF0000); // Red border
}

#[test]
fn test_custom_text_box() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Custom text box test").unwrap();
    writer
        .add_custom_text_box(
            "Custom styled text box",
            15,
            25, // position
            80,
            40, // size
            TextBoxAlignment::Center,
            TextBoxBorderStyle::Dashed,
            0x0000FF, // blue border
            0xF0F0F0, // light gray background
        )
        .unwrap();

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let text_box_para = section
        .paragraphs
        .iter()
        .find(|p| p.text_box_data.is_some())
        .unwrap();
    let text_box = text_box_para.text_box_data.as_ref().unwrap();

    assert_eq!(text_box.text, "Custom styled text box");
    assert_eq!(text_box.x, 1500); // 15mm
    assert_eq!(text_box.y, 2500); // 25mm
    assert_eq!(text_box.width, 8000); // 80mm
    assert_eq!(text_box.height, 4000); // 40mm
    assert_eq!(text_box.alignment, TextBoxAlignment::Center);
    assert_eq!(text_box.border_style, TextBoxBorderStyle::Dashed);
    assert_eq!(text_box.border_color, 0x0000FF);
    assert_eq!(text_box.background_color, 0xF0F0F0);
}

#[test]
fn test_floating_text_box() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Floating text box test").unwrap();
    writer
        .add_floating_text_box(
            "Floating box with rotation",
            30,
            40, // position
            60,
            20,  // size
            180, // semi-transparent
            45,  // 45 degree rotation
        )
        .unwrap();

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let text_box_para = section
        .paragraphs
        .iter()
        .find(|p| p.text_box_data.is_some())
        .unwrap();
    let text_box = text_box_para.text_box_data.as_ref().unwrap();

    assert_eq!(text_box.text, "Floating box with rotation");
    assert_eq!(text_box.alignment, TextBoxAlignment::Absolute);
    assert_eq!(text_box.fill_type, TextBoxFillType::None);
    assert_eq!(text_box.opacity, 180);
    assert_eq!(text_box.rotation, 45);
}

#[test]
fn test_text_box_serialization() {
    let text_box = hwpers::model::TextBox::new("Test serialization")
        .with_position_mm(10, 20)
        .with_size_mm(50, 30)
        .with_alignment(TextBoxAlignment::Center)
        .with_border(TextBoxBorderStyle::Solid, 2, 0x000000)
        .with_background(0xFFFFFF);

    let bytes = text_box.to_bytes();

    // Check that serialization produces data
    assert!(!bytes.is_empty());

    // Test round-trip parsing
    let mut record_data = Vec::new();
    record_data.extend_from_slice(&bytes);

    // Create a mock record for testing
    let header = hwpers::parser::record::RecordHeader {
        tag_id: 0x7874,
        level: 0,
        size: record_data.len() as u32,
    };
    let record = hwpers::parser::record::Record {
        header,
        data: record_data,
    };
    let parsed_text_box = hwpers::model::TextBox::from_record(&record).unwrap();

    assert_eq!(parsed_text_box.text, text_box.text);
    assert_eq!(parsed_text_box.x, text_box.x);
    assert_eq!(parsed_text_box.y, text_box.y);
    assert_eq!(parsed_text_box.width, text_box.width);
    assert_eq!(parsed_text_box.height, text_box.height);
    assert_eq!(parsed_text_box.alignment, text_box.alignment);
}

#[test]
fn test_multiple_text_boxes() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph("Document with multiple text boxes")
        .unwrap();

    // Add various text boxes
    writer.add_text_box("First text box").unwrap();
    writer
        .add_text_box_at_position("Second text box", 50, 60, 40, 25)
        .unwrap();
    writer
        .add_styled_text_box("Third text box", "info")
        .unwrap();
    writer
        .add_custom_text_box(
            "Fourth text box",
            100,
            120,
            70,
            35,
            TextBoxAlignment::Right,
            TextBoxBorderStyle::Double,
            0x800080, // purple
            0xFFE0FF, // light purple
        )
        .unwrap();

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];

    // Count text box paragraphs
    let text_box_count = section
        .paragraphs
        .iter()
        .filter(|p| p.text_box_data.is_some())
        .count();
    assert_eq!(text_box_count, 4);

    // Verify each text box has different properties
    let text_boxes: Vec<_> = section
        .paragraphs
        .iter()
        .filter_map(|p| p.text_box_data.as_ref())
        .collect();

    assert_eq!(text_boxes[0].text, "First text box");
    assert_eq!(text_boxes[1].text, "Second text box");
    assert_eq!(text_boxes[2].text, "Third text box");
    assert_eq!(text_boxes[3].text, "Fourth text box");

    // Check different properties
    assert_eq!(text_boxes[1].x, 5000); // 50mm
    assert_eq!(text_boxes[2].border_color, 0x0000FF); // Blue (info style)
    assert_eq!(text_boxes[3].alignment, TextBoxAlignment::Right);
    assert_eq!(text_boxes[3].border_style, TextBoxBorderStyle::Double);
}

#[test]
fn test_mixed_document_with_text_boxes() {
    let mut writer = HwpWriter::new();

    // Add various document elements with text boxes
    writer
        .add_heading("Document with Mixed Content", 1)
        .unwrap();
    writer
        .add_paragraph("This document demonstrates text boxes alongside other content.")
        .unwrap();

    // Add a table
    let table = writer
        .add_table(2, 2)
        .unwrap()
        .set_cell(0, 0, "A1")
        .set_cell(0, 1, "B1")
        .set_cell(1, 0, "A2")
        .set_cell(1, 1, "B2");
    table.finish().unwrap();

    // Add text boxes
    writer
        .add_styled_text_box("Highlighted note", "highlight")
        .unwrap();
    writer
        .add_text_box_at_position("Side comment", 120, 80, 60, 25)
        .unwrap();

    // Add more content
    writer
        .add_paragraph("Additional content after text boxes.")
        .unwrap();

    // Add header and footer
    writer.add_header("Document Title");
    writer.add_footer_with_page_number("Page", hwpers::model::PageNumberFormat::Numeric);

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];

    // Verify document structure
    assert!(!section.paragraphs.is_empty());

    // Count different types of content
    let text_box_count = section
        .paragraphs
        .iter()
        .filter(|p| p.text_box_data.is_some())
        .count();
    let table_count = section
        .paragraphs
        .iter()
        .filter(|p| p.table_data.is_some())
        .count();
    let text_paragraph_count = section
        .paragraphs
        .iter()
        .filter(|p| p.text.is_some())
        .count();

    assert_eq!(text_box_count, 2);
    assert_eq!(table_count, 1);
    assert!(text_paragraph_count >= 3);

    // Check headers and footers
    assert!(section.page_def.is_some());
    let page_def = section.page_def.as_ref().unwrap();
    assert_eq!(page_def.header_footer.headers().len(), 1);
    assert_eq!(page_def.header_footer.footers().len(), 1);
}
