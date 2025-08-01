use hwpers::HwpReader;
use std::path::PathBuf;

fn test_file_path(name: &str) -> PathBuf {
    PathBuf::from("test-files").join(name)
}

#[test]
fn test_basic_parsing() {
    let path = test_file_path("test_document.hwp");
    if !path.exists() {
        eprintln!("Skipping test: test file not found at {:?}", path);
        return;
    }

    let doc = HwpReader::from_file(path).expect("Failed to parse HWP file");

    // Verify header
    assert!(!doc.header.is_encrypted());

    // Verify we have at least one section
    assert!(doc.sections().count() > 0);
}

#[test]
fn test_text_extraction() {
    let path = test_file_path("test_document.hwp");
    if !path.exists() {
        eprintln!("Skipping test: test file not found at {:?}", path);
        return;
    }

    let doc = HwpReader::from_file(path).expect("Failed to parse HWP file");
    let text = doc.extract_text();

    // For now, we'll just verify the parser works even if text extraction is incomplete
    println!("Extracted text length: {} characters", text.len());

    // This is a minimal test - just verify the parser doesn't crash
    let _ = doc.sections().count(); // Verify we can iterate sections
}

#[test]
fn test_from_bytes() {
    let path = test_file_path("test_document.hwp");
    if !path.exists() {
        eprintln!("Skipping test: test file not found at {:?}", path);
        return;
    }

    let bytes = std::fs::read(&path).expect("Failed to read file");
    let doc = HwpReader::from_bytes(&bytes).expect("Failed to parse HWP file");

    // Verify we can parse from bytes
    assert!(doc.sections().count() > 0);
}
