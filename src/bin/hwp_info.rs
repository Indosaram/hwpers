use hwpers::HwpReader;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <hwp_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = Path::new(&args[1]);

    if !file_path.exists() {
        eprintln!("File not found: {file_path:?}");
        std::process::exit(1);
    }

    // First, try to read as CFB
    match hwpers::reader::CfbReader::from_file(file_path) {
        Ok(mut reader) => {
            println!("CFB structure detected!");
            println!("Available streams:");
            for stream in reader.list_streams() {
                println!("  - {stream}");
            }

            // Try to read FileHeader
            match reader.read_stream("FileHeader") {
                Ok(data) => {
                    println!("\nFileHeader stream: {} bytes", data.len());
                    if data.len() >= 32 {
                        let signature = String::from_utf8_lossy(&data[..32]);
                        println!("Signature: {signature:?}");
                    }
                }
                Err(e) => {
                    println!("Error reading FileHeader: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("Not a valid CFB file: {e}");
            std::process::exit(1);
        }
    }

    println!("\nTrying full parse...");
    match HwpReader::from_file(file_path) {
        Ok(doc) => {
            println!("Successfully parsed!");
            println!("Version: {}", doc.header.version_string());
            println!("Compressed: {}", doc.header.is_compressed());
            let section_count = doc.sections().count();
            println!("Sections: {section_count}");

            if section_count == 0 {
                println!("\nNo sections found. Body texts: {}", doc.body_texts.len());
                for (i, bt) in doc.body_texts.iter().enumerate() {
                    println!("  BodyText {}: {} sections", i, bt.sections.len());
                }
            }
        }
        Err(e) => {
            println!("Parse error: {e}");
        }
    }
}
