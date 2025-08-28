use hwpers::{HwpWriter, model::{HyperlinkType, HyperlinkDisplay}};

#[test]
fn test_basic_hyperlink() {
    let mut writer = HwpWriter::new();
    
    // Add content
    writer.add_paragraph("This document contains hyperlinks.").unwrap();
    
    // Add basic hyperlink
    writer.add_hyperlink("Visit our website", "https://example.com").unwrap();
    
    let document = writer.document();
    
    // Check that we have paragraphs
    let section = &document.body_texts[0].sections[0];
    assert_eq!(section.paragraphs.len(), 2);
    
    // Check hyperlink paragraph
    let hyperlink_para = &section.paragraphs[1];
    assert_eq!(hyperlink_para.hyperlinks.len(), 1);
    
    let hyperlink = &hyperlink_para.hyperlinks[0];
    assert_eq!(hyperlink.display_text, "Visit our website");
    assert_eq!(hyperlink.target_url, "https://example.com");
    assert_eq!(hyperlink.hyperlink_type, HyperlinkType::Url);
    assert!(hyperlink.underline);
    assert_eq!(hyperlink.text_color, 0x0000FF); // Blue
}

#[test]
fn test_email_hyperlink() {
    let mut writer = HwpWriter::new();
    
    writer.add_paragraph("Contact information:").unwrap();
    writer.add_email_link("Send us an email", "contact@example.com").unwrap();
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let hyperlink_para = section.paragraphs.iter().find(|p| !p.hyperlinks.is_empty()).unwrap();
    let hyperlink = &hyperlink_para.hyperlinks[0];
    
    assert_eq!(hyperlink.display_text, "Send us an email");
    assert_eq!(hyperlink.target_url, "mailto:contact@example.com");
    assert_eq!(hyperlink.hyperlink_type, HyperlinkType::Email);
}

#[test]
fn test_file_hyperlink() {
    let mut writer = HwpWriter::new();
    
    writer.add_paragraph("Resources:").unwrap();
    writer.add_file_link("Download manual", "/documents/manual.pdf").unwrap();
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let hyperlink_para = section.paragraphs.iter().find(|p| !p.hyperlinks.is_empty()).unwrap();
    let hyperlink = &hyperlink_para.hyperlinks[0];
    
    assert_eq!(hyperlink.display_text, "Download manual");
    assert_eq!(hyperlink.target_url, "/documents/manual.pdf");
    assert_eq!(hyperlink.hyperlink_type, HyperlinkType::File);
    assert_eq!(hyperlink.text_color, 0x008000); // Green
}

#[test]
fn test_bookmark_hyperlink() {
    let mut writer = HwpWriter::new();
    
    writer.add_paragraph("Table of contents:").unwrap();
    writer.add_bookmark_link("Go to Chapter 1", "chapter1").unwrap();
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let hyperlink_para = section.paragraphs.iter().find(|p| !p.hyperlinks.is_empty()).unwrap();
    let hyperlink = &hyperlink_para.hyperlinks[0];
    
    assert_eq!(hyperlink.display_text, "Go to Chapter 1");
    assert_eq!(hyperlink.target_url, "#chapter1");
    assert_eq!(hyperlink.hyperlink_type, HyperlinkType::Bookmark);
    assert_eq!(hyperlink.text_color, 0x800080); // Purple
}

#[test]
fn test_custom_hyperlink() {
    let mut writer = HwpWriter::new();
    
    writer.add_paragraph("Custom link:").unwrap();
    writer.add_custom_hyperlink(
        "Custom styled link",
        HyperlinkType::Url,
        "https://custom.example.com",
        HyperlinkDisplay::Both,
        0xFF0000, // Red color
        false,    // No underline
        true,     // Open in new window
    ).unwrap();
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let hyperlink_para = section.paragraphs.iter().find(|p| !p.hyperlinks.is_empty()).unwrap();
    let hyperlink = &hyperlink_para.hyperlinks[0];
    
    assert_eq!(hyperlink.display_text, "Custom styled link");
    assert_eq!(hyperlink.target_url, "https://custom.example.com");
    assert_eq!(hyperlink.hyperlink_type, HyperlinkType::Url);
    assert_eq!(hyperlink.display_mode, HyperlinkDisplay::Both);
    assert_eq!(hyperlink.text_color, 0xFF0000); // Red
    assert!(!hyperlink.underline);
    assert!(hyperlink.open_in_new_window);
}

#[test]
fn test_multiple_hyperlinks_in_paragraph() {
    let mut writer = HwpWriter::new();
    
    let text = "Visit https://example.com or email us at contact@example.com for more info.";
    let hyperlinks = vec![
        hwpers::model::Hyperlink::web_link("https://example.com", "https://example.com")
            .with_position(6)
            .with_length(19),
        hwpers::model::Hyperlink::email_link("contact@example.com", "contact@example.com")
            .with_position(38)
            .with_length(19),
    ];
    
    writer.add_paragraph_with_hyperlinks(text, hyperlinks).unwrap();
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let hyperlink_para = &section.paragraphs[0];
    
    assert_eq!(hyperlink_para.hyperlinks.len(), 2);
    
    let first_link = &hyperlink_para.hyperlinks[0];
    assert_eq!(first_link.start_position, 6);
    assert_eq!(first_link.length, 19);
    assert_eq!(first_link.hyperlink_type, HyperlinkType::Url);
    
    let second_link = &hyperlink_para.hyperlinks[1];
    assert_eq!(second_link.start_position, 38);
    assert_eq!(second_link.length, 19);
    assert_eq!(second_link.hyperlink_type, HyperlinkType::Email);
}

#[test]
fn test_hyperlink_serialization() {
    let hyperlink = hwpers::model::Hyperlink::new_url("Test Link", "https://test.com")
        .with_tooltip("Click to visit test site")
        .with_text_color(0x0000FF)
        .with_underline(true)
        .with_new_window(true);
    
    let bytes = hyperlink.to_bytes();
    
    // Check that serialization produces data
    assert!(!bytes.is_empty());
    
    // Test round-trip parsing
    let header = hwpers::parser::record::RecordHeader {
        tag_id: 0x6C6E, // 'ln' - hypothetical hyperlink tag (shortened to fit u16)
        level: 0,
        size: bytes.len() as u32,
    };
    let record = hwpers::parser::record::Record {
        header,
        data: bytes,
    };
    let parsed_hyperlink = hwpers::model::Hyperlink::from_record(&record).unwrap();
    
    assert_eq!(parsed_hyperlink.display_text, hyperlink.display_text);
    assert_eq!(parsed_hyperlink.target_url, hyperlink.target_url);
    assert_eq!(parsed_hyperlink.hyperlink_type, hyperlink.hyperlink_type);
    assert_eq!(parsed_hyperlink.text_color, hyperlink.text_color);
    assert_eq!(parsed_hyperlink.underline, hyperlink.underline);
    assert_eq!(parsed_hyperlink.open_in_new_window, hyperlink.open_in_new_window);
    assert_eq!(parsed_hyperlink.tooltip, hyperlink.tooltip);
}

#[test]
fn test_predefined_hyperlink_styles() {
    let web_link = hwpers::model::Hyperlink::web_link("Web", "https://example.com");
    assert_eq!(web_link.hyperlink_type, HyperlinkType::Url);
    assert_eq!(web_link.text_color, 0x0000FF); // Blue
    assert!(web_link.underline);

    let email_link = hwpers::model::Hyperlink::email_link("Email", "test@example.com");
    assert_eq!(email_link.hyperlink_type, HyperlinkType::Email);
    assert_eq!(email_link.target_url, "mailto:test@example.com");

    let file_link = hwpers::model::Hyperlink::file_link("File", "/path/to/file");
    assert_eq!(file_link.hyperlink_type, HyperlinkType::File);
    assert_eq!(file_link.text_color, 0x008000); // Green

    let internal_link = hwpers::model::Hyperlink::internal_link("Internal", "bookmark");
    assert_eq!(internal_link.hyperlink_type, HyperlinkType::Bookmark);
    assert_eq!(internal_link.target_url, "#bookmark");
    assert_eq!(internal_link.text_color, 0x800080); // Purple

    let plain_link = hwpers::model::Hyperlink::plain_link("Plain", "https://example.com");
    assert!(!plain_link.underline);

    let external_link = hwpers::model::Hyperlink::external_link("External", "https://example.com");
    assert!(external_link.open_in_new_window);
}

#[test]
fn test_mixed_document_with_hyperlinks() {
    let mut writer = HwpWriter::new();
    
    // Add various document elements with hyperlinks
    writer.add_heading("Document with Hyperlinks", 1).unwrap();
    writer.add_paragraph("This document demonstrates hyperlink functionality.").unwrap();
    
    // Add different types of hyperlinks
    writer.add_hyperlink("Visit our homepage", "https://example.com").unwrap();
    writer.add_email_link("Contact support", "support@example.com").unwrap();
    writer.add_file_link("Download PDF", "/docs/guide.pdf").unwrap();
    writer.add_bookmark_link("Go to conclusion", "conclusion").unwrap();
    
    // Add a table
    writer.add_table(2, 2)
        .set_cell(0, 0, "Links")
        .set_cell(0, 1, "Description")
        .set_cell(1, 0, "Website")
        .set_cell(1, 1, "Main site")
        .finish().unwrap();
    
    // Add more hyperlinks
    writer.add_custom_hyperlink(
        "Special link",
        HyperlinkType::Url,
        "https://special.example.com",
        HyperlinkDisplay::Both,
        0xFF0000, // Red
        true,     // Underline
        true,     // New window
    ).unwrap();
    
    // Add header and footer
    writer.add_header("Hyperlink Demo");
    writer.add_footer_with_page_number("Page", hwpers::model::PageNumberFormat::Numeric);
    
    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    
    // Verify document structure
    assert!(!section.paragraphs.is_empty());
    
    // Count hyperlinks
    let total_hyperlinks: usize = section.paragraphs.iter()
        .map(|p| p.hyperlinks.len())
        .sum();
    assert_eq!(total_hyperlinks, 5); // 5 hyperlink paragraphs
    
    // Check different hyperlink types
    let hyperlink_types: Vec<HyperlinkType> = section.paragraphs.iter()
        .flat_map(|p| p.hyperlinks.iter())
        .map(|h| h.hyperlink_type)
        .collect();
    
    assert!(hyperlink_types.contains(&HyperlinkType::Url));
    assert!(hyperlink_types.contains(&HyperlinkType::Email));
    assert!(hyperlink_types.contains(&HyperlinkType::File));
    assert!(hyperlink_types.contains(&HyperlinkType::Bookmark));
    
    // Check headers and footers
    assert!(section.page_def.is_some());
    let page_def = section.page_def.as_ref().unwrap();
    assert_eq!(page_def.header_footer.headers().len(), 1);
    assert_eq!(page_def.header_footer.footers().len(), 1);
}