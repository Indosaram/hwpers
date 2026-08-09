use hwpers::writer::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    writer.set_a4_portrait()?;
    writer.add_paragraph("테이블 테스트 문서")?;
    
    writer.add_table(3, 3)
        .set_header_row(true)
        .set_cell(0, 0, "이름")
        .set_cell(0, 1, "나이")
        .set_cell(0, 2, "직업")
        .set_cell(1, 0, "홍길동")
        .set_cell(1, 1, "30")
        .set_cell(1, 2, "개발자")
        .set_cell(2, 0, "김철수")
        .set_cell(2, 1, "25")
        .set_cell(2, 2, "디자이너")
        .finish()?;
    
    writer.add_paragraph("테이블 아래 텍스트")?;
    writer.save_to_file("/tmp/test_table.hwp")?;
    println!("Created /tmp/test_table.hwp");
    Ok(())
}
