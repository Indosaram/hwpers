// Demonstrates all the newly implemented features in hwpers writer
use hwpers::writer::{HwpWriter, style::ParagraphAlignment};
use hwpers::model::{
    hyperlink::Hyperlink,
    header_footer::PageNumberFormat,
    page_layout::PageOrientation,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with all new features...\n");

    let mut writer = HwpWriter::new();

    // 1. Page Layout Settings
    println!("Setting up page layout...");
    writer.set_custom_page_size(210.0, 297.0, PageOrientation::Portrait)?; // A4
    writer.set_page_margins_mm(25.0, 25.0, 20.0, 20.0)?;

    // 2. Header and Footer
    println!("Adding header and footer...");
    writer.add_header("HWP Writer Demo - Complete Feature Set");
    writer.add_footer_with_page_number("Page ", PageNumberFormat::Numeric);

    // 3. Title with center alignment
    println!("Adding title...");
    writer.add_aligned_paragraph(
        "한글 문서 작성기 기능 시연",
        ParagraphAlignment::Center
    )?;
    
    writer.add_aligned_paragraph(
        "Complete Feature Demonstration",
        ParagraphAlignment::Center
    )?;

    writer.add_paragraph("")?; // Empty line

    // 4. Section with hyperlinks
    println!("Adding hyperlinks section...");
    writer.add_aligned_paragraph(
        "유용한 링크들",
        ParagraphAlignment::Left
    )?;

    // Multiple hyperlinks in one paragraph
    let rust_link = Hyperlink::new_url("Rust 홈페이지", "https://rust-lang.org");
    let github_link = Hyperlink::new_url("GitHub 저장소", "https://github.com");
    let email_link = Hyperlink::new_email("이메일 문의", "contact@example.com");
    
    writer.add_paragraph_with_hyperlinks(
        "프로그래밍 리소스: Rust 홈페이지, GitHub 저장소, 이메일 문의",
        vec![rust_link, github_link, email_link]
    )?;

    writer.add_paragraph("")?; // Empty line

    // 5. Text with various alignments
    println!("Adding text with various alignments...");
    writer.add_aligned_paragraph(
        "왼쪽 정렬된 텍스트입니다.",
        ParagraphAlignment::Left
    )?;

    writer.add_aligned_paragraph(
        "가운데 정렬된 텍스트입니다.",
        ParagraphAlignment::Center
    )?;

    writer.add_aligned_paragraph(
        "오른쪽 정렬된 텍스트입니다.",
        ParagraphAlignment::Right
    )?;

    writer.add_aligned_paragraph(
        "양쪽 정렬된 텍스트입니다. 이 문단은 양쪽 끝이 맞춰지도록 정렬됩니다. 긴 텍스트일수록 양쪽 정렬의 효과가 잘 보입니다.",
        ParagraphAlignment::Justify
    )?;

    writer.add_paragraph("")?; // Empty line

    // 6. Text with custom spacing
    println!("Adding text with custom spacing...");
    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격이 200%이고, 문단 앞뒤 여백이 각각 15mm입니다.",
        200,  // 200% line spacing
        15.0, // 15mm before
        15.0  // 15mm after
    )?;

    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격이 80%로 좁고, 문단 앞뒤 여백이 5mm입니다.",
        80,   // 80% line spacing
        5.0,  // 5mm before
        5.0   // 5mm after
    )?;

    // 7. Bookmarks and internal links
    println!("Adding bookmarks and internal links...");
    writer.add_paragraph("")?; // Empty line
    
    writer.add_aligned_paragraph(
        "문서 내부 링크",
        ParagraphAlignment::Left
    )?;

    let bookmark_link = Hyperlink::new_bookmark("섹션 1로 이동", "section1");
    writer.add_paragraph_with_hyperlinks(
        "이 링크를 클릭하면 섹션 1로 이동합니다: 섹션 1로 이동",
        vec![bookmark_link]
    )?;

    // 8. Mixed content with various styles
    println!("Adding mixed content...");
    writer.add_paragraph("")?; // Empty line

    writer.add_aligned_paragraph(
        "복합 스타일 예제",
        ParagraphAlignment::Center
    )?;

    writer.add_paragraph_with_spacing(
        "이 문서는 hwpers 라이브러리의 새로운 기능들을 시연합니다. 하이퍼링크, 머리글/바닥글, 페이지 설정, 문단 서식 등이 포함되어 있습니다.",
        120,  // 120% line spacing
        10.0, // 10mm before
        10.0  // 10mm after
    )?;

    // 9. Footer information
    writer.add_paragraph("")?; // Empty line
    writer.add_paragraph("")?; // Empty line
    
    writer.add_aligned_paragraph(
        "문서 정보",
        ParagraphAlignment::Left
    )?;

    writer.add_paragraph("작성일: 2025년 1월")?;
    writer.add_paragraph("작성자: hwpers 라이브러리")?;
    writer.add_paragraph("버전: 0.2.0")?;

    // Save the document
    let output_path = "complete_feature_demo.hwp";
    writer.save_to_file(output_path)?;

    println!("\nDocument created successfully: {}", output_path);
    println!("Open this file in 한글 (Hanword) to see all the features!");

    // Also create individual example files for each feature
    create_hyperlink_example()?;
    create_header_footer_example()?;
    create_alignment_example()?;
    create_spacing_example()?;

    Ok(())
}

fn create_hyperlink_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    
    writer.add_aligned_paragraph("하이퍼링크 예제", ParagraphAlignment::Center)?;
    writer.add_paragraph("")?;
    
    let url_link = Hyperlink::new_url("Rust 공식 웹사이트", "https://rust-lang.org");
    let email_link = Hyperlink::new_email("이메일 보내기", "example@email.com");
    let file_link = Hyperlink::new_file("문서 열기", "./document.pdf");
    
    writer.add_paragraph_with_hyperlinks(
        "웹 링크: Rust 공식 웹사이트",
        vec![url_link]
    )?;
    
    writer.add_paragraph_with_hyperlinks(
        "이메일 링크: 이메일 보내기",
        vec![email_link]
    )?;
    
    writer.add_paragraph_with_hyperlinks(
        "파일 링크: 문서 열기",
        vec![file_link]
    )?;
    
    writer.save_to_file("hyperlink_example.hwp")?;
    println!("Created: hyperlink_example.hwp");
    Ok(())
}

fn create_header_footer_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    
    writer.add_header("문서 제목 - 머리글");
    writer.add_footer_with_page_number("- ", PageNumberFormat::Numeric);
    
    writer.add_aligned_paragraph("머리글/바닥글 예제", ParagraphAlignment::Center)?;
    writer.add_paragraph("")?;
    writer.add_paragraph("이 문서는 머리글과 바닥글을 포함합니다.")?;
    writer.add_paragraph("바닥글에는 페이지 번호가 표시됩니다.")?;
    
    writer.save_to_file("header_footer_example.hwp")?;
    println!("Created: header_footer_example.hwp");
    Ok(())
}

fn create_alignment_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    
    writer.add_aligned_paragraph("정렬 예제", ParagraphAlignment::Center)?;
    writer.add_paragraph("")?;
    
    writer.add_aligned_paragraph(
        "이 텍스트는 왼쪽으로 정렬되어 있습니다.",
        ParagraphAlignment::Left
    )?;
    
    writer.add_aligned_paragraph(
        "이 텍스트는 가운데로 정렬되어 있습니다.",
        ParagraphAlignment::Center
    )?;
    
    writer.add_aligned_paragraph(
        "이 텍스트는 오른쪽으로 정렬되어 있습니다.",
        ParagraphAlignment::Right
    )?;
    
    writer.add_aligned_paragraph(
        "이 텍스트는 양쪽으로 정렬되어 있습니다. 긴 문장일수록 양쪽 정렬의 효과를 더 잘 볼 수 있습니다. 양쪽 정렬은 문서를 더 깔끔하게 만들어 줍니다.",
        ParagraphAlignment::Justify
    )?;
    
    writer.save_to_file("alignment_example.hwp")?;
    println!("Created: alignment_example.hwp");
    Ok(())
}

fn create_spacing_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();
    
    writer.add_aligned_paragraph("간격 조절 예제", ParagraphAlignment::Center)?;
    writer.add_paragraph("")?;
    
    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격 50%입니다. 매우 좁은 간격입니다.",
        50, 0.0, 0.0
    )?;
    
    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격 100%입니다. 기본 간격입니다.",
        100, 5.0, 5.0
    )?;
    
    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격 150%입니다. 약간 넓은 간격입니다.",
        150, 10.0, 10.0
    )?;
    
    writer.add_paragraph_with_spacing(
        "이 문단은 줄 간격 200%입니다. 매우 넓은 간격입니다.\n또한 문단 앞뒤로 20mm의 여백이 있습니다.",
        200, 20.0, 20.0
    )?;
    
    writer.save_to_file("spacing_example.hwp")?;
    println!("Created: spacing_example.hwp");
    Ok(())
}