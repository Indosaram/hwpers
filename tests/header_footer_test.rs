use hwpers::{
    model::{HeaderFooterAlignment, PageApplyType, PageNumberFormat},
    HwpWriter,
};

#[test]
fn test_basic_header() {
    let mut writer = HwpWriter::new();

    // Add basic content
    writer
        .add_paragraph("This is a document with header.")
        .unwrap();

    // Add header
    writer.add_header("Document Title");

    let document = writer.document();

    // Check that page definition exists with header
    let section = &document.body_texts[0].sections[0];
    assert!(section.page_def.is_some());

    let page_def = section.page_def.as_ref().unwrap();
    let headers = page_def.header_footer.headers();
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].text, "Document Title");
    assert_eq!(headers[0].apply_type, PageApplyType::All);
    assert_eq!(headers[0].alignment, 0); // Left alignment
}

#[test]
fn test_basic_footer() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph("This is a document with footer.")
        .unwrap();
    writer.add_footer("Copyright 2023");

    let document = writer.document();

    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();
    let footers = page_def.header_footer.footers();

    assert_eq!(footers.len(), 1);
    assert_eq!(footers[0].text, "Copyright 2023");
    assert_eq!(footers[0].apply_type, PageApplyType::All);
}

#[test]
fn test_header_with_options() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Content").unwrap();
    writer.add_header_with_options(
        "Centered Header",
        PageApplyType::OddPages,
        HeaderFooterAlignment::Center,
    );

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();
    let headers = page_def.header_footer.headers();

    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].text, "Centered Header");
    assert_eq!(headers[0].apply_type, PageApplyType::OddPages);
    assert_eq!(headers[0].alignment, 1); // Center alignment
}

#[test]
fn test_footer_with_options() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Content").unwrap();
    writer.add_footer_with_options(
        "Right Footer",
        PageApplyType::EvenPages,
        HeaderFooterAlignment::Right,
    );

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();
    let footers = page_def.header_footer.footers();

    assert_eq!(footers.len(), 1);
    assert_eq!(footers[0].text, "Right Footer");
    assert_eq!(footers[0].apply_type, PageApplyType::EvenPages);
    assert_eq!(footers[0].alignment, 2); // Right alignment
}

#[test]
fn test_header_with_page_number() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Document with page numbers").unwrap();
    writer.add_header_with_page_number("Page", PageNumberFormat::Numeric);

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();
    let headers = page_def.header_footer.headers();

    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].text, "Page");
    assert!(headers[0].include_page_number);
    assert_eq!(headers[0].page_number_format, 1); // Numeric format
    assert_eq!(headers[0].alignment, 1); // Center alignment (default for page numbers)
}

#[test]
fn test_footer_with_page_number() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Content").unwrap();
    writer.add_footer_with_page_number("Page", PageNumberFormat::RomanLower);

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();
    let footers = page_def.header_footer.footers();

    assert_eq!(footers.len(), 1);
    assert_eq!(footers[0].text, "Page");
    assert!(footers[0].include_page_number);
    assert_eq!(footers[0].page_number_format, 2); // Roman lower format
}

#[test]
fn test_multiple_headers_and_footers() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Content").unwrap();

    // Add different headers for different page types
    writer.add_header_with_options(
        "First Page Header",
        PageApplyType::FirstPage,
        HeaderFooterAlignment::Center,
    );
    writer.add_header_with_options(
        "Odd Page Header",
        PageApplyType::OddPages,
        HeaderFooterAlignment::Left,
    );
    writer.add_header_with_options(
        "Even Page Header",
        PageApplyType::EvenPages,
        HeaderFooterAlignment::Right,
    );

    // Add footers
    writer.add_footer("Left Footer");
    writer.add_footer_with_page_number("Page", PageNumberFormat::Numeric);

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];
    let page_def = section.page_def.as_ref().unwrap();

    // Check headers
    let headers = page_def.header_footer.headers();
    assert_eq!(headers.len(), 3);

    // Check footers
    let footers = page_def.header_footer.footers();
    assert_eq!(footers.len(), 2);

    // Verify specific headers
    let first_page_header = page_def.header_footer.find_by_type(
        hwpers::model::HeaderFooterType::Header,
        PageApplyType::FirstPage,
    );
    assert!(first_page_header.is_some());
    assert_eq!(first_page_header.unwrap().text, "First Page Header");
}

#[test]
fn test_header_footer_serialization() {
    let header = hwpers::model::HeaderFooter::new_header("Test Header")
        .with_page_number(PageNumberFormat::Numeric)
        .with_alignment(HeaderFooterAlignment::Center)
        .with_apply_type(PageApplyType::All);

    let bytes = header.to_bytes();

    // Check that serialization produces some data
    assert!(!bytes.is_empty());

    // The first few bytes should represent type, apply type, alignment, etc.
    assert_eq!(bytes[0], 0); // Header type
    assert_eq!(bytes[1], 0); // All pages
    assert_eq!(bytes[2], 1); // Center alignment
    assert_eq!(bytes[3], 1); // Include page number
    assert_eq!(bytes[4], 1); // Numeric format
}

#[test]
fn test_mixed_document_with_headers_footers() {
    let mut writer = HwpWriter::new();

    // Add document content
    writer.add_heading("Chapter 1", 1).unwrap();
    writer
        .add_paragraph("This is the first paragraph.")
        .unwrap();

    // Add table
    let table = writer
        .add_table(2, 2)
        .unwrap()
        .set_cell(0, 0, "A1")
        .set_cell(0, 1, "B1")
        .set_cell(1, 0, "A2")
        .set_cell(1, 1, "B2");
    table.finish().unwrap();

    // Add headers and footers
    writer.add_header("Document Title");
    writer.add_footer_with_page_number("Page", PageNumberFormat::Numeric);

    // Add more content
    writer.add_paragraph("Final paragraph.").unwrap();

    let document = writer.document();

    // Verify document structure
    let section = &document.body_texts[0].sections[0];
    assert!(section.page_def.is_some());

    let page_def = section.page_def.as_ref().unwrap();
    assert_eq!(page_def.header_footer.headers().len(), 1);
    assert_eq!(page_def.header_footer.footers().len(), 1);

    // Check that paragraphs and table exist
    assert!(!section.paragraphs.is_empty());

    // Find table paragraph
    let table_para = section.paragraphs.iter().find(|p| p.table_data.is_some());
    assert!(table_para.is_some());
}
