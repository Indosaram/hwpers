use hwpers::{HwpReader, HwpWriter};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Round-trip Verification ===\n");

    // 1. Read original file
    let original_path = "test-files/test_document.hwp";
    let original_bytes = fs::read(original_path)?;
    println!("Original file size: {} bytes", original_bytes.len());

    let original_doc = HwpReader::from_file(original_path)?;
    let original_text = original_doc.extract_text();
    println!("Original text length: {} characters", original_text.len());
    println!("Original text hash: {:x}", md5::compute(&original_text));
    println!(
        "First 200 chars: {:?}",
        &original_text.chars().take(200).collect::<String>()
    );

    // 2. Convert to Writer and save
    let writer = HwpWriter::from_document(original_doc);
    let roundtrip_path = "verify_roundtrip.hwp";
    writer.save_to_file(roundtrip_path)?;

    let roundtrip_bytes = fs::read(roundtrip_path)?;
    println!("\nRoundtrip file size: {} bytes", roundtrip_bytes.len());

    // 3. Read back the roundtrip file
    let roundtrip_doc = HwpReader::from_file(roundtrip_path)?;
    let roundtrip_text = roundtrip_doc.extract_text();
    println!("Roundtrip text length: {} characters", roundtrip_text.len());
    println!("Roundtrip text hash: {:x}", md5::compute(&roundtrip_text));
    println!(
        "First 200 chars: {:?}",
        &roundtrip_text.chars().take(200).collect::<String>()
    );

    // 4. Compare character by character
    println!("\n=== Comparison ===");
    if original_text == roundtrip_text {
        println!("✅ PERFECT MATCH! Every single character is identical!");

        // Additional verification
        let orig_chars: Vec<char> = original_text.chars().collect();
        let round_chars: Vec<char> = roundtrip_text.chars().collect();

        println!(
            "Character count: {} vs {}",
            orig_chars.len(),
            round_chars.len()
        );

        // Check some random positions
        let positions = [0, 100, 500, 1000, 5000];
        for &pos in &positions {
            if pos < orig_chars.len() {
                println!(
                    "Position {}: '{}' vs '{}'",
                    pos, orig_chars[pos], round_chars[pos]
                );
            }
        }
    } else {
        println!("❌ Text differs!");

        // Find first difference
        let orig_chars: Vec<char> = original_text.chars().collect();
        let round_chars: Vec<char> = roundtrip_text.chars().collect();

        for i in 0..orig_chars.len().min(round_chars.len()) {
            if orig_chars[i] != round_chars[i] {
                println!(
                    "First difference at position {}: '{}' vs '{}'",
                    i, orig_chars[i], round_chars[i]
                );
                break;
            }
        }
    }

    Ok(())
}

// Simple MD5 implementation for demo
mod md5 {
    pub fn compute(data: &str) -> u128 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish() as u128
    }
}
