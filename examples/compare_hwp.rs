//! Compare HWP file structures

use std::env;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file1.hwp> <file2.hwp>", args[0]);
        return;
    }

    let path1 = &args[1];
    let path2 = &args[2];

    println!("Comparing: {} vs {}", path1, path2);
    println!();

    compare_streams(path1, path2, "/FileHeader");
    compare_streams(path1, path2, "/DocInfo");
    compare_streams(path1, path2, "/BodyText/Section0");
}

fn compare_streams(path1: &str, path2: &str, stream: &str) {
    println!("=== {} ===", stream);

    let data1 = read_stream(path1, stream);
    let data2 = read_stream(path2, stream);

    match (data1, data2) {
        (Ok(d1), Ok(d2)) => {
            println!("  File1: {} bytes", d1.len());
            println!("  File2: {} bytes", d2.len());

            // Show first 64 bytes of each
            println!("  File1 hex (first 64 bytes):");
            print_hex(&d1, 64);
            println!("  File2 hex (first 64 bytes):");
            print_hex(&d2, 64);
        }
        (Err(e1), Ok(_)) => println!("  File1 error: {:?}", e1),
        (Ok(_), Err(e2)) => println!("  File2 error: {:?}", e2),
        (Err(e1), Err(e2)) => {
            println!("  File1 error: {:?}", e1);
            println!("  File2 error: {:?}", e2);
        }
    }
    println!();
}

fn read_stream(path: &str, stream: &str) -> Result<Vec<u8>, String> {
    let mut cfb = cfb::open(path).map_err(|e| format!("{:?}", e))?;
    let mut stream = cfb.open_stream(stream).map_err(|e| format!("{:?}", e))?;
    let mut data = Vec::new();
    stream.read_to_end(&mut data).map_err(|e| format!("{:?}", e))?;
    Ok(data)
}

fn print_hex(data: &[u8], max_bytes: usize) {
    let bytes_to_show = data.len().min(max_bytes);
    for (i, chunk) in data[..bytes_to_show].chunks(16).enumerate() {
        print!("    {:04x}: ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
        }
        println!();
    }
}
