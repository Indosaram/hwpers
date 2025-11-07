use hwpers::writer::style::{StyledText, TextStyle};
use hwpers::HwpWriter;

#[test]
fn test_styled_paragraph() {
    let mut writer = HwpWriter::new();

    let styled_text = StyledText::new("Hello World!".to_string())
        .add_range(0, 5, TextStyle::new().bold()) // "Hello" - bold
        .add_range(6, 12, TextStyle::new().italic()); // "World!" - italic

    writer.add_styled_paragraph(&styled_text).unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    // Check that paragraph has text and character shapes
    assert!(paragraph.text.is_some());
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 2); // Two different styles

    // Check text content
    assert_eq!(paragraph.text.as_ref().unwrap().content, "Hello World!");

    // Check character shapes
    let char_shapes = paragraph.char_shapes.as_ref().unwrap();
    assert_eq!(char_shapes.char_positions.len(), 2);
    assert_eq!(char_shapes.char_positions[0].position, 0); // "Hello" starts at 0
    assert_eq!(char_shapes.char_positions[1].position, 6); // "World!" starts at 6
}

#[test]
fn test_mixed_text() {
    let mut writer = HwpWriter::new();

    writer
        .add_mixed_text(
            "Red text and blue text",
            vec![
                (0, 8, TextStyle::new().color(0xFF0000)), // "Red text" - red
                (13, 22, TextStyle::new().color(0x0000FF)), // "blue text" - blue
            ],
        )
        .unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 2); // Two color ranges
}

#[test]
fn test_bold_ranges() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph_with_bold(
            "This is bold text",
            vec![(8, 12)], // "bold" is bold
        )
        .unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert_eq!(
        paragraph.text.as_ref().unwrap().content,
        "This is bold text"
    );
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 1); // One bold range
}

#[test]
fn test_highlight_ranges() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph_with_highlight(
            "Text with yellow highlight",
            vec![(10, 16, 0xFFFF00)], // "yellow" highlighted in yellow
        )
        .unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert_eq!(
        paragraph.text.as_ref().unwrap().content,
        "Text with yellow highlight"
    );
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 1); // One highlight range
}

#[test]
fn test_color_ranges() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph_with_colors(
            "Red and blue text",
            vec![
                (0, 3, 0xFF0000),  // "Red" in red
                (8, 12, 0x0000FF), // "blue" in blue
            ],
        )
        .unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert_eq!(
        paragraph.text.as_ref().unwrap().content,
        "Red and blue text"
    );
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 2); // Two color ranges
}

#[test]
fn test_empty_styled_text() {
    let mut writer = HwpWriter::new();

    let styled_text = StyledText::new("Plain text".to_string());
    writer.add_styled_paragraph(&styled_text).unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert_eq!(paragraph.text.as_ref().unwrap().content, "Plain text");
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 1); // Default shape
}

#[test]
fn test_overlapping_ranges() {
    let mut writer = HwpWriter::new();

    let styled_text = StyledText::new("Overlapping styles".to_string())
        .add_range(0, 11, TextStyle::new().bold()) // "Overlapping" - bold
        .add_range(5, 18, TextStyle::new().italic()); // "lapping styles" - italic (overlaps)

    writer.add_styled_paragraph(&styled_text).unwrap();

    let document = writer.document();
    let paragraph = &document.body_texts[0].sections[0].paragraphs[0];

    assert!(paragraph.text.is_some());
    assert!(paragraph.char_shapes.is_some());
    assert_eq!(paragraph.char_shape_count, 2); // Two overlapping ranges
}

#[test]
fn test_styled_text_substring_search() {
    let styled_text = StyledText::new("Find and style this word".to_string())
        .style_substring("style", TextStyle::new().bold());

    // Check that the range was correctly identified
    assert_eq!(styled_text.ranges.len(), 1);
    assert_eq!(styled_text.ranges[0].start, 9);
    assert_eq!(styled_text.ranges[0].end, 14);
}

#[test]
fn test_styled_text_all_occurrences() {
    let styled_text = StyledText::new("test and test again test".to_string())
        .style_all_occurrences("test", TextStyle::new().underline());

    // Should find all 3 occurrences of "test"
    assert_eq!(styled_text.ranges.len(), 3);
    assert_eq!(styled_text.ranges[0].start, 0); // First "test"
    assert_eq!(styled_text.ranges[0].end, 4);
    assert_eq!(styled_text.ranges[1].start, 9); // Second "test"
    assert_eq!(styled_text.ranges[1].end, 13);
    assert_eq!(styled_text.ranges[2].start, 20); // Third "test"
    assert_eq!(styled_text.ranges[2].end, 24);
}
