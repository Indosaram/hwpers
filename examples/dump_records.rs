//! Dump HWP record structure

use std::env;
use std::io::Read;
use flate2::read::{ZlibDecoder, DeflateDecoder};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file.hwp> [stream]", args[0]);
        return;
    }

    let path = &args[1];
    let stream = if args.len() > 2 { &args[2] } else { "/DocInfo" };

    dump_records(path, stream);
}

fn dump_records(path: &str, stream_path: &str) {
    let mut cfb = cfb::open(path).unwrap();
    let mut stream = cfb.open_stream(stream_path).unwrap();
    let mut data = Vec::new();
    stream.read_to_end(&mut data).unwrap();

    // Try to decompress - HWP uses raw deflate (not zlib)
    let data = {
        // First try raw deflate
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        match decoder.read_to_end(&mut decompressed) {
            Ok(_) if !decompressed.is_empty() => {
                println!("Stream decompressed with raw deflate: {} -> {} bytes", data.len(), decompressed.len());
                decompressed
            }
            _ => {
                // Try zlib
                let mut decoder = ZlibDecoder::new(&data[..]);
                let mut decompressed = Vec::new();
                match decoder.read_to_end(&mut decompressed) {
                    Ok(_) if !decompressed.is_empty() => {
                        println!("Stream decompressed with zlib: {} -> {} bytes", data.len(), decompressed.len());
                        decompressed
                    }
                    _ => {
                        println!("Stream not compressed or unknown format ({} bytes)", data.len());
                        data
                    }
                }
            }
        }
    };

    println!("Stream: {} ({} bytes)", stream_path, data.len());
    println!();

    let mut offset = 0;
    let mut count = 0;

    while offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }

        // Read record header (4 bytes)
        let header = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]);

        let tag = header & 0x3FF;  // 10 bits
        let level = (header >> 10) & 0x3FF;  // 10 bits
        let size = (header >> 20) & 0xFFF;  // 12 bits

        let actual_size = if size == 0xFFF {
            // Extended size
            if offset + 8 > data.len() {
                break;
            }
            u32::from_le_bytes([
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
            ]) as usize
        } else {
            size as usize
        };

        let header_size = if size == 0xFFF { 8 } else { 4 };
        let data_start = offset + header_size;
        let data_end = data_start + actual_size;

        if data_end > data.len() {
            println!("[{}] @{:04x}: tag=0x{:03x} level={} size={} (TRUNCATED)",
                count, offset, tag, level, actual_size);
            break;
        }

        let tag_name = get_tag_name(tag as u16);
        print!("[{:3}] @{:04x}: tag=0x{:03x}({:16}) level={} size={:4}",
            count, offset, tag, tag_name, level, actual_size);

        // Show preview of data
        if actual_size > 0 && actual_size <= 16 {
            print!("  data=");
            for b in &data[data_start..data_end] {
                print!("{:02x} ", b);
            }
        } else if actual_size > 16 {
            print!("  data=");
            for b in &data[data_start..data_start + 16] {
                print!("{:02x} ", b);
            }
            print!("...");
        }
        println!();

        offset = data_end;
        count += 1;

        if count > 100 {
            println!("... (stopped at 100 records)");
            break;
        }
    }
}

fn get_tag_name(tag: u16) -> &'static str {
    match tag {
        0x10 => "DOC_PROPERTIES",
        0x11 => "ID_MAPPINGS",
        0x12 => "BIN_DATA",
        0x13 => "FACE_NAME",
        0x14 => "BORDER_FILL",
        0x15 => "CHAR_SHAPE",
        0x16 => "TAB_DEF",
        0x17 => "NUMBERING",
        0x18 => "BULLET",
        0x19 => "PARA_SHAPE",
        0x1A => "STYLE",
        0x1B => "DOC_DATA",
        0x1E => "COMPATIBLE_DOC",
        0x1F => "LAYOUT_COMPAT",
        0x42 => "PARA_HEADER",
        0x43 => "PARA_TEXT",
        0x44 => "PARA_CHAR_SHAPE",
        0x45 => "PARA_LINE_SEG",
        0x46 => "PARA_RANGE_TAG",
        0x47 => "CTRL_HEADER",
        0x48 => "LIST_HEADER",
        0x49 => "PAGE_DEF",
        0x4A => "FOOTNOTE_SHAPE",
        0x4B => "PAGE_BORDER_FILL",
        0x4C => "SHAPE_COMPONENT",
        0x4F => "TABLE",
        0x50 => "CELL",
        _ => "UNKNOWN",
    }
}
