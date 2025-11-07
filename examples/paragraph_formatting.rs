use hwpers::writer::style::ParagraphAlignment;
use hwpers::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with paragraph formatting...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("단락 서식 예제", 1)?;

    // Alignment examples
    writer.add_heading("정렬 예제", 2)?;

    writer.add_aligned_paragraph("이 단락은 왼쪽 정렬입니다.", ParagraphAlignment::Left)?;
    writer.add_aligned_paragraph("이 단락은 오른쪽 정렬입니다.", ParagraphAlignment::Right)?;
    writer.add_aligned_paragraph("이 단락은 가운데 정렬입니다.", ParagraphAlignment::Center)?;
    writer.add_aligned_paragraph("이 단락은 양쪽 정렬입니다. 양쪽 정렬은 텍스트를 왼쪽과 오른쪽 여백에 맞추어 정렬하며, 단어 사이의 간격을 조절하여 깔끔한 모양을 만듭니다.", ParagraphAlignment::Justify)?;

    writer.add_paragraph("")?; // Empty line

    // Spacing examples
    writer.add_heading("간격 예제", 2)?;

    writer.add_paragraph("이것은 기본 간격의 단락입니다.")?;

    writer.add_paragraph_with_spacing(
        "이 단락은 150% 줄 간격과 위 10mm, 아래 10mm 간격이 설정되어 있습니다.",
        150,  // 150% line spacing
        10.0, // 10mm before
        10.0, // 10mm after
    )?;

    writer.add_paragraph_with_spacing(
        "이 단락은 200% 줄 간격과 위 5mm, 아래 15mm 간격이 설정되어 있습니다.",
        200,  // 200% line spacing
        5.0,  // 5mm before
        15.0, // 15mm after
    )?;

    writer.add_paragraph_with_spacing(
        "이 단락은 80% 줄 간격으로 조밀하게 설정되어 있습니다.",
        80,  // 80% line spacing
        2.0, // 2mm before
        2.0, // 2mm after
    )?;

    // Combined formatting
    writer.add_heading("복합 서식", 2)?;

    // Center aligned with spacing
    writer.add_aligned_paragraph("[ 공지사항 ]", ParagraphAlignment::Center)?;
    writer.add_paragraph_with_spacing(
        "이 문서는 단락 서식 기능을 시연하기 위한 예제입니다.",
        120, // 120% line spacing
        5.0, // 5mm before
        5.0, // 5mm after
    )?;

    // Long justified text with proper spacing
    writer.add_heading("긴 문장 예제", 2)?;
    writer.add_aligned_paragraph(
        "한글 워드프로세서는 대한민국에서 가장 널리 사용되는 워드프로세서 소프트웨어입니다. \
        1989년 처음 출시된 이후로 꾸준히 발전해 왔으며, 한국어 문서 작성에 최적화된 다양한 \
        기능을 제공합니다. 특히 한국어 타이포그래피와 문서 레이아웃에 대한 깊은 이해를 바탕으로 \
        설계되어, 한국 사용자들에게 매우 친숙한 인터페이스와 기능을 제공합니다.",
        ParagraphAlignment::Justify,
    )?;

    writer.add_paragraph("")?;

    // Different alignments in sequence
    writer.add_aligned_paragraph("첫 번째 줄 - 왼쪽", ParagraphAlignment::Left)?;
    writer.add_aligned_paragraph("두 번째 줄 - 가운데", ParagraphAlignment::Center)?;
    writer.add_aligned_paragraph("세 번째 줄 - 오른쪽", ParagraphAlignment::Right)?;

    // Save to file
    writer.save_to_file("paragraph_formatting.hwp")?;

    println!("✅ Created paragraph_formatting.hwp with various paragraph formatting");
    println!("\nDocument contains:");
    println!("- Different text alignments");
    println!("- Various line spacing options");
    println!("- Paragraph spacing (before/after)");
    println!("- Combined formatting examples");

    Ok(())
}
