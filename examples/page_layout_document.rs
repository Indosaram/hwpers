use hwpers::{writer::style::TextStyle, HwpWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with various page layouts...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("페이지 레이아웃 예제 문서", 1)?;

    // Introduction
    writer.add_heading("소개", 2)?;
    writer.add_paragraph("이 문서는 다양한 페이지 레이아웃 기능을 보여줍니다.")?;
    writer.add_paragraph("")?;

    // Example 1: Default A4 Portrait
    writer.add_heading("1. 기본 A4 세로 방향", 2)?;
    writer.add_paragraph("문서가 기본 A4 세로 방향으로 설정되어 있습니다.")?;

    // Set A4 portrait explicitly
    writer.set_a4_portrait()?;
    writer.add_paragraph("A4 세로 방향이 명시적으로 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Example 2: A4 Landscape
    writer.add_heading("2. A4 가로 방향", 2)?;
    writer.add_paragraph("이제 페이지를 A4 가로 방향으로 변경합니다:")?;

    writer.set_a4_landscape()?;
    writer.add_paragraph("A4 가로 방향으로 설정되었습니다. 페이지가 더 넓어졌습니다.")?;
    writer.add_paragraph("")?;

    // Example 3: Letter Portrait
    writer.add_heading("3. Letter 용지 세로 방향", 2)?;
    writer.add_paragraph("미국 표준 Letter 용지 크기로 변경합니다:")?;

    writer.set_letter_portrait()?;
    writer.add_paragraph("Letter 세로 방향으로 설정되었습니다 (8.5 × 11 인치).")?;
    writer.add_paragraph("")?;

    // Example 4: Letter Landscape
    writer.add_heading("4. Letter 용지 가로 방향", 2)?;
    writer.add_paragraph("Letter 용지를 가로 방향으로 설정합니다:")?;

    writer.set_letter_landscape()?;
    writer.add_paragraph("Letter 가로 방향으로 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Example 5: Custom Page Size
    writer.add_heading("5. 사용자 정의 용지 크기", 2)?;
    writer.add_paragraph("사용자 정의 크기로 설정합니다 (300mm × 400mm):")?;

    use hwpers::model::page_layout::PageOrientation;
    writer.set_custom_page_size(300.0, 400.0, PageOrientation::Portrait)?;
    writer.add_paragraph("사용자 정의 크기 (300×400mm)로 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Example 6: Different Margins
    writer.add_heading("6. 여백 설정", 2)?;
    writer.add_paragraph("다양한 여백을 설정해 보겠습니다.")?;

    // Narrow margins
    writer.add_heading("좁은 여백", 3)?;
    writer.set_narrow_margins();
    writer.add_paragraph(
        "좁은 여백 (12.7mm)으로 설정되었습니다. 더 많은 내용을 페이지에 넣을 수 있습니다.",
    )?;
    writer.add_paragraph("")?;

    // Normal margins
    writer.add_heading("보통 여백", 3)?;
    writer.set_normal_margins();
    writer.add_paragraph("보통 여백 (25.4mm)으로 설정되었습니다. 표준적인 문서 여백입니다.")?;
    writer.add_paragraph("")?;

    // Wide margins
    writer.add_heading("넓은 여백", 3)?;
    writer.set_wide_margins();
    writer.add_paragraph(
        "넓은 여백으로 설정되었습니다. 좌우 50.8mm, 상하 25.4mm의 여백을 가집니다.",
    )?;
    writer.add_paragraph("")?;

    // Custom margins
    writer.add_heading("사용자 정의 여백", 3)?;
    writer.set_page_margins_mm(20.0, 15.0, 25.0, 10.0);
    writer.add_paragraph(
        "사용자 정의 여백이 설정되었습니다: 왼쪽 20mm, 오른쪽 15mm, 위쪽 25mm, 아래쪽 10mm",
    )?;
    writer.add_paragraph("")?;

    // Example 7: Multiple Columns
    writer.add_heading("7. 다단 설정", 2)?;
    writer.add_paragraph("문서를 여러 단으로 나누어 신문이나 잡지 스타일로 만들 수 있습니다.")?;

    // 2 columns
    writer.add_heading("2단 레이아웃", 3)?;
    writer.set_columns(2, 5.0);
    writer.add_paragraph("2단 레이아웃이 설정되었습니다. 단 사이 간격은 5mm입니다.")?;
    writer.add_paragraph("이 텍스트는 2단으로 나뉘어 표시됩니다. 긴 텍스트를 추가하여 단 나누기 효과를 확인할 수 있습니다. Lorem ipsum dolor sit amet, consectetur adipiscing elit.")?;
    writer.add_paragraph("")?;

    // 3 columns
    writer.add_heading("3단 레이아웃", 3)?;
    writer.set_columns(3, 4.0);
    writer.add_paragraph("3단 레이아웃이 설정되었습니다. 단 사이 간격은 4mm입니다.")?;
    writer.add_paragraph(
        "이제 텍스트가 3개의 좁은 단으로 나뉩니다. 이는 뉴스레터나 브로셔에 적합한 레이아웃입니다.",
    )?;
    writer.add_paragraph("")?;

    // Example 8: Background Colors
    writer.add_heading("8. 페이지 배경색", 2)?;
    writer.add_paragraph("페이지에 배경색을 설정할 수 있습니다.")?;

    // Light blue background
    writer.add_heading("연한 파란색 배경", 3)?;
    writer.set_page_background_color(0xF0F8FF); // Alice Blue
    writer.add_paragraph("연한 파란색 배경 (Alice Blue)이 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Light yellow background
    writer.add_heading("연한 노란색 배경", 3)?;
    writer.set_page_background_color(0xFFFFF0); // Ivory
    writer.add_paragraph("연한 노란색 배경 (Ivory)이 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Example 9: Page Numbering
    writer.add_heading("9. 페이지 번호 매기기", 2)?;
    writer.add_paragraph("페이지 번호 매기기 기능을 설정할 수 있습니다.")?;

    use hwpers::model::header_footer::PageNumberFormat;
    writer.set_page_numbering(1, PageNumberFormat::Numeric)?;
    writer.add_paragraph("숫자 형식의 페이지 번호가 1부터 시작하도록 설정되었습니다.")?;
    writer.add_paragraph("")?;

    // Example 10: Complex Layout
    writer.add_heading("10. 복합 레이아웃", 2)?;
    writer.add_paragraph("여러 설정을 조합하여 복잡한 레이아웃을 만들 수 있습니다.")?;

    // Create a complex layout using the individual methods
    writer.set_a4_portrait()?;
    writer.set_page_margins_mm(30.0, 20.0, 25.0, 15.0);
    writer.set_columns(2, 8.0);
    writer.set_page_background_color(0xFAFAFA); // Very light gray
    writer.set_page_numbering(1, PageNumberFormat::RomanLower)?;

    writer.add_paragraph("복합 레이아웃이 적용되었습니다:")?;
    writer.add_paragraph("• A4 세로 방향")?;
    writer.add_paragraph("• 사용자 정의 여백 (좌30mm, 우20mm, 상25mm, 하15mm)")?;
    writer.add_paragraph("• 2단 레이아웃 (8mm 간격)")?;
    writer.add_paragraph("• 연한 회색 배경")?;
    writer.add_paragraph("• 소문자 로마 숫자 페이지 번호")?;
    writer.add_paragraph("")?;

    // Example 11: Mixed Content with Layout
    writer.add_heading("11. 레이아웃과 함께하는 다양한 콘텐츠", 2)?;
    writer.add_paragraph("설정된 레이아웃 내에서 다양한 콘텐츠를 추가해 보겠습니다.")?;

    // Add a table
    writer.add_simple_table(&[
        vec!["항목", "설명"],
        vec!["용지 크기", "A4 (210×297mm)"],
        vec!["방향", "세로"],
        vec!["단 수", "2단"],
        vec!["여백", "사용자 정의"],
        vec!["배경", "연한 회색"],
    ])?;

    writer.add_paragraph("")?;

    // Add some styled text
    let bold_style = TextStyle::new().bold();
    writer.add_paragraph_with_style("굵은 글씨로 강조된 텍스트입니다.", &bold_style)?;

    let colored_style = TextStyle::new().color(0xFF0000); // Red
    writer.add_paragraph_with_style("빨간색 텍스트입니다.", &colored_style)?;

    writer.add_paragraph("")?;

    // Add a list
    use hwpers::writer::style::ListType;
    writer.add_list(
        &["첫 번째 항목", "두 번째 항목", "세 번째 항목"],
        ListType::Bullet,
    )?;

    writer.add_paragraph("")?;

    // Conclusion
    writer.add_heading("결론", 2)?;
    writer.add_paragraph("이 문서는 HWP Writer의 다양한 페이지 레이아웃 기능을 보여주었습니다:")?;
    writer.add_paragraph("• 다양한 용지 크기와 방향")?;
    writer.add_paragraph("• 유연한 여백 설정")?;
    writer.add_paragraph("• 다단 레이아웃")?;
    writer.add_paragraph("• 페이지 배경색")?;
    writer.add_paragraph("• 페이지 번호 매기기")?;
    writer.add_paragraph("• 복합 레이아웃 구성")?;

    writer.add_paragraph("")?;
    writer
        .add_paragraph("이러한 기능들을 활용하여 전문적이고 아름다운 문서를 만들 수 있습니다.")?;

    // Save to file
    writer.save_to_file("page_layout_document.hwp")?;

    println!("✅ Created page_layout_document.hwp with various page layout examples");
    println!("\\nDocument contains:");
    println!("- Different paper sizes and orientations");
    println!("- Various margin settings");
    println!("- Multi-column layouts");
    println!("- Page background colors");
    println!("- Page numbering options");
    println!("- Complex combined layouts");

    Ok(())
}
