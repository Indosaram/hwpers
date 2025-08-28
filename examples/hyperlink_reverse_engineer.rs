use hwpers::reader::CfbReader;
use hwpers::parser::record::{Record, HwpTag};
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 하이퍼링크 구조 역공학 ===\n");

    let mut reader = CfbReader::from_file("hyperlink_document.hwp")?;
    
    // Read and decompress BodyText/Section0
    let section_data = reader.read_stream("BodyText/Section0")?;
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;
    
    let data = if header.is_compressed() {
        decompress_stream(&section_data)?
    } else {
        section_data
    };
    
    // Find all hyperlink records
    let mut reader = StreamReader::new(data);
    let mut hyperlink_records = Vec::new();
    
    while reader.remaining() >= 8 {
        let position = reader.position();
        
        match Record::parse(&mut reader) {
            Ok(record) => {
                let tag = HwpTag::from_u16(record.tag_id());
                if tag == Some(HwpTag::ParaRangeTag) {
                    hyperlink_records.push((position, record));
                }
            }
            Err(_) => break,
        }
    }
    
    println!("발견된 하이퍼링크 레코드 수: {}", hyperlink_records.len());
    
    // Analyze the structure of the first few records
    for (i, (position, record)) in hyperlink_records.iter().take(3).enumerate() {
        println!("\n🔍 하이퍼링크 #{} (위치 0x{:08X})", i + 1, position);
        let data = &record.data;
        
        // Skip control header and try to find string patterns
        // 28 bytes: control ID + control header
        // Then we should see hyperlink-specific data
        
        if data.len() > 28 {
            println!("  전체 크기: {} bytes", data.len());
            
            // Look for strings starting from various offsets
            let mut found_strings = Vec::new();
            
            // Search for potential strings throughout the record
            for start_offset in (28..data.len()).step_by(2) {
                if start_offset + 4 < data.len() {
                    // Try to read length prefix
                    let potential_len = u16::from_le_bytes([data[start_offset], data[start_offset + 1]]) as usize;
                    
                    // Reasonable string length (1-50 characters)
                    if potential_len > 0 && potential_len <= 50 && start_offset + 2 + potential_len * 2 <= data.len() {
                        let mut utf16_chars = Vec::new();
                        let mut valid = true;
                        
                        for char_offset in (start_offset + 2..start_offset + 2 + potential_len * 2).step_by(2) {
                            if char_offset + 1 < data.len() {
                                let char_val = u16::from_le_bytes([data[char_offset], data[char_offset + 1]]);
                                if char_val == 0 || (char_val >= 32 && char_val < 0xD800) || char_val > 0xDFFF {
                                    utf16_chars.push(char_val);
                                } else {
                                    valid = false;
                                    break;
                                }
                            }
                        }
                        
                        if valid && !utf16_chars.is_empty() {
                            if let Ok(text) = String::from_utf16(&utf16_chars) {
                                let clean_text = text.trim_end_matches('\0');
                                if !clean_text.is_empty() && (clean_text.chars().any(|c| c.is_alphabetic() || is_hangul(c))) {
                                    found_strings.push((start_offset, potential_len, clean_text.to_string()));
                                }
                            }
                        }
                    }
                }
            }
            
            // Display found strings
            println!("  발견된 문자열들:");
            for (offset, len, text) in &found_strings {
                println!("    위치 0x{:04X} (길이 {}): \"{}\"", offset, len, text);
            }
            
            // Show hex dump around string areas
            if !found_strings.is_empty() {
                println!("  해당 영역 16진수 덤프:");
                for (offset, _len, text) in &found_strings {
                    let start = offset.saturating_sub(8);
                    let end = std::cmp::min(start + 64, data.len());
                    
                    println!("    문자열 \"{}\" 주변:", text);
                    for chunk_start in (start..end).step_by(16) {
                        let chunk_end = std::cmp::min(chunk_start + 16, end);
                        print!("      {:04X}: ", chunk_start);
                        
                        for j in chunk_start..chunk_end {
                            if j == *offset {
                                print!("[{:02X}]", data[j]);
                            } else {
                                print!(" {:02X} ", data[j]);
                            }
                        }
                        
                        print!(" | ");
                        for j in chunk_start..chunk_end {
                            let byte = data[j];
                            if byte.is_ascii_graphic() || byte == b' ' {
                                print!("{}", byte as char);
                            } else {
                                print!(".");
                            }
                        }
                        println!();
                    }
                    println!();
                }
            }
        }
    }
    
    Ok(())
}

fn is_hangul(c: char) -> bool {
    let code = c as u32;
    (code >= 0xAC00 && code <= 0xD7AF) ||
    (code >= 0x1100 && code <= 0x11FF) ||
    (code >= 0x3130 && code <= 0x318F)
}