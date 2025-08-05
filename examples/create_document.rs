use hwpers::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating new HWP document...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add content
    writer.add_paragraph("안녕하세요! 이것은 hwpers 라이브러리로 만든 HWP 문서입니다.")?;
    writer.add_paragraph("")?; // Empty paragraph for spacing
    writer.add_paragraph("This document was created using the hwpers Rust library.")?;
    writer.add_paragraph("It should open correctly in Hangul word processor.")?;

    // Save to file
    writer.save_to_file("example_document.hwp")?;

    println!("✅ Created example_document.hwp");
    println!("This file can be opened in Hangul word processor.");

    Ok(())
}
