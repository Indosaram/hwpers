use hwpers::{
    model::{TextBoxAlignment, TextBoxBorderStyle},
    HwpWriter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();

    // Set document metadata
    writer.set_document_title("Text Box Demo Document")?;

    // Add header
    writer.add_header("Text Box Features Demo");

    // Add title
    writer.add_heading("Text Box Examples", 1)?;

    // Add introduction paragraph
    writer.add_paragraph("This document demonstrates various text box features available in the HWP writer. Text boxes can be used to create callouts, sidebars, annotations, and other visual elements.")?;

    // Example 1: Basic text box
    writer.add_heading("1. Basic Text Box", 2)?;
    writer.add_paragraph("Below is a basic text box with default styling:")?;
    writer.add_text_box("This is a basic text box with default settings.")?;

    // Example 2: Positioned text box
    writer.add_heading("2. Positioned Text Box", 2)?;
    writer.add_paragraph("Text boxes can be positioned at specific coordinates:")?;
    writer.add_text_box_at_position(
        "This text box is positioned at 50mm, 30mm with size 80mm x 25mm",
        50,
        30,
        80,
        25,
    )?;

    // Example 3: Styled text boxes
    writer.add_heading("3. Predefined Styles", 2)?;
    writer.add_paragraph("Several predefined styles are available:")?;

    writer.add_styled_text_box("Highlight style - great for important notes", "highlight")?;
    writer.add_styled_text_box("Warning style - for alerts and cautions", "warning")?;
    writer.add_styled_text_box("Info style - for helpful information", "info")?;
    writer.add_styled_text_box("Transparent style - subtle background", "transparent")?;
    writer.add_styled_text_box("Bubble style - for quotes and comments", "bubble")?;

    // Example 4: Custom styling
    writer.add_heading("4. Custom Styling", 2)?;
    writer.add_paragraph("You can create custom styled text boxes:")?;

    writer.add_custom_text_box(
        "Custom blue box with dashed border",
        20,
        150, // position
        100,
        40, // size
        TextBoxAlignment::Center,
        TextBoxBorderStyle::Dashed,
        0x0000FF, // blue border
        0xE6F3FF, // light blue background
    )?;

    writer.add_custom_text_box(
        "Green success message box",
        130,
        150, // position
        90,
        35, // size
        TextBoxAlignment::Left,
        TextBoxBorderStyle::Solid,
        0x00AA00, // green border
        0xE6FFE6, // light green background
    )?;

    // Example 5: Floating text box with rotation
    writer.add_heading("5. Floating and Rotated Text Box", 2)?;
    writer.add_paragraph("Text boxes can be floating with transparency and rotation:")?;

    writer.add_floating_text_box(
        "ROTATED WATERMARK",
        60,
        180, // position
        120,
        20,  // size
        180, // semi-transparent
        45,  // 45 degree rotation
    )?;

    // Example 6: Multiple text boxes layout
    writer.add_heading("6. Multiple Text Boxes Layout", 2)?;
    writer.add_paragraph("Create complex layouts with multiple text boxes:")?;

    // Top row
    writer.add_text_box_at_position("Header Info", 10, 220, 60, 20)?;
    writer.add_text_box_at_position("Title Area", 80, 220, 80, 20)?;
    writer.add_text_box_at_position("Date", 170, 220, 40, 20)?;

    // Middle content
    writer.add_text_box_at_position(
        "Main content area with longer text that demonstrates wrapping in text boxes",
        10,
        250,
        120,
        40,
    )?;

    writer.add_text_box_at_position(
        "Sidebar\n\n• Point 1\n• Point 2\n• Point 3",
        140,
        250,
        70,
        40,
    )?;

    // Bottom row
    writer.add_styled_text_box("Footer Information", "info")?;

    // Final content
    writer.add_heading("Summary", 2)?;
    writer.add_paragraph("Text boxes provide a powerful way to create rich document layouts with positioned content, custom styling, borders, backgrounds, and special effects like rotation and transparency.")?;

    // Add footer with page numbers
    writer.add_footer_with_page_number("Page", hwpers::model::PageNumberFormat::Numeric);

    // Update statistics and save
    writer.update_document_statistics();
    writer.save_to_file("text_box_document.hwp")?;

    println!("Text box demo document created: text_box_document.hwp");
    println!(
        "Document contains {} paragraphs with various text box examples",
        writer.document().body_texts[0].sections[0].paragraphs.len()
    );

    // Print statistics
    if let Some(stats) = writer.get_document_statistics() {
        println!("Document statistics:");
        println!("  - Characters: {}", stats.total_character_count);
        println!("  - Words: {}", stats.total_word_count);
        println!("  - Lines: {}", stats.line_count);
        println!("  - Pages: {}", stats.total_page_count);
    }

    Ok(())
}
