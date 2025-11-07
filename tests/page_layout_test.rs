use hwpers::{
    model::{
        inches_to_hwp_units, mm_to_hwp_units, PageLayout, PageMargins, PageOrientation, PaperSize,
    },
    HwpWriter,
};

#[test]
fn test_page_orientation() {
    let mut writer = HwpWriter::new();

    // Add some content
    writer.add_paragraph("Testing page orientation").unwrap();

    // Set landscape orientation
    writer.set_page_orientation(PageOrientation::Landscape);

    let layout = writer.get_page_layout();
    assert_eq!(layout.orientation, PageOrientation::Landscape);
    // For A4, landscape should have width > height
    assert!(layout.width > layout.height);
}

#[test]
fn test_paper_sizes() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Testing paper sizes").unwrap();

    // Test A4
    writer.set_paper_size(PaperSize::A4);
    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::A4);
    let (a4_width, a4_height) = PaperSize::A4.dimensions_hwp_units();
    assert_eq!(layout.width, a4_width);
    assert_eq!(layout.height, a4_height);

    // Test Letter
    writer.set_paper_size(PaperSize::Letter);
    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::Letter);
    let (letter_width, letter_height) = PaperSize::Letter.dimensions_hwp_units();
    assert_eq!(layout.width, letter_width);
    assert_eq!(layout.height, letter_height);

    // Test A3
    writer.set_paper_size(PaperSize::A3);
    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::A3);
    let (a3_width, a3_height) = PaperSize::A3.dimensions_hwp_units();
    assert_eq!(layout.width, a3_width);
    assert_eq!(layout.height, a3_height);
}

#[test]
fn test_page_margins_mm() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph("Testing margins in millimeters")
        .unwrap();

    // Set margins in millimeters
    writer.set_page_margins_mm(25.0, 30.0, 20.0, 15.0);

    let layout = writer.get_page_layout();
    assert_eq!(layout.margins.left, mm_to_hwp_units(25.0));
    assert_eq!(layout.margins.right, mm_to_hwp_units(30.0));
    assert_eq!(layout.margins.top, mm_to_hwp_units(20.0));
    assert_eq!(layout.margins.bottom, mm_to_hwp_units(15.0));
}

#[test]
fn test_page_margins_inches() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Testing margins in inches").unwrap();

    // Set margins in inches
    writer.set_page_margins_inches(1.0, 1.2, 0.8, 0.6);

    let layout = writer.get_page_layout();
    assert_eq!(layout.margins.left, inches_to_hwp_units(1.0));
    assert_eq!(layout.margins.right, inches_to_hwp_units(1.2));
    assert_eq!(layout.margins.top, inches_to_hwp_units(0.8));
    assert_eq!(layout.margins.bottom, inches_to_hwp_units(0.6));
}

#[test]
fn test_predefined_margins() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Testing predefined margins").unwrap();

    // Test narrow margins
    writer.set_narrow_margins();
    let layout = writer.get_page_layout();
    let narrow_margins = PageMargins::narrow();
    assert_eq!(layout.margins.left, narrow_margins.left);
    assert_eq!(layout.margins.right, narrow_margins.right);
    assert_eq!(layout.margins.top, narrow_margins.top);
    assert_eq!(layout.margins.bottom, narrow_margins.bottom);

    // Test normal margins
    writer.set_normal_margins();
    let layout = writer.get_page_layout();
    let normal_margins = PageMargins::normal();
    assert_eq!(layout.margins.left, normal_margins.left);
    assert_eq!(layout.margins.right, normal_margins.right);

    // Test wide margins
    writer.set_wide_margins();
    let layout = writer.get_page_layout();
    let wide_margins = PageMargins::wide();
    assert_eq!(layout.margins.left, wide_margins.left);
    assert_eq!(layout.margins.right, wide_margins.right);
}

#[test]
fn test_custom_page_size() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Testing custom page size").unwrap();

    // Set custom page size (300mm x 400mm)
    writer.set_custom_page_size_mm(300.0, 400.0);

    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::Custom);
    assert_eq!(layout.width, mm_to_hwp_units(300.0));
    assert_eq!(layout.height, mm_to_hwp_units(400.0));
}

#[test]
fn test_columns() {
    let mut writer = HwpWriter::new();

    writer.add_paragraph("Testing multiple columns").unwrap();

    // Set 3 columns with 5mm spacing
    writer.set_columns(3, 5.0);

    let layout = writer.get_page_layout();
    assert_eq!(layout.columns, 3);
    assert_eq!(layout.column_spacing, mm_to_hwp_units(5.0));

    // Test column width calculation
    let expected_width = layout.effective_width();
    let total_spacing = 2 * layout.column_spacing; // 2 gaps for 3 columns
    let expected_column_width = (expected_width - total_spacing) / 3;
    assert_eq!(layout.column_width(), expected_column_width);
}

#[test]
fn test_page_background_color() {
    let mut writer = HwpWriter::new();

    writer
        .add_paragraph("Testing page background color")
        .unwrap();

    // Set light blue background
    let blue_color = 0xE6F3FF;
    writer.set_page_background_color(blue_color);

    let layout = writer.get_page_layout();
    assert_eq!(layout.background_color, Some(blue_color));
}

#[test]
fn test_page_layout_builders() {
    // Test A4 portrait
    let a4_portrait = PageLayout::a4_portrait();
    assert_eq!(a4_portrait.paper_size, PaperSize::A4);
    assert_eq!(a4_portrait.orientation, PageOrientation::Portrait);
    let (width, height) = PaperSize::A4.dimensions_hwp_units();
    assert_eq!(a4_portrait.width, width);
    assert_eq!(a4_portrait.height, height);

    // Test A4 landscape
    let a4_landscape = PageLayout::a4_landscape();
    assert_eq!(a4_landscape.paper_size, PaperSize::A4);
    assert_eq!(a4_landscape.orientation, PageOrientation::Landscape);
    assert_eq!(a4_landscape.width, height); // Swapped for landscape
    assert_eq!(a4_landscape.height, width);

    // Test Letter portrait
    let letter_portrait = PageLayout::letter_portrait();
    assert_eq!(letter_portrait.paper_size, PaperSize::Letter);
    assert_eq!(letter_portrait.orientation, PageOrientation::Portrait);

    // Test custom size
    let custom = PageLayout::custom_mm(200.0, 300.0, PageOrientation::Portrait);
    assert_eq!(custom.paper_size, PaperSize::Custom);
    assert_eq!(custom.width, mm_to_hwp_units(200.0));
    assert_eq!(custom.height, mm_to_hwp_units(300.0));
}

#[test]
fn test_margin_builders() {
    // Test new_mm
    let margins_mm = PageMargins::new_mm(20.0, 25.0, 15.0, 10.0);
    assert_eq!(margins_mm.left, mm_to_hwp_units(20.0));
    assert_eq!(margins_mm.right, mm_to_hwp_units(25.0));
    assert_eq!(margins_mm.top, mm_to_hwp_units(15.0));
    assert_eq!(margins_mm.bottom, mm_to_hwp_units(10.0));

    // Test new_inches
    let margins_inches = PageMargins::new_inches(1.0, 1.5, 0.8, 0.6);
    assert_eq!(margins_inches.left, inches_to_hwp_units(1.0));
    assert_eq!(margins_inches.right, inches_to_hwp_units(1.5));
    assert_eq!(margins_inches.top, inches_to_hwp_units(0.8));
    assert_eq!(margins_inches.bottom, inches_to_hwp_units(0.6));

    // Test with_header_footer_mm
    let margins_with_hf =
        PageMargins::new_mm(20.0, 20.0, 20.0, 20.0).with_header_footer_mm(12.0, 10.0);
    assert_eq!(margins_with_hf.header, mm_to_hwp_units(12.0));
    assert_eq!(margins_with_hf.footer, mm_to_hwp_units(10.0));

    // Test with_gutter_mm
    let margins_with_gutter = PageMargins::new_mm(20.0, 20.0, 20.0, 20.0).with_gutter_mm(5.0);
    assert_eq!(margins_with_gutter.gutter, mm_to_hwp_units(5.0));

    // Test with_mirror_margins
    let margins_mirror = PageMargins::new_mm(20.0, 20.0, 20.0, 20.0).with_mirror_margins(true);
    assert!(margins_mirror.mirror_margins);
}

#[test]
fn test_effective_dimensions() {
    let layout =
        PageLayout::a4_portrait().with_margins(PageMargins::new_mm(25.0, 25.0, 20.0, 20.0));

    let expected_width = layout.width - mm_to_hwp_units(25.0) - mm_to_hwp_units(25.0);
    let expected_height = layout.height - mm_to_hwp_units(20.0) - mm_to_hwp_units(20.0);

    assert_eq!(layout.effective_width(), expected_width);
    assert_eq!(layout.effective_height(), expected_height);
}

#[test]
fn test_complex_page_layout() {
    let mut writer = HwpWriter::new();

    // Create a complex document with custom layout
    writer.add_heading("Complex Page Layout Test", 1).unwrap();
    writer
        .add_paragraph("This document demonstrates complex page layout features.")
        .unwrap();

    // Set A3 landscape with custom margins
    writer.set_paper_size(PaperSize::A3);
    writer.set_page_orientation(PageOrientation::Landscape);
    writer.set_page_margins_mm(30.0, 25.0, 20.0, 15.0);

    // Set 2 columns with column line
    writer.set_columns(2, 8.0);

    // Set background color
    writer.set_page_background_color(0xF8F8F8);

    // Add more content
    writer.add_paragraph("This text should appear in a 2-column layout on A3 landscape paper with custom margins and a light gray background.").unwrap();

    // Verify layout
    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::A3);
    assert_eq!(layout.orientation, PageOrientation::Landscape);
    assert_eq!(layout.columns, 2);
    assert_eq!(layout.column_spacing, mm_to_hwp_units(8.0));
    assert_eq!(layout.background_color, Some(0xF8F8F8));
    assert_eq!(layout.margins.left, mm_to_hwp_units(30.0));
    assert_eq!(layout.margins.right, mm_to_hwp_units(25.0));
    assert_eq!(layout.margins.top, mm_to_hwp_units(20.0));
    assert_eq!(layout.margins.bottom, mm_to_hwp_units(15.0));

    // Test that we have A3 landscape dimensions
    let (a3_width, a3_height) = PaperSize::A3.dimensions_hwp_units();
    assert_eq!(layout.width, a3_height); // Landscape: height becomes width
    assert_eq!(layout.height, a3_width); // Landscape: width becomes height
}

#[test]
fn test_paper_size_names() {
    assert_eq!(PaperSize::A4.name(), "A4");
    assert_eq!(PaperSize::A3.name(), "A3");
    assert_eq!(PaperSize::A5.name(), "A5");
    assert_eq!(PaperSize::Letter.name(), "Letter");
    assert_eq!(PaperSize::Legal.name(), "Legal");
    assert_eq!(PaperSize::Tabloid.name(), "Tabloid");
    assert_eq!(PaperSize::B4.name(), "B4");
    assert_eq!(PaperSize::B5.name(), "B5");
    assert_eq!(PaperSize::Custom.name(), "Custom");
}

#[test]
fn test_page_layout_serialization() {
    let layout = PageLayout::a4_landscape()
        .with_margins(PageMargins::new_mm(25.0, 25.0, 20.0, 20.0))
        .with_columns(3, 5.0)
        .with_background_color(0xFFE0E0);

    let bytes = layout.to_bytes();

    // Check that serialization produces data
    assert!(!bytes.is_empty());

    // Basic size check (should contain all the layout information)
    // This is a rough check as the exact format depends on implementation
    assert!(bytes.len() > 40); // Should have substantial data
}

#[test]
fn test_mixed_document_with_page_layout() {
    let mut writer = HwpWriter::new();

    // Set up custom page layout first
    writer.set_paper_size(PaperSize::Letter);
    writer.set_page_orientation(PageOrientation::Portrait);
    writer.set_normal_margins();
    writer.set_columns(1, 0.0); // Single column

    // Add document content
    writer
        .add_heading("Document with Custom Page Layout", 1)
        .unwrap();
    writer
        .add_paragraph("This document uses Letter paper size with normal margins.")
        .unwrap();

    // Add a table
    let table = writer
        .add_table(2, 3)
        .unwrap()
        .set_cell(0, 0, "Feature")
        .set_cell(0, 1, "Value")
        .set_cell(0, 2, "Unit")
        .set_cell(1, 0, "Paper Size")
        .set_cell(1, 1, "Letter")
        .set_cell(1, 2, "8.5×11 inch");
    table.finish().unwrap();

    // Add hyperlinks
    writer
        .add_hyperlink(
            "Letter paper info",
            "https://en.wikipedia.org/wiki/Letter_(paper_size)",
        )
        .unwrap();

    // Add text box
    writer
        .add_styled_text_box(
            "Note: This document uses Letter paper size which is common in North America.",
            "info",
        )
        .unwrap();

    // Add headers and footers
    writer.add_header("Custom Layout Demo");
    writer.add_footer_with_page_number("Page", hwpers::model::PageNumberFormat::Numeric);

    let document = writer.document();
    let section = &document.body_texts[0].sections[0];

    // Verify document structure
    assert!(!section.paragraphs.is_empty());

    // Verify page layout
    let layout = writer.get_page_layout();
    assert_eq!(layout.paper_size, PaperSize::Letter);
    assert_eq!(layout.orientation, PageOrientation::Portrait);
    assert_eq!(layout.columns, 1);

    // Verify headers and footers
    assert!(section.page_def.is_some());
    let page_def = section.page_def.as_ref().unwrap();
    assert_eq!(page_def.header_footer.headers().len(), 1);
    assert_eq!(page_def.header_footer.footers().len(), 1);
}
