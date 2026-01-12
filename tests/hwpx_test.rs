use hwpers::hwpx::writer::{HwpxHyperlink, HwpxImage, HwpxTable, HwpxTextStyle, StyledText};
use hwpers::{HwpxReader, HwpxWriter};
use std::path::PathBuf;

fn test_file_path(name: &str) -> PathBuf {
    PathBuf::from("test-files").join(name)
}

#[test]
fn test_read_hwpx_file() {
    let path = test_file_path("test_sample.hwpx");
    if !path.exists() {
        println!("Test file not found: {:?}", path);
        return;
    }

    let result = HwpxReader::from_file(&path);
    match &result {
        Ok(doc) => {
            println!("Successfully parsed HWPX file");
            println!("Version: {}", doc.header.version_string());

            let text = doc.extract_text();
            println!("Extracted text length: {} chars", text.len());
            println!("Text preview: {}", &text[..text.len().min(200)]);

            assert!(!text.is_empty(), "Document should have text content");
        }
        Err(e) => {
            panic!("Failed to parse HWPX file: {:?}", e);
        }
    }
}

#[test]
fn test_hwpx_from_bytes() {
    let path = test_file_path("test_sample.hwpx");
    if !path.exists() {
        return;
    }

    let bytes = std::fs::read(&path).expect("Failed to read file");
    let result = HwpxReader::from_bytes(&bytes);

    assert!(result.is_ok(), "Should parse HWPX from bytes");
}

#[test]
fn test_hwpx_writer_basic() {
    let mut writer = HwpxWriter::new();
    writer.add_paragraph("Hello HWPX").unwrap();
    writer.add_paragraph("안녕하세요").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04], "ZIP signature");
}

#[test]
fn test_hwpx_roundtrip() {
    let mut writer = HwpxWriter::new();
    writer.add_paragraph("Roundtrip Test").unwrap();
    writer.add_paragraph("라운드트립 테스트").unwrap();
    writer.add_paragraph("Third paragraph").unwrap();

    let bytes = writer.to_bytes().unwrap();

    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read written HWPX");
    let text = document.extract_text();

    assert!(
        text.contains("Roundtrip Test"),
        "Should contain first paragraph"
    );
    assert!(
        text.contains("라운드트립 테스트"),
        "Should contain Korean text"
    );
    assert!(
        text.contains("Third paragraph"),
        "Should contain third paragraph"
    );
}

#[test]
fn test_hwpx_save_to_file() {
    let mut writer = HwpxWriter::new();
    writer.add_paragraph("Test Document").unwrap();
    writer.add_paragraph("문서 테스트").unwrap();

    let output_path = test_file_path("test_output.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists(), "Output file should exist");

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();
    assert!(text.contains("Test Document"));
    assert!(text.contains("문서 테스트"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_hwpx_from_document() {
    let path = test_file_path("test_sample.hwpx");
    if !path.exists() {
        return;
    }

    let original = HwpxReader::from_file(&path).expect("Failed to read original");
    let original_text = original.extract_text();

    let writer = HwpxWriter::from_document(original);
    let bytes = writer.to_bytes().unwrap();

    let roundtrip = HwpxReader::from_bytes(&bytes).expect("Failed to read roundtrip");
    let roundtrip_text = roundtrip.extract_text();

    assert_eq!(
        original_text.trim(),
        roundtrip_text.trim(),
        "Text should be preserved after roundtrip"
    );
}

#[test]
fn test_hwpx_empty_document() {
    let writer = HwpxWriter::new();
    let bytes = writer.to_bytes().unwrap();

    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read empty document");
    let text = document.extract_text();
    assert!(text.is_empty() || text.trim().is_empty());
}

#[test]
fn test_hwpx_special_characters() {
    let mut writer = HwpxWriter::new();
    writer.add_paragraph("Special chars: <>&\"'").unwrap();
    writer.add_paragraph("Unicode: 한글 日本語 中文").unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    assert!(text.contains("<"), "Should preserve <");
    assert!(text.contains(">"), "Should preserve >");
    assert!(text.contains("&"), "Should preserve &");
    assert!(text.contains("한글"), "Should preserve Korean");
    assert!(text.contains("日本語"), "Should preserve Japanese");
}

#[test]
fn test_hwpx_styled_paragraph() {
    let mut writer = HwpxWriter::new();

    let bold_style = HwpxTextStyle::new().bold().size(14);
    writer
        .add_styled_paragraph("Bold Title", bold_style)
        .unwrap();

    let red_style = HwpxTextStyle::new().color(0xFF0000);
    writer.add_styled_paragraph("Red Text", red_style).unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    assert!(text.contains("Bold Title"), "Should contain bold title");
    assert!(text.contains("Red Text"), "Should contain red text");
}

#[test]
fn test_hwpx_mixed_styled_paragraph() {
    let mut writer = HwpxWriter::new();

    let runs = vec![
        StyledText::new("Normal text, "),
        StyledText::with_style("bold", HwpxTextStyle::new().bold()),
        StyledText::new(" and "),
        StyledText::with_style("italic", HwpxTextStyle::new().italic()),
        StyledText::new(" mixed."),
    ];
    writer.add_mixed_styled_paragraph(runs).unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    println!("Extracted text: {:?}", text);

    assert!(text.contains("Normal text"));
    assert!(text.contains("bold"));
    assert!(text.contains("italic"));
    assert!(text.contains("mixed"));
}

#[test]
fn test_hwpx_styled_text_roundtrip() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("Plain paragraph").unwrap();
    writer
        .add_styled_paragraph(
            "Styled paragraph",
            HwpxTextStyle::new()
                .bold()
                .italic()
                .size(16)
                .color(0x0000FF),
        )
        .unwrap();
    writer.add_paragraph("Another plain paragraph").unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    assert!(text.contains("Plain paragraph"));
    assert!(text.contains("Styled paragraph"));
    assert!(text.contains("Another plain paragraph"));
}

#[test]
fn test_hwpx_styled_save_and_read() {
    let mut writer = HwpxWriter::new();

    writer
        .add_styled_paragraph("제목", HwpxTextStyle::new().bold().size(24))
        .unwrap();
    writer.add_paragraph("본문 내용입니다.").unwrap();
    writer
        .add_styled_paragraph("빨간색 텍스트", HwpxTextStyle::new().color(0xFF0000))
        .unwrap();

    let output_path = PathBuf::from("test-files/styled_test.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists());

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();

    assert!(text.contains("제목"));
    assert!(text.contains("본문 내용입니다"));
    assert!(text.contains("빨간색 텍스트"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_hwpx_simple_table() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("Table below:").unwrap();

    let table = HwpxTable::from_data(vec![
        vec!["Header 1", "Header 2", "Header 3"],
        vec!["A1", "B1", "C1"],
        vec!["A2", "B2", "C2"],
    ]);
    writer.add_table(table).unwrap();

    writer.add_paragraph("Table above.").unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    assert!(text.contains("Table below"));
    assert!(text.contains("Table above"));
}

#[test]
fn test_hwpx_table_with_korean() {
    let mut writer = HwpxWriter::new();

    let table = HwpxTable::from_data(vec![
        vec!["이름", "나이", "직업"],
        vec!["홍길동", "30", "개발자"],
        vec!["김철수", "25", "디자이너"],
    ]);
    writer.add_table(table).unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_hwpx_table_save_and_read() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("테이블 문서").unwrap();

    let mut table = HwpxTable::new(3, 3);
    table.set_cell(0, 0, "A1");
    table.set_cell(0, 1, "B1");
    table.set_cell(0, 2, "C1");
    table.set_cell(1, 0, "A2");
    table.set_cell(1, 1, "B2");
    table.set_cell(1, 2, "C2");
    table.set_cell(2, 0, "A3");
    table.set_cell(2, 1, "B3");
    table.set_cell(2, 2, "C3");
    writer.add_table(table).unwrap();

    let output_path = PathBuf::from("test-files/table_test.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists());

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();

    assert!(text.contains("테이블 문서"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_hwpx_image_basic() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("Image below:").unwrap();

    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let image = HwpxImage::from_bytes(png_data).expect("Valid PNG");
    writer.add_image(image.with_size(30, 30)).unwrap();

    writer.add_paragraph("Image above.").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();
    assert!(text.contains("Image below"));
    assert!(text.contains("Image above"));
}

#[test]
fn test_hwpx_image_save_and_read() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("문서에 이미지 포함").unwrap();

    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let image = HwpxImage::from_bytes(png_data).expect("Valid PNG");
    writer.add_image(image).unwrap();

    let output_path = PathBuf::from("test-files/image_test.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists());

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();
    assert!(text.contains("문서에 이미지 포함"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_hwpx_hyperlink_basic() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("Check out this link:").unwrap();
    writer
        .add_hyperlink("Rust Website", "https://rust-lang.org")
        .unwrap();
    writer.add_paragraph("End of document.").unwrap();

    let bytes = writer.to_bytes().unwrap();
    let document = HwpxReader::from_bytes(&bytes).expect("Failed to read");
    let text = document.extract_text();

    assert!(text.contains("Check out this link"));
    assert!(text.contains("Rust Website"));
    assert!(text.contains("End of document"));
}

#[test]
fn test_hwpx_multiple_hyperlinks() {
    let mut writer = HwpxWriter::new();

    let links = vec![
        HwpxHyperlink::new("Google", "https://google.com"),
        HwpxHyperlink::new("GitHub", "https://github.com"),
    ];
    writer
        .add_paragraph_with_hyperlinks("Visit Google or GitHub for more.", links)
        .unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_hwpx_hyperlink_save_and_read() {
    let mut writer = HwpxWriter::new();

    writer.add_paragraph("하이퍼링크 테스트").unwrap();
    writer.add_hyperlink("네이버", "https://naver.com").unwrap();

    let output_path = PathBuf::from("test-files/hyperlink_test.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists());

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();

    assert!(text.contains("하이퍼링크 테스트"));
    assert!(text.contains("네이버"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_hwpx_header_basic() {
    let mut writer = HwpxWriter::new();

    writer.add_header("Document Header");
    writer.add_paragraph("Main content").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let content = String::from_utf8_lossy(&bytes);
    assert!(content.contains("masterPage") || bytes.len() > 100);
}

#[test]
fn test_hwpx_footer_basic() {
    let mut writer = HwpxWriter::new();

    writer.add_footer("Document Footer");
    writer.add_paragraph("Main content").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_hwpx_footer_with_page_number() {
    let mut writer = HwpxWriter::new();

    writer.add_footer_with_page_number("Page ");
    writer.add_paragraph("Document content").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_hwpx_header_footer_combined() {
    use hwpers::hwpx::{HwpxFooter, PageNumberFormat};

    let mut writer = HwpxWriter::new();

    writer.add_header("My Document Title");
    writer.add_footer_config(
        HwpxFooter::new("Page ").with_page_number_format(PageNumberFormat::Numeric),
    );

    writer.add_paragraph("First paragraph").unwrap();
    writer.add_paragraph("Second paragraph").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_hwpx_header_footer_save_and_read() {
    let mut writer = HwpxWriter::new();

    writer.add_header("문서 머리글");
    writer.add_footer("문서 바닥글");
    writer.add_paragraph("본문 내용입니다.").unwrap();

    let output_path = PathBuf::from("test-files/header_footer_test.hwpx");
    writer.save_to_file(&output_path).unwrap();

    assert!(output_path.exists());

    let document = HwpxReader::from_file(&output_path).expect("Failed to read saved file");
    let text = document.extract_text();

    assert!(text.contains("본문 내용입니다."));
}

#[test]
fn test_hwpx_odd_even_headers() {
    use hwpers::hwpx::HwpxHeader;

    let mut writer = HwpxWriter::new();

    writer.add_header_config(HwpxHeader::for_odd_pages("Odd Page Header"));
    writer.add_header_config(HwpxHeader::for_even_pages("Even Page Header"));
    writer.add_paragraph("Content").unwrap();

    let bytes = writer.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}
