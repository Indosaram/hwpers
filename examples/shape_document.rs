use hwpers::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();

    // Set document metadata
    writer.set_document_title("Shape Drawing Demo")?;

    // Add header
    writer.add_header("Shape Drawing Features Demo");

    // Add title
    writer.add_heading("Shape Drawing Examples", 1)?;

    // Add introduction paragraph
    writer.add_paragraph(
        "Shape drawing features (rectangles, circles, lines, arrows, etc.) are planned for v0.4.0.",
    )?;

    writer.add_paragraph(
        "This example will be updated once the shape API is implemented."
    )?;

    // Save document
    writer.save_to_file("shape_demo.hwp")?;

    println!("Shape drawing features demo (placeholder) saved to shape_demo.hwp");
    println!("Full shape API coming in v0.4.0!");

    Ok(())
}
