// Sample HWP file analyzer - analyzes /Users/indo/Downloads/sample.hwp
use hwpers::parser::record::{HwpTag, Record};
use hwpers::reader::CfbReader;
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sample HWP 파일 분석 ===\n");

    let file_path = "/Users/indo/Downloads/sample.hwp";
    let mut reader = CfbReader::from_file(file_path)?;

    // List all streams first
    println!("🗂️ CFB 스트림 목록:");
    reader.list_streams().iter().for_each(|stream| {
        println!("  - {}", stream);
    });
    println!();

    // Parse FileHeader to check compression and other properties
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;
    println!("📁 파일 헤더 정보:");
    println!(
        "  서명: {:?}",
        std::str::from_utf8(&header.signature[..17]).unwrap_or("Invalid")
    );
    println!("  버전: {}", header.version);
    println!("  압축 여부: {}", header.is_compressed());
    println!("  암호화 여부: {}", header.is_encrypted());
    println!();

    // Analyze DocInfo
    if reader.stream_exists("DocInfo") {
        let doc_info_data = reader.read_stream("DocInfo")?;
        println!("📋 DocInfo 스트림 크기: {} bytes", doc_info_data.len());

        // Try to decompress if needed and analyze
        let data = if header.is_compressed() {
            match decompress_stream(&doc_info_data) {
                Ok(decompressed) => {
                    println!("  압축 해제 후 크기: {} bytes", decompressed.len());
                    decompressed
                }
                Err(e) => {
                    println!("  압축 해제 실패: {}", e);
                    doc_info_data
                }
            }
        } else {
            doc_info_data
        };

        // Parse DocInfo records
        let mut reader = StreamReader::new(data);
        let mut doc_info_records = Vec::new();

        while reader.remaining() >= 8 {
            match Record::parse(&mut reader) {
                Ok(record) => {
                    doc_info_records.push(record);
                }
                Err(_) => break,
            }
        }

        println!("  DocInfo 레코드 수: {}", doc_info_records.len());
        println!();
    }

    // Analyze BodyText sections
    let mut section_idx = 0;

    loop {
        let section_name = format!("BodyText/Section{}", section_idx);
        if !reader.stream_exists(&section_name) {
            break;
        }

        println!("📄 {} 분석:", section_name);
        let section_data = reader.read_stream(&section_name)?;
        println!("  원본 크기: {} bytes", section_data.len());

        // Decompress if needed
        let data = if header.is_compressed() {
            match decompress_stream(&section_data) {
                Ok(decompressed) => {
                    println!("  압축 해제 후 크기: {} bytes", decompressed.len());
                    decompressed
                }
                Err(e) => {
                    println!("  압축 해제 실패: {}", e);
                    continue;
                }
            }
        } else {
            section_data
        };

        // Parse records and collect statistics
        let mut stream_reader = StreamReader::new(data);
        let mut record_count = 0;
        let mut tag_counts = HashMap::new();
        let mut special_records: Vec<(String, u64, Record)> = Vec::new();

        while stream_reader.remaining() >= 8 {
            let position = stream_reader.position();

            match Record::parse(&mut stream_reader) {
                Ok(record) => {
                    record_count += 1;
                    let tag = HwpTag::from_u16(record.tag_id());

                    // Count tags
                    *tag_counts.entry(record.tag_id()).or_insert(0) += 1;

                    // Collect special records
                    match tag {
                        Some(HwpTag::ParaRangeTag) => {
                            special_records.push((
                                "ParaRangeTag (하이퍼링크)".to_string(),
                                position,
                                record,
                            ));
                        }
                        Some(HwpTag::ParaText) => {
                            // Don't collect all ParaText, too many
                        }
                        Some(HwpTag::ParaCharShape) => {
                            // Don't collect all ParaCharShape, too many
                        }
                        Some(HwpTag::Table) => {
                            special_records.push(("Table".to_string(), position, record));
                        }
                        Some(HwpTag::ShapeComponentPicture) => {
                            special_records.push(("Picture".to_string(), position, record));
                        }
                        Some(HwpTag::HeaderFooter) => {
                            special_records.push(("HeaderFooter".to_string(), position, record));
                        }
                        _ => {
                            // Check for control records (tag >= 0x70)
                            if record.tag_id() >= 0x70 {
                                let control_name = format!("Control_0x{:04X}", record.tag_id());
                                special_records.push((control_name, position, record));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ 레코드 파싱 실패 at 0x{:08X}: {}", position, e);
                    break;
                }
            }
        }

        println!("  총 레코드 수: {}", record_count);

        // Print tag statistics
        println!("  📊 태그 통계 (상위 10개):");
        let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

        for (tag_id, count) in sorted_tags.iter().take(10) {
            let tag_name = HwpTag::from_u16(*tag_id)
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| format!("Unknown_0x{:04X}", tag_id));
            println!("    0x{:04X} ({}): {} 개", tag_id, tag_name, count);
        }

        // Analyze special records
        if !special_records.is_empty() {
            println!("  🎯 특별 레코드 상세 분석:");
            for (name, position, record) in &special_records {
                println!("    {} at 0x{:08X}:", name, position);
                println!("      크기: {} bytes", record.data.len());

                // Show hex dump for smaller records
                if record.data.len() <= 256 {
                    print_hex_dump(&record.data, 8);
                }

                // Try to parse specific types
                match HwpTag::from_u16(record.tag_id()) {
                    Some(HwpTag::ParaRangeTag) => {
                        analyze_hyperlink_record(record);
                    }
                    Some(HwpTag::Table) => {
                        analyze_table_record(record);
                    }
                    Some(HwpTag::ShapeComponentPicture) => {
                        analyze_picture_record(record);
                    }
                    _ => {
                        // Generic analysis
                        if record.tag_id() >= 0x70 {
                            analyze_control_record(record);
                        }
                    }
                }
                println!();
            }
        }

        section_idx += 1;
        println!();
    }

    if section_idx == 0 {
        println!("❌ BodyText 섹션을 찾을 수 없습니다.");
    } else {
        println!("✅ 총 {} 개의 섹션을 분석했습니다.", section_idx);
    }

    Ok(())
}

fn print_hex_dump(data: &[u8], max_lines: usize) {
    for (i, chunk) in data.chunks(16).enumerate().take(max_lines) {
        print!("        {:04X}: ", i * 16);
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
        println!("        ... ({} more bytes)", data.len() - max_lines * 16);
    }
}

fn analyze_hyperlink_record(record: &Record) {
    println!("      🔗 하이퍼링크 분석:");
    match hwpers::model::hyperlink::Hyperlink::from_record(record) {
        Ok(hyperlink) => {
            println!("        ✅ 파싱 성공:");
            println!("          표시 텍스트: \"{}\"", hyperlink.display_text);
            println!("          대상 URL: \"{}\"", hyperlink.target_url);
            println!("          유형: {:?}", hyperlink.hyperlink_type);
            println!(
                "          위치: {}, 길이: {}",
                hyperlink.start_position, hyperlink.length
            );
        }
        Err(e) => {
            println!("        ❌ 파싱 실패: {}", e);
        }
    }
}

fn analyze_table_record(record: &Record) {
    println!("      📊 표 분석:");
    println!("        데이터 크기: {} bytes", record.data.len());
    // Add more table-specific analysis here
}

fn analyze_picture_record(record: &Record) {
    println!("      🖼️ 그림 분석:");
    println!("        데이터 크기: {} bytes", record.data.len());
    // Add more picture-specific analysis here
}

fn analyze_control_record(record: &Record) {
    println!("      🎛️ 컨트롤 분석:");
    println!("        컨트롤 ID: 0x{:04X}", record.tag_id());
    println!("        데이터 크기: {} bytes", record.data.len());
}
