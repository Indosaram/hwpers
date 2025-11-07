use hwpers::writer::style::{StyledText, TextStyle};
use hwpers::{error::Result, HwpWriter};

fn main() -> Result<()> {
    let mut writer = HwpWriter::new();

    // Add title with basic styling
    writer.add_heading("한글 텍스트 서식 예제", 1)?;

    // Add paragraph with simple bold text
    writer.add_paragraph_with_bold(
        "이 문장에서 굵게 처리된 텍스트가 있습니다.",
        vec![(9, 15)], // "굵게 처리된" 부분을 굵게
    )?;

    // Add paragraph with multiple colors
    writer.add_paragraph_with_colors(
        "빨간색 텍스트와 파란색 텍스트가 포함된 문장입니다.",
        vec![
            (0, 5, 0xFF0000),   // "빨간색" - 빨간색
            (11, 16, 0x0000FF), // "파란색" - 파란색
        ],
    )?;

    // Add paragraph with highlighting
    writer.add_paragraph_with_highlight(
        "이 문장에는 노란색 하이라이트가 적용됩니다.",
        vec![(9, 14, 0xFFFF00)], // "노란색 하이라이트" 부분을 노란색으로 하이라이트
    )?;

    // Add complex styled text using StyledText builder
    let styled_text = StyledText::new("복잡한 서식이 적용된 텍스트입니다.".to_string())
        .add_range(0, 3, TextStyle::new().bold().color(0xFF0000)) // "복잡한" - 빨간색 굵게
        .add_range(4, 6, TextStyle::new().italic().color(0x00FF00)) // "서식이" - 녹색 기울임
        .add_range(7, 9, TextStyle::new().underline().color(0x0000FF)) // "적용된" - 파란색 밑줄
        .add_range(10, 13, TextStyle::new().size(16).background(0xFFFF00)); // "텍스트입니다" - 16pt 노란 배경

    writer.add_styled_paragraph(&styled_text)?;

    // Add paragraph with font changes
    let font_text = StyledText::new("다양한 폰트가 사용된 문장입니다.".to_string())
        .add_range(0, 3, TextStyle::new().font("Arial").size(14)) // "다양한" - Arial 14pt
        .add_range(4, 6, TextStyle::new().font("Times New Roman").size(12)) // "폰트가" - Times 12pt
        .add_range(7, 9, TextStyle::new().font("Courier New").size(10)); // "사용된" - Courier 10pt

    writer.add_styled_paragraph(&font_text)?;

    // Add mixed styling example using convenience method
    writer.add_mixed_text(
        "이 예제는 다양한 스타일이 조합된 텍스트를 보여줍니다.",
        vec![
            (0, 2, TextStyle::new().bold()), // "이 예제는" - 굵게
            (8, 11, TextStyle::new().italic().color(0x800080)), // "다양한" - 보라색 기울임
            (12, 15, TextStyle::new().underline().strikethrough()), // "스타일이" - 밑줄+취소선
            (16, 18, TextStyle::new().size(18).background(0xFFE4B5)), // "조합된" - 18pt 주황 배경
            (19, 22, TextStyle::new().font("맑은 고딕").color(0x008080)), // "텍스트를" - 청록색
        ],
    )?;

    // Save the document
    writer.save_to_file("styled_text_document.hwp")?;
    println!("Text range styling document saved as 'styled_text_document.hwp'");

    Ok(())
}
