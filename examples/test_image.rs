use hwpers::writer::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    writer.set_a4_portrait()?;
    
    writer.add_paragraph("이미지 테스트 문서")?;
    writer.add_image("/tmp/test_image.png")?;
    writer.add_paragraph("위에 이미지가 표시됩니다.")?;
    
    writer.save_to_file("/tmp/test_image.hwp")?;
    println!("Created /tmp/test_image.hwp");
    Ok(())
}
