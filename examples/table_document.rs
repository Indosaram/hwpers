use hwpers::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with tables...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("표 예제 문서", 1)?;

    // Simple table using add_simple_table
    writer.add_heading("간단한 표", 2)?;
    writer.add_simple_table(&[
        vec!["이름", "나이", "직업"],
        vec!["김철수", "30", "개발자"],
        vec!["이영희", "28", "디자이너"],
        vec!["박민수", "35", "매니저"],
    ])?;

    writer.add_paragraph("")?; // Empty line

    // Table with header using TableBuilder
    writer.add_heading("헤더가 있는 표", 2)?;
    writer
        .add_table(4, 3)?
        .set_header_row(true)
        .set_cell(0, 0, "제품명")
        .set_cell(0, 1, "가격")
        .set_cell(0, 2, "재고")
        .set_cell(1, 0, "노트북")
        .set_cell(1, 1, "1,200,000원")
        .set_cell(1, 2, "15개")
        .set_cell(2, 0, "마우스")
        .set_cell(2, 1, "30,000원")
        .set_cell(2, 2, "50개")
        .set_cell(3, 0, "키보드")
        .set_cell(3, 1, "80,000원")
        .set_cell(3, 2, "30개")
        .finish()?;

    writer.add_paragraph("")?;

    // Larger table
    writer.add_heading("성적표", 2)?;
    writer
        .add_table(6, 5)?
        .set_header_row(true)
        .set_cell(0, 0, "이름")
        .set_cell(0, 1, "국어")
        .set_cell(0, 2, "영어")
        .set_cell(0, 3, "수학")
        .set_cell(0, 4, "평균")
        .set_cell(1, 0, "학생1")
        .set_cell(1, 1, "90")
        .set_cell(1, 2, "85")
        .set_cell(1, 3, "88")
        .set_cell(1, 4, "87.7")
        .set_cell(2, 0, "학생2")
        .set_cell(2, 1, "88")
        .set_cell(2, 2, "92")
        .set_cell(2, 3, "90")
        .set_cell(2, 4, "90.0")
        .set_cell(3, 0, "학생3")
        .set_cell(3, 1, "78")
        .set_cell(3, 2, "80")
        .set_cell(3, 3, "85")
        .set_cell(3, 4, "81.0")
        .set_cell(4, 0, "학생4")
        .set_cell(4, 1, "95")
        .set_cell(4, 2, "88")
        .set_cell(4, 3, "92")
        .set_cell(4, 4, "91.7")
        .set_cell(5, 0, "평균")
        .set_cell(5, 1, "87.8")
        .set_cell(5, 2, "86.3")
        .set_cell(5, 3, "88.8")
        .set_cell(5, 4, "87.6")
        .finish()?;

    writer.add_paragraph("")?;

    // Mixed content with table
    writer.add_heading("표와 텍스트 혼합", 2)?;
    writer.add_paragraph("다음은 프로젝트 일정표입니다:")?;

    writer
        .add_table(5, 3)?
        .set_header_row(true)
        .set_cell(0, 0, "작업")
        .set_cell(0, 1, "시작일")
        .set_cell(0, 2, "종료일")
        .set_cell(1, 0, "기획")
        .set_cell(1, 1, "2024-01-01")
        .set_cell(1, 2, "2024-01-15")
        .set_cell(2, 0, "디자인")
        .set_cell(2, 1, "2024-01-10")
        .set_cell(2, 2, "2024-02-01")
        .set_cell(3, 0, "개발")
        .set_cell(3, 1, "2024-01-20")
        .set_cell(3, 2, "2024-03-15")
        .set_cell(4, 0, "테스트")
        .set_cell(4, 1, "2024-03-01")
        .set_cell(4, 2, "2024-03-31")
        .finish()?;

    writer.add_paragraph("위 일정에 따라 프로젝트를 진행할 예정입니다.")?;

    // Save to file
    writer.save_to_file("table_document.hwp")?;

    println!("✅ Created table_document.hwp with various tables");
    println!("\nDocument contains:");
    println!("- Simple tables");
    println!("- Tables with headers");
    println!("- Large tables");
    println!("- Tables mixed with text");

    Ok(())
}
