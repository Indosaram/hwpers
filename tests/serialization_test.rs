use hwpers::writer::HwpWriter;
use hwpers::HwpReader;
use std::fs;

#[test]
fn test_hyperlink_serialization() {
    let mut writer = HwpWriter::new();
    
    // Create hyperlink using the model
    let hyperlink = hwpers::model::hyperlink::Hyperlink::new_url(
        "Rust Homepage",
        "https://rust-lang.org"
    );
    
    // Add paragraph with hyperlink
    writer.add_paragraph_with_hyperlinks(
        "Visit our website at Rust Homepage for more information.",
        vec![hyperlink]
    ).expect("Failed to add hyperlink");
    
    // Write to file
    let output_path = "test_hyperlink.hwp";
    writer.save_to_file(output_path).expect("Failed to save file");
    
    // Read back and verify
    let document = HwpReader::from_file(output_path).expect("Failed to read file");
    let text = document.extract_text();
    assert!(text.contains("Visit our website"));
    assert!(text.contains("Rust Homepage"));
    assert!(text.contains("for more information"));
    
    // Clean up
    fs::remove_file(output_path).ok();
}

#[test] 
fn test_header_footer_serialization() {
    let mut writer = HwpWriter::new();
    
    // Add header
    writer.add_header("Test Document Header");
    
    // Add footer with page number
    writer.add_footer_with_page_number("Page ", hwpers::model::header_footer::PageNumberFormat::Numeric);
    
    // Add some content
    writer.add_paragraph("This is the main content of the document.").expect("Failed to add paragraph");
    
    // Write to file
    let output_path = "test_header_footer.hwp";
    writer.save_to_file(output_path).expect("Failed to save file");
    
    // Read back and verify structure exists
    let document = HwpReader::from_file(output_path).expect("Failed to read file");
    assert!(document.sections().count() > 0);
    
    // Clean up
    fs::remove_file(output_path).ok();
}

#[test]
fn test_page_layout_serialization() {
    let mut writer = HwpWriter::new();
    
    // Set custom page layout
    writer.set_custom_page_size(210.0, 297.0, hwpers::model::page_layout::PageOrientation::Portrait)
        .expect("Failed to set page size"); // A4
    writer.set_page_margins_mm(20.0, 20.0, 20.0, 20.0).expect("Failed to set margins");
    
    // Add content
    writer.add_paragraph("Test page with custom layout").expect("Failed to add paragraph");
    
    // Write to file
    let output_path = "test_page_layout.hwp";
    writer.save_to_file(output_path).expect("Failed to save file");
    
    // Read back and verify
    let document = HwpReader::from_file(output_path).expect("Failed to read file");
    let text = document.extract_text();
    assert!(text.contains("Test page with custom layout"));
    
    // Clean up
    fs::remove_file(output_path).ok();
}

#[test]
fn test_paragraph_formatting_serialization() {
    let mut writer = HwpWriter::new();
    
    // Add paragraphs with different alignments
    writer.add_aligned_paragraph(
        "Left aligned text",
        hwpers::writer::style::ParagraphAlignment::Left
    ).expect("Failed to add left aligned");
    
    writer.add_aligned_paragraph(
        "Center aligned text",
        hwpers::writer::style::ParagraphAlignment::Center
    ).expect("Failed to add center aligned");
    
    writer.add_aligned_paragraph(
        "Right aligned text", 
        hwpers::writer::style::ParagraphAlignment::Right
    ).expect("Failed to add right aligned");
    
    // Add paragraph with custom spacing
    writer.add_paragraph_with_spacing(
        "Text with custom spacing",
        150, // line spacing
        10.0,  // before spacing  
        10.0   // after spacing
    ).expect("Failed to add paragraph with spacing");
    
    // Write to file
    let output_path = "test_paragraph_format.hwp";
    writer.save_to_file(output_path).expect("Failed to save file");
    
    // Read back and verify
    let document = HwpReader::from_file(output_path).expect("Failed to read file");
    let text = document.extract_text();
    assert!(text.contains("Left aligned text"));
    assert!(text.contains("Center aligned text"));
    assert!(text.contains("Right aligned text"));
    assert!(text.contains("Text with custom spacing"));
    
    // Clean up
    fs::remove_file(output_path).ok();
}