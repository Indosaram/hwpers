use hwpers::parser::record::{HwpTag, Record};
use hwpers::reader::CfbReader;
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 하이퍼링크 레코드 상세 분석 ===\n");

    let mut reader = CfbReader::from_file("hyperlink_document.hwp")?;

    // Read BodyText/Section0
    let section_data = reader.read_stream("BodyText/Section0")?;
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;

    let data = if header.is_compressed() {
        decompress_stream(&section_data)?
    } else {
        section_data
    };

    // Find the first hyperlink record
    let mut reader = StreamReader::new(data);

    while reader.remaining() >= 8 {
        let position = reader.position();

        match Record::parse(&mut reader) {
            Ok(record) => {
                let tag = HwpTag::from_u16(record.tag_id());

                if tag == Some(HwpTag::ParaRangeTag) {
                    println!("🎯 첫 번째 ParaRangeTag 레코드 (위치 0x{:08X})", position);
                    println!("크기: {} bytes", record.data.len());

                    // Analyze the raw data byte by byte
                    let data = &record.data;
                    println!("\n📊 바이트별 분석:");

                    for (i, chunk) in data.chunks(16).enumerate() {
                        print!("{:04X}: ", i * 16);

                        // Hex dump
                        for (j, &byte) in chunk.iter().enumerate() {
                            if j == 8 {
                                print!(" "); // Space after 8 bytes
                            }
                            print!("{:02X} ", byte);
                        }

                        // Pad remaining space
                        for _ in chunk.len()..16 {
                            if chunk.len() <= 8 {
                                print!("   ");
                            } else {
                                print!("   ");
                            }
                        }

                        print!(" | ");

                        // ASCII interpretation
                        for &byte in chunk {
                            if byte.is_ascii_graphic() || byte == b' ' {
                                print!("{}", byte as char);
                            } else {
                                print!(".");
                            }
                        }
                        println!();
                    }

                    // Try to identify structure
                    println!("\n🔍 구조 분석:");
                    if data.len() >= 4 {
                        let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                        println!(
                            "  0x00-0x03: 컨트롤 ID = 0x{:08X} ('{}')",
                            ctrl_id,
                            String::from_utf8_lossy(&data[0..4])
                        );
                    }

                    // Look for UTF-16 strings
                    println!("\n📝 UTF-16 문자열 검색:");
                    for start in (0..data.len()).step_by(2) {
                        if start + 20 <= data.len() {
                            // At least 10 UTF-16 characters
                            let mut utf16_chars = Vec::new();
                            let mut valid = true;

                            for i in (start..start + 20).step_by(2) {
                                if i + 1 < data.len() {
                                    let char_val = u16::from_le_bytes([data[i], data[i + 1]]);
                                    // Check for valid UTF-16 ranges
                                    if char_val == 0 {
                                        // Allow null termination
                                        utf16_chars.push(char_val);
                                    } else if char_val >= 0xD800 && char_val <= 0xDFFF {
                                        // Invalid surrogate range
                                        valid = false;
                                        break;
                                    } else {
                                        utf16_chars.push(char_val);
                                    }
                                }
                            }

                            if valid && !utf16_chars.is_empty() {
                                if let Ok(text) = String::from_utf16(&utf16_chars) {
                                    let trimmed = text.trim_end_matches('\0');
                                    if trimmed.chars().any(|c| c.is_alphabetic() || c.is_hangul()) {
                                        println!("  위치 0x{:04X}: \"{}\"", start, trimmed);
                                    }
                                }
                            }
                        }
                    }

                    break; // Just analyze the first one
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

trait IsHangul {
    fn is_hangul(&self) -> bool;
}

impl IsHangul for char {
    fn is_hangul(&self) -> bool {
        let code = *self as u32;
        // Hangul Syllables (AC00-D7AF), Hangul Jamo (1100-11FF), etc.
        (code >= 0xAC00 && code <= 0xD7AF)
            || (code >= 0x1100 && code <= 0x11FF)
            || (code >= 0x3130 && code <= 0x318F)
            || (code >= 0xA960 && code <= 0xA97F)
    }
}
