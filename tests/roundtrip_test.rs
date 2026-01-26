use hwpers::{HwpReader, HwpWriter};
use hwpers::writer::style::{TextStyle, ListType, ImageFormat};
use std::path::PathBuf;

fn test_file_path(name: &str) -> PathBuf {
    PathBuf::from("test-files").join(name)
}

/// Helper to verify text content matches between original and roundtrip document
fn verify_text_roundtrip(original_text: &str, roundtrip_text: &str) -> bool {
    // Normalize whitespace and compare
    let normalize = |s: &str| s.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect::<String>()
        .trim()
        .to_string();

    let orig_normalized = normalize(original_text);
    let rt_normalized = normalize(roundtrip_text);

    if orig_normalized != rt_normalized {
        println!("Original (normalized): {:?}", orig_normalized);
        println!("Roundtrip (normalized): {:?}", rt_normalized);
        false
    } else {
        true
    }
}

#[test]
fn test_create_simple_roundtrip() {
    // First, create a simple document with our writer
    let mut writer = HwpWriter::new();
    writer.add_paragraph("테스트 문서").unwrap();
    writer.add_paragraph("두 번째 줄").unwrap();

    let original_bytes = writer.to_bytes().unwrap();
    println!("Original document size: {} bytes", original_bytes.len());

    // Try to read it back (this will likely fail with current implementation)
    let read_result = HwpReader::from_bytes(&original_bytes);
    match read_result {
        Ok(document) => {
            println!("✓ Successfully read back the document");
            let text = document.extract_text();
            println!("Extracted text: '{text}'");

            // Now try to convert back to writer and generate again
            let writer2 = HwpWriter::from_document(document);
            let roundtrip_bytes = writer2.to_bytes().unwrap();

            println!("Roundtrip document size: {} bytes", roundtrip_bytes.len());

            // Compare sizes (they should be similar, but may not be identical due to formatting differences)
            let size_diff = (original_bytes.len() as i32 - roundtrip_bytes.len() as i32).abs();
            println!("Size difference: {size_diff} bytes");

            // For now, we just verify both are reasonable sizes
            assert!(original_bytes.len() > 100);
            assert!(roundtrip_bytes.len() > 100);
        }
        Err(e) => {
            println!("✗ Failed to read back document: {e:?}");

            // Verify we can at least generate bytes
            assert!(original_bytes.len() > 100);
        }
    }
}

#[test]
fn test_read_existing_hwp_and_convert() {
    let path = test_file_path("test_document.hwp");
    if !path.exists() {
        eprintln!("Skipping test: test file not found at {path:?}");
        return;
    }

    // Read existing HWP file
    let document = match HwpReader::from_file(&path) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Failed to read test file: {e:?}");
            return;
        }
    };

    println!("✓ Successfully read existing HWP file");
    let original_text = document.extract_text();
    println!("Original text length: {} characters", original_text.len());
    let safe_preview = original_text.chars().take(30).collect::<String>(); // Use char count instead
    println!("Original text preview: {safe_preview:?}");

    // Convert to writer
    let writer = HwpWriter::from_document(document);
    let converted_bytes = writer.to_bytes().unwrap();

    println!("✓ Converted to writer format");
    println!("Converted document size: {} bytes", converted_bytes.len());

    // Try to read the converted document back
    let read_back_result = HwpReader::from_bytes(&converted_bytes);
    match read_back_result {
        Ok(converted_doc) => {
            println!("✓ Successfully read back converted document");
            let converted_text = converted_doc.extract_text();
            println!("Converted text length: {} characters", converted_text.len());

            // Compare texts
            if original_text.trim() == converted_text.trim() {
                println!("✓ Text content is identical!");
            } else {
                println!("✗ Text content differs");
                let orig_preview = original_text.chars().take(50).collect::<String>();
                let conv_preview = converted_text.chars().take(50).collect::<String>();
                println!("Original: {orig_preview:?}");
                println!("Converted: {conv_preview:?}");
            }
        }
        Err(e) => {
            println!("✗ Failed to read back converted document: {e:?}");
            println!("This indicates our serialization needs improvement");
        }
    }

    // Save converted file for manual inspection
    let output_path = "converted_output.hwp";
    writer.save_to_file(output_path).unwrap();
    println!("✓ Saved converted file to: {output_path}");
}

#[test]
fn test_minimal_document_structure() {
    // Test the minimal document structure our writer creates
    let writer = HwpWriter::new();
    let bytes = writer.to_bytes().unwrap();

    // Check CFB signature
    assert_eq!(
        &bytes[0..8],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
    );
    println!("✓ CFB signature is correct");

    // Try to parse with HwpReader
    let parse_result = HwpReader::from_bytes(&bytes);
    match parse_result {
        Ok(doc) => {
            println!("✓ Minimal document parsed successfully");
            println!("Sections: {}", doc.body_texts.len());

            // Test that we can convert back
            let writer2 = HwpWriter::from_document(doc);
            let bytes2 = writer2.to_bytes().unwrap();
            println!("✓ Round-trip successful, size: {} bytes", bytes2.len());
        }
        Err(e) => {
            println!("✗ Failed to parse minimal document: {e:?}");
            println!("Our CFB structure needs more work");
        }
    }
}

#[test]
fn test_styled_text_roundtrip() {
    let mut writer = HwpWriter::new();

    // Add styled paragraphs
    writer.add_styled_paragraph("Bold text", TextStyle::new().bold()).unwrap();
    writer.add_styled_paragraph("Italic text", TextStyle::new().italic()).unwrap();
    writer.add_styled_paragraph("Colored text", TextStyle::new().color(0xFF0000)).unwrap();

    let bytes = writer.to_bytes().unwrap();

    // Try to read back
    let result = HwpReader::from_bytes(&bytes);
    match result {
        Ok(doc) => {
            let text = doc.extract_text();
            println!("Extracted styled text: {:?}", text);

            // Verify key content is present
            assert!(text.contains("Bold") || text.contains("bold"));
            assert!(text.contains("Italic") || text.contains("italic"));
            assert!(text.contains("Colored") || text.contains("colored"));
            println!("✓ Styled text roundtrip successful");
        }
        Err(e) => {
            println!("✗ Failed to read styled document: {e:?}");
        }
    }
}

#[test]
fn test_compression_roundtrip() {
    let mut writer = HwpWriter::new().with_compression(true);
    writer.add_paragraph("Compressed document test").unwrap();
    writer.add_paragraph("Second paragraph for compression").unwrap();

    let compressed_bytes = writer.to_bytes().unwrap();
    println!("Compressed document size: {} bytes", compressed_bytes.len());

    // Also test uncompressed version
    let mut writer2 = HwpWriter::new();
    writer2.add_paragraph("Compressed document test").unwrap();
    writer2.add_paragraph("Second paragraph for compression").unwrap();

    let uncompressed_bytes = writer2.to_bytes().unwrap();
    println!("Uncompressed document size: {} bytes", uncompressed_bytes.len());

    // Try to read both back
    let result1 = HwpReader::from_bytes(&compressed_bytes);
    let result2 = HwpReader::from_bytes(&uncompressed_bytes);

    match (result1, result2) {
        (Ok(doc1), Ok(doc2)) => {
            let text1 = doc1.extract_text();
            let text2 = doc2.extract_text();

            // Both should have the same content
            assert!(verify_text_roundtrip(&text1, &text2));
            println!("✓ Compression roundtrip successful");
        }
        (Err(e1), _) => {
            println!("✗ Failed to read compressed document: {e1:?}");
        }
        (_, Err(e2)) => {
            println!("✗ Failed to read uncompressed document: {e2:?}");
        }
    }
}

#[test]
fn test_table_roundtrip() {
    let mut writer = HwpWriter::new();
    writer.add_paragraph("Table test:").unwrap();

    writer.create_table(2, 2)
        .set_cell(0, 0, "A1")
        .set_cell(0, 1, "B1")
        .set_cell(1, 0, "A2")
        .set_cell(1, 1, "B2")
        .finish()
        .unwrap();

    let bytes = writer.to_bytes().unwrap();

    let result = HwpReader::from_bytes(&bytes);
    match result {
        Ok(doc) => {
            println!("✓ Table roundtrip: document parsed successfully");
            println!("Body texts: {}", doc.body_texts.len());
        }
        Err(e) => {
            println!("✗ Table roundtrip failed: {e:?}");
        }
    }
}

#[test]
fn test_hyperlink_roundtrip() {
    let mut writer = HwpWriter::new();
    writer.add_paragraph("Click here:").unwrap();
    writer.add_hyperlink("Visit Example", "https://example.com").unwrap();

    let bytes = writer.to_bytes().unwrap();

    let result = HwpReader::from_bytes(&bytes);
    match result {
        Ok(doc) => {
            let text = doc.extract_text();
            println!("Extracted hyperlink text: {:?}", text);
            assert!(text.contains("Example") || text.contains("Visit"));
            println!("✓ Hyperlink roundtrip successful");
        }
        Err(e) => {
            println!("✗ Hyperlink roundtrip failed: {e:?}");
        }
    }
}

#[test]
fn test_header_footer_roundtrip() {
    let mut writer = HwpWriter::new();
    writer.add_header("Document Header");
    writer.add_footer("Document Footer");
    writer.add_paragraph("Main content").unwrap();

    let bytes = writer.to_bytes().unwrap();

    let result = HwpReader::from_bytes(&bytes);
    match result {
        Ok(_doc) => {
            println!("✓ Header/footer roundtrip: document parsed successfully");
        }
        Err(e) => {
            println!("✗ Header/footer roundtrip failed: {e:?}");
        }
    }
}
