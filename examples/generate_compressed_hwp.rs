//! Generate compressed HWP file

use hwpers::HwpWriter;

fn main() {
    // Create document with compression enabled
    let mut writer = HwpWriter::new().with_compression(true);
    writer.add_paragraph("압축 테스트 문서").unwrap();
    writer.add_paragraph("두 번째 문단입니다.").unwrap();

    let output_path = "compressed_test.hwp";
    writer.save_to_file(output_path).unwrap();
    println!("✓ 압축된 HWP 파일 생성: {}", output_path);

    // Also verify it can be read back
    match hwpers::HwpReader::from_file(output_path) {
        Ok(doc) => {
            println!("✓ 파일 읽기 성공");
            println!("  텍스트: {:?}", doc.extract_text());
        }
        Err(e) => println!("✗ 읽기 실패: {:?}", e)
    }
}
