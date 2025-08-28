use hwpers::{HwpWriter, writer::style::ListType};

#[test]
fn test_simple_bullet_list() {
    let mut writer = HwpWriter::new();
    
    writer.add_list(
        &["Item 1", "Item 2", "Item 3"],
        ListType::Bullet,
    ).unwrap();
    
    let document = writer.document();
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 3);
    
    // Check that bullet points were added
    let para1 = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(para1.text.as_ref().unwrap().content.contains("•"));
}

#[test]
fn test_numbered_list() {
    let mut writer = HwpWriter::new();
    
    writer.add_list(
        &["First", "Second", "Third"],
        ListType::Numbered,
    ).unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    // Check numbering
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("1."));
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("2."));
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("3."));
}

#[test]
fn test_alphabetic_list() {
    let mut writer = HwpWriter::new();
    
    writer.add_list(
        &["Apple", "Banana", "Cherry"],
        ListType::Alphabetic,
    ).unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    // Check alphabetic numbering
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("a)"));
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("b)"));
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("c)"));
}

#[test]
fn test_korean_list() {
    let mut writer = HwpWriter::new();
    
    writer.add_list(
        &["첫째", "둘째", "셋째"],
        ListType::Korean,
    ).unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    // Check Korean numbering
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("가."));
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("나."));
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("다."));
}

#[test]
fn test_roman_numerals() {
    let mut writer = HwpWriter::new();
    
    writer.add_list(
        &["Introduction", "Methods", "Results", "Discussion"],
        ListType::Roman,
    ).unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    // Check Roman numerals
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("i."));
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("ii."));
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("iii."));
    assert!(paragraphs[3].text.as_ref().unwrap().content.contains("iv."));
}

#[test]
fn test_manual_list_building() {
    let mut writer = HwpWriter::new();
    
    writer.start_list(ListType::Numbered).unwrap();
    writer.add_list_item("First item").unwrap();
    writer.add_list_item("Second item").unwrap();
    writer.add_list_item("Third item").unwrap();
    writer.end_list().unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    assert_eq!(paragraphs.len(), 3);
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("1."));
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("2."));
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("3."));
}

#[test]
fn test_nested_lists() {
    let mut writer = HwpWriter::new();
    
    writer.start_list(ListType::Numbered).unwrap();
    writer.add_list_item("Main item 1").unwrap();
    
    // Start nested list
    writer.start_nested_list(ListType::Bullet).unwrap();
    writer.add_list_item("Nested item 1").unwrap();
    writer.add_list_item("Nested item 2").unwrap();
    writer.end_list().unwrap(); // End nested
    
    writer.add_list_item("Main item 2").unwrap();
    writer.end_list().unwrap(); // End main
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    assert_eq!(paragraphs.len(), 4);
    
    // Check main list numbering continues correctly
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("1."));
    assert!(paragraphs[3].text.as_ref().unwrap().content.contains("2."));
    
    // Check nested items have different indentation (para_shape_id)
    assert_ne!(paragraphs[0].para_shape_id, paragraphs[1].para_shape_id);
}

#[test]
fn test_list_indentation() {
    let mut writer = HwpWriter::new();
    
    writer.start_list(ListType::Bullet).unwrap();
    writer.add_list_item("Level 0").unwrap();
    
    writer.start_nested_list(ListType::Bullet).unwrap();
    writer.add_list_item("Level 1").unwrap();
    
    writer.start_nested_list(ListType::Bullet).unwrap();
    writer.add_list_item("Level 2").unwrap();
    writer.end_list().unwrap();
    
    writer.end_list().unwrap();
    writer.end_list().unwrap();
    
    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    
    // Check different bullet styles for different levels
    assert!(paragraphs[0].text.as_ref().unwrap().content.contains("•")); // Level 0
    assert!(paragraphs[1].text.as_ref().unwrap().content.contains("◦")); // Level 1
    assert!(paragraphs[2].text.as_ref().unwrap().content.contains("▪")); // Level 2
    
    // Check different indentation levels
    let shape_ids: Vec<_> = paragraphs.iter().map(|p| p.para_shape_id).collect();
    assert_ne!(shape_ids[0], shape_ids[1]);
    assert_ne!(shape_ids[1], shape_ids[2]);
    assert_ne!(shape_ids[0], shape_ids[2]);
}