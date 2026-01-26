//! Generate test HWP files for manual verification in Hangul program

use hwpers::HwpWriter;
use hwpers::writer::style::{TextStyle, StyledText, ListType};

fn main() {
    // 1. 기본 문서 생성
    let mut writer = HwpWriter::new();

    // 제목
    let title = StyledText::new("hwpers Writer 테스트 문서".to_string())
        .add_range(0, 20, TextStyle::new().bold());
    writer.add_styled_paragraph(&title).unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 일반 텍스트
    writer.add_paragraph("이 문서는 hwpers 라이브러리로 생성되었습니다.").unwrap();
    writer.add_paragraph("한글 프로그램에서 정상적으로 열리는지 테스트합니다.").unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 스타일 텍스트 섹션
    let section_title = StyledText::new("1. 텍스트 스타일 테스트".to_string())
        .add_range(0, 14, TextStyle::new().bold());
    writer.add_styled_paragraph(&section_title).unwrap();

    // 굵은 텍스트
    let bold_text = StyledText::new("굵은 글씨 테스트입니다.".to_string())
        .add_range(0, 12, TextStyle::new().bold());
    writer.add_styled_paragraph(&bold_text).unwrap();

    // 기울임 텍스트
    let italic_text = StyledText::new("기울임 글씨 테스트입니다.".to_string())
        .add_range(0, 13, TextStyle::new().italic());
    writer.add_styled_paragraph(&italic_text).unwrap();

    // 밑줄 텍스트
    let underline_text = StyledText::new("밑줄 글씨 테스트입니다.".to_string())
        .add_range(0, 12, TextStyle::new().underline());
    writer.add_styled_paragraph(&underline_text).unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 테이블 섹션
    let table_title = StyledText::new("2. 테이블 테스트".to_string())
        .add_range(0, 10, TextStyle::new().bold());
    writer.add_styled_paragraph(&table_title).unwrap();

    writer.add_table(3, 3)
        .set_cell(0, 0, "번호")
        .set_cell(0, 1, "이름")
        .set_cell(0, 2, "설명")
        .set_cell(1, 0, "1")
        .set_cell(1, 1, "항목 A")
        .set_cell(1, 2, "첫 번째 항목입니다")
        .set_cell(2, 0, "2")
        .set_cell(2, 1, "항목 B")
        .set_cell(2, 2, "두 번째 항목입니다")
        .finish()
        .unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 하이퍼링크 섹션
    let link_title = StyledText::new("3. 하이퍼링크 테스트".to_string())
        .add_range(0, 12, TextStyle::new().bold());
    writer.add_styled_paragraph(&link_title).unwrap();

    writer.add_hyperlink("Anthropic 홈페이지", "https://www.anthropic.com").unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 리스트 섹션
    let list_title = StyledText::new("4. 리스트 테스트".to_string())
        .add_range(0, 10, TextStyle::new().bold());
    writer.add_styled_paragraph(&list_title).unwrap();

    writer.add_list(&[
        "첫 번째 불릿 항목",
        "두 번째 불릿 항목",
        "세 번째 불릿 항목",
    ], ListType::Bullet).unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    writer.add_list(&[
        "첫 번째 번호 항목",
        "두 번째 번호 항목",
        "세 번째 번호 항목",
    ], ListType::Numbered).unwrap();

    // 빈 줄
    writer.add_paragraph("").unwrap();

    // 마무리
    writer.add_paragraph("--- 문서 끝 ---").unwrap();

    // 파일 저장
    let output_path = "test_output.hwp";
    writer.save_to_file(output_path).unwrap();
    println!("✓ 테스트 HWP 파일 생성 완료: {}", output_path);
    println!("  한글 프로그램에서 열어서 확인해 주세요.");
}
