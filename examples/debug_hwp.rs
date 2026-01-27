//! Debug HWP file structure

use hwpers::HwpReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "test_output.hwp"
    };

    println!("Analyzing: {}", path);

    // Try to read with HwpReader
    match HwpReader::from_file(path) {
        Ok(doc) => {
            println!("✓ File parsed successfully!");
            println!("  Version: {:?}", doc.header.version);
            println!("  Compressed: {}", doc.header.is_compressed());
            println!("  Sections: {}", doc.body_texts.len());

            let text = doc.extract_text();
            println!("  Text length: {} chars", text.len());
            println!("  Text preview: {:?}", text.chars().take(100).collect::<String>());
        }
        Err(e) => {
            println!("✗ Failed to parse: {:?}", e);
        }
    }

    // Also check CFB structure directly
    println!("\n--- CFB Structure ---");
    match cfb::open(path) {
        Ok(mut cfb) => {
            for entry in cfb.walk() {
                println!("  {} ({})", entry.path().display(),
                    if entry.is_stream() {
                        format!("{} bytes", entry.len())
                    } else {
                        "dir".to_string()
                    });
            }
        }
        Err(e) => {
            println!("  CFB error: {:?}", e);
        }
    }
}
