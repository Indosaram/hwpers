use hwpers::HwpWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with hyperlinks...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("하이퍼링크 예제 문서", 1)?;

    // Website links
    writer.add_heading("웹사이트 링크", 2)?;
    writer.add_paragraph("다음은 다양한 웹사이트 링크입니다:")?;

    writer.add_hyperlink("구글 방문하기", "https://www.google.com")?;
    writer.add_paragraph("")?; // Empty line

    writer.add_hyperlink("GitHub 저장소", "https://github.com/indo/hwpers")?;
    writer.add_paragraph("")?;

    writer.add_hyperlink("Rust 공식 사이트", "https://www.rust-lang.org")?;
    writer.add_paragraph("")?;

    // Email links
    writer.add_heading("이메일 링크", 2)?;
    writer.add_paragraph("이메일 주소를 클릭하여 메일을 보낼 수 있습니다:")?;

    writer.add_email_link("문의하기", "contact@example.com")?;
    writer.add_paragraph("")?;

    writer.add_email_link("기술 지원", "support@example.com")?;
    writer.add_paragraph("")?;

    // File links
    writer.add_heading("파일 링크", 2)?;
    writer.add_paragraph("로컬 파일이나 네트워크 파일을 열 수 있습니다:")?;

    writer.add_file_link("README 파일", "./README.md")?;
    writer.add_paragraph("")?;

    writer.add_file_link("설정 파일", "C:\\Program Files\\MyApp\\config.ini")?;
    writer.add_paragraph("")?;

    writer.add_file_link("네트워크 폴더", "\\\\server\\shared\\documents")?;
    writer.add_paragraph("")?;

    // Mixed content
    writer.add_heading("혼합 콘텐츠", 2)?;
    writer.add_paragraph("텍스트와 하이퍼링크를 함께 사용할 수 있습니다.")?;
    writer.add_paragraph("")?;

    writer.add_paragraph("자세한 정보는")?;
    writer.add_hyperlink("여기", "https://example.com/more-info")?;
    writer.add_paragraph("를 참조하세요.")?;

    writer.add_paragraph("")?;
    writer.add_paragraph("질문이 있으시면")?;
    writer.add_email_link("이메일", "help@example.com")?;
    writer.add_paragraph("로 연락주세요.")?;

    // Custom styled links
    writer.add_heading("다양한 링크 스타일", 2)?;

    writer.add_paragraph("사용자 정의 스타일링된 링크들:")?;

    // Create custom hyperlink with different colors
    let custom_hyperlink =
        hwpers::model::hyperlink::Hyperlink::new_url("특별한 링크", "https://special.example.com")
            .with_text_color(0xFF0000) // Red
            .with_underline(true)
            .with_tooltip("이것은 빨간색 링크입니다");

    writer.add_hyperlink_with_options(custom_hyperlink)?;
    writer.add_paragraph("")?;

    // Green file link
    let green_link =
        hwpers::model::hyperlink::Hyperlink::new_file("중요한 문서", "./important.pdf")
            .with_text_color(0x008000) // Green
            .with_underline(true)
            .with_tooltip("중요한 PDF 문서");

    writer.add_hyperlink_with_options(green_link)?;
    writer.add_paragraph("")?;

    // Save to file
    writer.save_to_file("hyperlink_document.hwp")?;

    println!("✅ Created hyperlink_document.hwp with various hyperlink types");
    println!("\\nDocument contains:");
    println!("- Website links");
    println!("- Email links");
    println!("- File links");
    println!("- Mixed content with links");
    println!("- Custom styled links");

    Ok(())
}
