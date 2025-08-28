// Debug tool for analyzing HWP records
use hwpers::reader::CfbReader;
use hwpers::parser::record::{Record, HwpTag};
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HWP 레코드 디버깅 ===\n");

    let mut reader = CfbReader::from_file("hyperlink_document.hwp")?;
    
    // Read BodyText/Section0
    let section_data = reader.read_stream("BodyText/Section0")?;
    println!("BodyText/Section0 크기: {} bytes", section_data.len());
    
    // Parse FileHeader to check compression
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;
    println!("압축 여부: {}", header.is_compressed());
    
    // Decompress if needed
    let data = if header.is_compressed() {
        let decompressed = decompress_stream(&section_data)?;
        println!("압축 해제 후 크기: {} bytes", decompressed.len());
        decompressed
    } else {
        section_data
    };
    
    // Parse records manually
    let mut reader = StreamReader::new(data);
    let mut record_count = 0;
    let mut hyperlink_records = Vec::new();
    
    println!("\n🔍 레코드 분석:");
    
    while reader.remaining() >= 8 { // At least header size
        let position = reader.position();
        
        match Record::parse(&mut reader) {
            Ok(record) => {
                record_count += 1;
                let tag = HwpTag::from_u16(record.tag_id());
                
                if record_count <= 20 || tag == Some(HwpTag::ParaRangeTag) {
                    println!("  레코드 #{}: 위치 0x{:08X}", record_count, position);
                    println!("    태그: 0x{:04X} ({:?})", record.tag_id(), tag);
                    println!("    레벨: {}", record.header.level);
                    println!("    크기: {} bytes", record.header.size);
                    
                    if record.header.size <= 1024 { // Show hex dump for small records
                        let data = &record.data;
                        print!("    데이터: ");
                        for (i, &byte) in data.iter().take(32).enumerate() {
                            if i > 0 && i % 16 == 0 {
                                print!("\n            ");
                            }
                            print!("{:02X} ", byte);
                        }
                        if data.len() > 32 {
                            print!("...");
                        }
                        println!();
                    }
                    
                    if tag == Some(HwpTag::ParaRangeTag) {
                        hyperlink_records.push((position, record));
                    }
                }
                
                if record_count > 100 && tag != Some(HwpTag::ParaRangeTag) {
                    break; // Limit output
                }
            }
            Err(e) => {
                println!("  ❌ 레코드 파싱 실패 at 0x{:08X}: {}", position, e);
                break;
            }
        }
    }
    
    println!("\n📊 요약:");
    println!("  총 레코드 수: {} (스캔된 개수)", record_count);
    println!("  ParaRangeTag (0x54) 레코드 수: {}", hyperlink_records.len());
    
    // Detailed analysis of ParaRangeTag records
    if !hyperlink_records.is_empty() {
        println!("\n🎯 ParaRangeTag 레코드 상세 분석:");
        for (i, (position, record)) in hyperlink_records.iter().enumerate() {
            println!("  ParaRangeTag #{}: 위치 0x{:08X}", i + 1, position);
            
            // Try to parse as hyperlink
            match hwpers::model::hyperlink::Hyperlink::from_record(record) {
                Ok(hyperlink) => {
                    println!("    ✅ 하이퍼링크 파싱 성공:");
                    println!("      표시 텍스트: \"{}\"", hyperlink.display_text);
                    println!("      대상 URL: \"{}\"", hyperlink.target_url);
                    println!("      유형: {:?}", hyperlink.hyperlink_type);
                    println!("      위치: {}, 길이: {}", hyperlink.start_position, hyperlink.length);
                }
                Err(e) => {
                    println!("    ❌ 하이퍼링크 파싱 실패: {}", e);
                    
                    // Show raw data for debugging
                    let data = &record.data;
                    println!("    원시 데이터 ({} bytes):", data.len());
                    for (i, chunk) in data.chunks(16).enumerate().take(8) {
                        print!("      {:04X}: ", i * 16);
                        for &byte in chunk {
                            print!("{:02X} ", byte);
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
                    if data.len() > 128 {
                        println!("      ... ({} more bytes)", data.len() - 128);
                    }
                }
            }
        }
    }
    
    Ok(())
}