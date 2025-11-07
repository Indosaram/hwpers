use hwpers::{writer::style::TextStyle, HwpWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with styled text...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add a large heading
    writer.add_heading("스타일 문서 예제", 1)?;

    // Add a normal paragraph
    writer.add_paragraph("이것은 일반 단락입니다. 기본 스타일로 작성되었습니다.")?;
    writer.add_paragraph("")?; // Empty line

    // Add a medium heading
    writer.add_heading("텍스트 스타일링", 2)?;

    // Add styled paragraphs
    let bold_style = TextStyle::new().bold();
    writer.add_paragraph_with_style("이것은 굵은 글씨입니다.", &bold_style)?;

    let italic_style = TextStyle::new().italic();
    writer.add_paragraph_with_style("이것은 기울임 글씨입니다.", &italic_style)?;

    let underline_style = TextStyle::new().underline();
    writer.add_paragraph_with_style("이것은 밑줄이 있는 글씨입니다.", &underline_style)?;

    writer.add_paragraph("")?; // Empty line

    // Add a smaller heading
    writer.add_heading("폰트 크기와 색상", 3)?;

    // Large text
    let large_style = TextStyle::new().size(18);
    writer.add_paragraph_with_style("큰 글씨 (18pt)", &large_style)?;

    // Small text
    let small_style = TextStyle::new().size(9);
    writer.add_paragraph_with_style("작은 글씨 (9pt)", &small_style)?;

    // Colored text
    let red_style = TextStyle::new().color(0xFF0000); // Red
    writer.add_paragraph_with_style("빨간색 글씨", &red_style)?;

    let blue_style = TextStyle::new().color(0x0000FF); // Blue
    writer.add_paragraph_with_style("파란색 글씨", &blue_style)?;

    writer.add_paragraph("")?; // Empty line

    // Combined styles
    writer.add_heading("복합 스타일", 3)?;

    let complex_style = TextStyle::new().size(14).bold().italic().color(0x008000); // Green
    writer.add_paragraph_with_style("굵고 기울어진 14pt 녹색 글씨", &complex_style)?;

    // Different fonts
    writer.add_heading("폰트 변경", 3)?;

    let gulim_style = TextStyle::new().font("굴림");
    writer.add_paragraph_with_style("굴림체로 작성된 텍스트입니다.", &gulim_style)?;

    let batang_style = TextStyle::new().font("바탕");
    writer.add_paragraph_with_style("바탕체로 작성된 텍스트입니다.", &batang_style)?;

    // Save to file
    writer.save_to_file("styled_document.hwp")?;

    println!("✅ Created styled_document.hwp with various text styles");
    println!("\nDocument contains:");
    println!("- Different heading levels");
    println!("- Bold, italic, and underlined text");
    println!("- Various font sizes");
    println!("- Colored text");
    println!("- Different fonts");
    println!("- Combined styles");

    Ok(())
}
