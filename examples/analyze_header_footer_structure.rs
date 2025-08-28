// Analyze the exact structure of HeaderFooter record from sample.hwp
use hwpers::reader::CfbReader;
use hwpers::parser::record::{Record, HwpTag};
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HeaderFooter 구조 분석 ===\n");

    let file_path = "/Users/indo/Downloads/sample.hwp";
    let mut reader = CfbReader::from_file(file_path)?;
    
    // Parse FileHeader
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;
    
    // Analyze BodyText/Section0
    let section_data = reader.read_stream("BodyText/Section0")?;
    
    // Decompress if needed
    let data = if header.is_compressed() {
        decompress_stream(&section_data)?
    } else {
        section_data
    };
    
    // Find HeaderFooter record
    let mut stream_reader = StreamReader::new(data);
    
    while stream_reader.remaining() >= 8 {
        let position = stream_reader.position();
        
        match Record::parse(&mut stream_reader) {
            Ok(record) => {
                let tag = HwpTag::from_u16(record.tag_id());
                
                if tag == Some(HwpTag::HeaderFooter) {
                    analyze_header_footer_detailed(&record, position);
                    break; // Only analyze the first one
                }
            }
            Err(_) => break,
        }
    }
    
    Ok(())
}

fn analyze_header_footer_detailed(record: &Record, position: u64) {
    println!("🔍 HeaderFooter 레코드 상세 분석:");
    println!("  위치: 0x{:08X}", position);
    println!("  크기: {} bytes", record.data.len());
    println!();
    
    // Print complete hex dump
    println!("📋 완전한 헥스 덤프:");
    print_hex_dump(&record.data, 16);
    println!();
    
    // Try to parse as our current implementation expects
    println!("🧪 현재 구현으로 파싱 시도:");
    match hwpers::model::header_footer::HeaderFooter::from_record(record) {
        Ok(header_footer) => {
            println!("  ✅ 파싱 성공:");
            println!("    타입: {:?}", header_footer.header_footer_type);
            println!("    적용: {:?}", header_footer.apply_type);
            println!("    텍스트: \"{}\"", header_footer.text);
            println!("    정렬: {}", header_footer.alignment);
            println!("    높이: {} HWPU", header_footer.height);
            println!("    여백: {} HWPU", header_footer.margin);
        }
        Err(e) => {
            println!("  ❌ 파싱 실패: {}", e);
        }
    }
    println!();
    
    // Manual analysis of the structure
    println!("🔬 수동 구조 분석:");
    
    if record.data.len() >= 40 {
        let mut reader = StreamReader::new(record.data.clone());
        
        println!("  바이트 단위 분석:");
        for i in 0..10 {
            if reader.remaining() >= 4 {
                let value = reader.read_u32().unwrap();
                println!("    오프셋 {}: 0x{:08X} ({})", i * 4, value, value);
            }
        }
        
        // Reset reader and try different interpretations
        reader = StreamReader::new(record.data.clone());
        
        println!();
        println!("  가능한 구조 해석:");
        
        // Interpretation 1: What we observed from the hex dump
        if record.data.len() >= 40 {
            let field1 = reader.read_u32().unwrap(); // 0x0000E888
            let field2 = reader.read_u32().unwrap(); // 0x000148DA  
            let field3 = reader.read_u32().unwrap(); // 0x00002138
            let field4 = reader.read_u32().unwrap(); // 0x00002138
            let field5 = reader.read_u32().unwrap(); // 0x00001624
            let field6 = reader.read_u32().unwrap(); // 0x0000109C
            let field7 = reader.read_u32().unwrap(); // 0x0000109C
            let field8 = reader.read_u32().unwrap(); // 0x0000109C
            let field9 = reader.read_u32().unwrap(); // 0x00000000
            let field10 = reader.read_u32().unwrap(); // 0x00000000
            
            println!("    필드 1 (0x00): 0x{:08X} ({})", field1, field1);
            println!("    필드 2 (0x04): 0x{:08X} ({})", field2, field2);
            println!("    필드 3 (0x08): 0x{:08X} ({}) - 높이?", field3, field3);
            println!("    필드 4 (0x0C): 0x{:08X} ({}) - 높이?", field4, field4);
            println!("    필드 5 (0x10): 0x{:08X} ({}) - 왼쪽 여백?", field5, field5);
            println!("    필드 6 (0x14): 0x{:08X} ({}) - 위쪽 여백?", field6, field6);
            println!("    필드 7 (0x18): 0x{:08X} ({}) - 오른쪽 여백?", field7, field7);
            println!("    필드 8 (0x1C): 0x{:08X} ({}) - 아래쪽 여백?", field8, field8);
            println!("    필드 9 (0x20): 0x{:08X} ({})", field9, field9);
            println!("    필드 10 (0x24): 0x{:08X} ({})", field10, field10);
            
            // Convert HWPU to mm for better understanding
            println!();
            println!("  HWPU → mm 변환 (1 HWPU ≈ 0.01 mm):");
            println!("    높이: {} mm", field3 as f32 / 100.0);
            println!("    왼쪽 여백: {} mm", field5 as f32 / 100.0);
            println!("    위쪽 여백: {} mm", field6 as f32 / 100.0);
            println!("    오른쪽 여백: {} mm", field7 as f32 / 100.0);
            println!("    아래쪽 여백: {} mm", field8 as f32 / 100.0);
        }
    }
    
    // Check if there are more bytes
    if record.data.len() > 40 {
        println!();
        println!("📝 추가 데이터 ({} bytes):", record.data.len() - 40);
        let extra_data = &record.data[40..];
        print_hex_dump(extra_data, 4);
        
        // Try to parse as text
        if extra_data.len() >= 2 {
            println!();
            println!("🔤 텍스트 데이터 시도:");
            
            // Try UTF-16LE
            if extra_data.len() % 2 == 0 {
                let mut utf16_chars = Vec::new();
                for chunk in extra_data.chunks_exact(2) {
                    let char_value = u16::from_le_bytes([chunk[0], chunk[1]]);
                    utf16_chars.push(char_value);
                }
                
                if let Ok(text) = String::from_utf16(&utf16_chars) {
                    let clean_text = text.trim_end_matches('\0');
                    if !clean_text.is_empty() {
                        println!("  UTF-16LE 해석: \"{}\"", clean_text);
                    }
                }
            }
            
            // Try as length-prefixed UTF-16LE
            if extra_data.len() >= 2 {
                let mut reader = StreamReader::new(extra_data.to_vec());
                if let Ok(text_len) = reader.read_u16() {
                    println!("  길이 프리픽스: {} 문자", text_len);
                    
                    let expected_bytes = text_len as usize * 2;
                    if reader.remaining() >= expected_bytes {
                        if let Ok(text_bytes) = reader.read_bytes(expected_bytes) {
                            let mut utf16_chars = Vec::new();
                            for chunk in text_bytes.chunks_exact(2) {
                                let char_value = u16::from_le_bytes([chunk[0], chunk[1]]);
                                utf16_chars.push(char_value);
                            }
                            
                            if let Ok(text) = String::from_utf16(&utf16_chars) {
                                println!("  길이 기반 UTF-16LE: \"{}\"", text);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn print_hex_dump(data: &[u8], max_lines: usize) {
    for (i, chunk) in data.chunks(16).enumerate().take(max_lines) {
        print!("  {:04X}: ", i * 16);
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
        println!("  ... ({} more bytes)", data.len() - max_lines * 16);
    }
}