// Analyze hyperlink structure from test file
use hwpers::reader::CfbReader;
use hwpers::parser::record::{Record, HwpTag};
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 하이퍼링크 구조 분석 ===\n");

    let file_path = "hyperlink_document.hwp";
    let mut reader = CfbReader::from_file(file_path)?;
    
    // Parse FileHeader
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;
    
    // List streams first
    println!("🗂️ CFB 스트림 목록:");
    reader.list_streams().iter().for_each(|stream| {
        println!("  - {}", stream);
    });
    println!();
    
    // Analyze BodyText/Section0
    let section_data = reader.read_stream("BodyText/Section0")?;
    
    // Decompress if needed
    let data = if header.is_compressed() {
        decompress_stream(&section_data)?
    } else {
        section_data
    };
    
    // Find ParaRangeTag records (hyperlinks)
    let mut stream_reader = StreamReader::new(data);
    let mut hyperlink_count = 0;
    
    while stream_reader.remaining() >= 8 {
        let position = stream_reader.position();
        
        match Record::parse(&mut stream_reader) {
            Ok(record) => {
                let tag = HwpTag::from_u16(record.tag_id());
                
                if tag == Some(HwpTag::ParaRangeTag) {
                    hyperlink_count += 1;
                    println!("🔗 하이퍼링크 #{} at 0x{:08X}:", hyperlink_count, position);
                    analyze_hyperlink_detailed(&record, position);
                    println!();
                }
            }
            Err(_) => break,
        }
    }
    
    if hyperlink_count == 0 {
        println!("❌ 하이퍼링크를 찾을 수 없습니다.");
    } else {
        println!("✅ 총 {} 개의 하이퍼링크를 발견했습니다.", hyperlink_count);
    }
    
    Ok(())
}

fn analyze_hyperlink_detailed(record: &Record, _position: u64) {
    println!("  크기: {} bytes", record.data.len());
    
    // Print hex dump
    println!("  📋 헥스 덤프:");
    print_hex_dump(&record.data, 8);
    
    // Try to parse with current implementation
    println!("  🧪 현재 구현으로 파싱:");
    match hwpers::model::hyperlink::Hyperlink::from_record(record) {
        Ok(hyperlink) => {
            println!("    ✅ 파싱 성공:");
            println!("      표시 텍스트: \"{}\"", hyperlink.display_text);
            println!("      대상 URL: \"{}\"", hyperlink.target_url);
            println!("      유형: {:?}", hyperlink.hyperlink_type);
            println!("      위치: {}, 길이: {}", hyperlink.start_position, hyperlink.length);
        }
        Err(e) => {
            println!("    ❌ 파싱 실패: {}", e);
        }
    }
    
    // Manual structure analysis
    println!("  🔬 수동 구조 분석:");
    
    if record.data.len() >= 8 {
        let mut reader = StreamReader::new(record.data.clone());
        
        // Try to parse the known structure
        match (|| -> Result<(), Box<dyn std::error::Error>> {
            let control_id = reader.read_u32()?;
            println!("    컨트롤 ID: 0x{:08X}", control_id);
            
            // Skip some bytes and look for patterns
            if reader.remaining() >= 4 {
                let next_field = reader.read_u32()?;
                println!("    다음 필드: 0x{:08X} ({})", next_field, next_field);
            }
            
            // Look for range information
            if reader.remaining() >= 8 {
                let start_pos = reader.read_u32()?;
                let length = reader.read_u32()?;
                println!("    범위: 시작={}, 길이={}", start_pos, length);
            }
            
            // Try to find URL and display text
            if reader.remaining() > 0 {
                println!("    남은 데이터: {} bytes", reader.remaining());
                
                // Try to find text patterns
                let remaining_data = reader.read_bytes(reader.remaining())?;
                analyze_text_in_data(&remaining_data);
            }
            
            Ok(())
        })() {
            Ok(_) => {}
            Err(e) => {
                println!("    구조 분석 중 오류: {}", e);
            }
        }
    }
    
    // Look for string patterns in the data
    println!("  🔤 문자열 패턴 검색:");
    find_strings_in_data(&record.data);
}

fn find_strings_in_data(data: &[u8]) {
    // Look for UTF-16LE strings
    if data.len() >= 4 {
        for start in 0..data.len() - 3 {
            // Look for length-prefixed UTF-16LE strings
            if start + 2 < data.len() {
                let len = u16::from_le_bytes([data[start], data[start + 1]]) as usize;
                if len > 0 && len < 100 && start + 2 + len * 2 <= data.len() {
                    let text_bytes = &data[start + 2..start + 2 + len * 2];
                    if text_bytes.len() % 2 == 0 {
                        let mut utf16_chars = Vec::new();
                        for chunk in text_bytes.chunks_exact(2) {
                            let char_value = u16::from_le_bytes([chunk[0], chunk[1]]);
                            utf16_chars.push(char_value);
                        }
                        
                        if let Ok(text) = String::from_utf16(&utf16_chars) {
                            if text.chars().all(|c| c.is_ascii() || c as u32 > 127) && !text.trim().is_empty() {
                                println!("    오프셋 0x{:04X}: 길이={}, 텍스트=\"{}\"", start, len, text);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Look for NULL-terminated UTF-16LE strings
    for start in (0..data.len()).step_by(2) {
        if start + 1 < data.len() {
            let mut end = start;
            let mut chars = Vec::new();
            
            while end + 1 < data.len() {
                let char_value = u16::from_le_bytes([data[end], data[end + 1]]);
                if char_value == 0 {
                    break;
                }
                chars.push(char_value);
                end += 2;
                
                if chars.len() > 50 { // Limit length
                    break;
                }
            }
            
            if chars.len() >= 3 && chars.len() <= 50 {
                if let Ok(text) = String::from_utf16(&chars) {
                    if text.chars().all(|c| c.is_ascii_graphic() || c == ' ' || c as u32 > 127) {
                        println!("    오프셋 0x{:04X}: NULL 종료 문자열=\"{}\"", start, text);
                    }
                }
            }
        }
    }
}

fn analyze_text_in_data(data: &[u8]) {
    println!("      텍스트 데이터 분석 ({} bytes):", data.len());
    
    // Try different interpretations
    if data.len() >= 2 {
        // Try as length-prefixed UTF-16LE
        let mut offset = 0;
        while offset + 2 <= data.len() {
            let len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            if len > 0 && len < 100 && offset + 2 + len * 2 <= data.len() {
                let text_bytes = &data[offset + 2..offset + 2 + len * 2];
                if let Ok(text) = parse_utf16le(text_bytes) {
                    if !text.trim().is_empty() {
                        println!("        오프셋 {}: \"{}\" (길이: {})", offset, text, len);
                    }
                }
                offset += 2 + len * 2;
            } else {
                offset += 2;
            }
            
            if offset >= data.len() {
                break;
            }
        }
    }
}

fn parse_utf16le(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    if data.len() % 2 != 0 {
        return Err("Invalid UTF-16LE data length".into());
    }
    
    let mut chars = Vec::new();
    for chunk in data.chunks_exact(2) {
        let char_value = u16::from_le_bytes([chunk[0], chunk[1]]);
        chars.push(char_value);
    }
    
    Ok(String::from_utf16(&chars)?)
}

fn print_hex_dump(data: &[u8], max_lines: usize) {
    for (i, chunk) in data.chunks(16).enumerate().take(max_lines) {
        print!("    {:04X}: ", i * 16);
        for &byte in chunk {
            print!("{:02X} ", byte);
        }
        // Pad if chunk is less than 16 bytes
        for _ in chunk.len()..16 {
            print!("   ");
        }
        print!(" | ");
        for &byte in chunk {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!();
    }
    if data.len() > max_lines * 16 {
        println!("    ... ({} more bytes)", data.len() - max_lines * 16);
    }
}