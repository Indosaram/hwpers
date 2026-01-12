use hwpers::hwpx::{HwpxTable, HwpxTextStyle, HwpxWriter};

fn main() {
    generate_styled();
    generate_table();
    generate_hyperlink();
}

fn generate_styled() {
    let mut writer = HwpxWriter::new();
    writer.add_header("스타일 테스트");

    let bold_style = HwpxTextStyle::new().bold().size(16);
    writer
        .add_styled_paragraph("굵은 16pt 텍스트", bold_style)
        .unwrap();

    let red_style = HwpxTextStyle::new().color(0xFF0000);
    writer
        .add_styled_paragraph("빨간색 텍스트", red_style)
        .unwrap();

    writer
        .save_to_file("test-files/verify_styled.hwpx")
        .unwrap();
    println!("Generated: test-files/verify_styled.hwpx");
}

fn generate_table() {
    let mut writer = HwpxWriter::new();
    writer.add_header("테이블 테스트");

    writer.add_paragraph("아래는 테이블입니다:").unwrap();
    let table = HwpxTable::from_data(vec![
        vec!["이름", "직업", "나이"],
        vec!["김철수", "학생", "20"],
        vec!["이영희", "선생님", "30"],
    ]);
    writer.add_table(table).unwrap();

    writer.save_to_file("test-files/verify_table.hwpx").unwrap();
    println!("Generated: test-files/verify_table.hwpx");
}

fn generate_hyperlink() {
    let mut writer = HwpxWriter::new();
    writer.add_header("하이퍼링크 테스트");

    writer
        .add_hyperlink("네이버로 가기", "https://naver.com")
        .unwrap();
    writer
        .add_hyperlink("구글로 가기", "https://google.com")
        .unwrap();

    writer
        .save_to_file("test-files/verify_hyperlink.hwpx")
        .unwrap();
    println!("Generated: test-files/verify_hyperlink.hwpx");
}
