// Verify that generated HWP files can be read back
use hwpers::HwpReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = [
        "complete_feature_demo.hwp",
        "hyperlink_example.hwp", 
        "header_footer_example.hwp",
        "alignment_example.hwp",
        "spacing_example.hwp",
    ];

    for file in &files {
        println!("\n=== Verifying {} ===", file);
        
        match HwpReader::from_file(file) {
            Ok(doc) => {
                println!("✅ Successfully parsed");
                
                // Extract and show text
                let text = doc.extract_text();
                let preview = if text.chars().count() > 50 {
                    let truncated: String = text.chars().take(50).collect();
                    format!("{}...", truncated)
                } else {
                    text.clone()
                };
                println!("Text preview: {}", preview);
                
                // Show document structure
                println!("Sections: {}", doc.sections().count());
                
                let total_paragraphs: usize = doc.sections()
                    .map(|s| s.paragraphs.len())
                    .sum();
                println!("Total paragraphs: {}", total_paragraphs);
            }
            Err(e) => {
                println!("❌ Failed to parse: {}", e);
            }
        }
    }

    Ok(())
}