// Detailed analysis of sample.hwp focusing on specific controls
use hwpers::parser::record::{HwpTag, Record};
use hwpers::reader::CfbReader;
use hwpers::reader::StreamReader;
use hwpers::utils::compression::decompress_stream;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 상세 Sample HWP 파일 분석 ===\n");

    let file_path = "/Users/indo/Downloads/sample.hwp";
    let mut reader = CfbReader::from_file(file_path)?;

    // Parse FileHeader
    let header_data = reader.read_stream("FileHeader")?;
    let header = hwpers::parser::header::FileHeader::parse(header_data)?;

    // Analyze BinData (images)
    analyze_bindata(&mut reader)?;

    // Analyze BodyText in detail
    analyze_bodytext_detailed(&mut reader, header.is_compressed())?;

    Ok(())
}

fn analyze_bindata(
    reader: &mut CfbReader<std::fs::File>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️ BinData 분석:");

    // List all BinData streams
    let streams = reader.list_streams();
    let bindata_streams: Vec<_> = streams
        .iter()
        .filter(|s| s.starts_with("/BinData/"))
        .collect();

    if bindata_streams.is_empty() {
        println!("  BinData 스트림이 없습니다.");
        return Ok(());
    }

    for stream_name in &bindata_streams {
        let data = reader.read_stream(&stream_name[1..])?; // Remove leading /
        println!("  {}: {} bytes", stream_name, data.len());

        // Analyze file header
        if data.len() >= 4 {
            let magic = &data[0..4];
            let file_type = match magic {
                [0xFF, 0xD8, 0xFF, ..] => "JPEG",
                [0x89, b'P', b'N', b'G'] => "PNG",
                [b'G', b'I', b'F', b'8'] => "GIF",
                [0x42, 0x4D, ..] => "BMP",
                _ => {
                    println!(
                        "    매직 바이트: {:02X} {:02X} {:02X} {:02X}",
                        magic[0], magic[1], magic[2], magic[3]
                    );
                    "Unknown"
                }
            };
            println!("    파일 타입: {}", file_type);

            // For JPEG, try to extract more info
            if file_type == "JPEG" {
                analyze_jpeg(&data);
            }
        }
    }
    println!();
    Ok(())
}

fn analyze_jpeg(data: &[u8]) {
    // Basic JPEG analysis - look for EXIF data
    if data.len() < 20 {
        return;
    }

    let mut offset = 2; // Skip SOI marker
    while offset + 4 < data.len() {
        if data[offset] != 0xFF {
            break;
        }

        let marker = data[offset + 1];
        let segment_length = ((data[offset + 2] as u16) << 8) | (data[offset + 3] as u16);

        match marker {
            0xE0 => println!("      JFIF 세그먼트 발견 (길이: {})", segment_length),
            0xE1 => {
                println!("      EXIF 세그먼트 발견 (길이: {})", segment_length);
                // Check for EXIF identifier
                if offset + 10 < data.len() && &data[offset + 4..offset + 8] == b"Exif" {
                    println!("        EXIF 데이터 확인됨");
                }
            }
            0xDB => println!("      양자화 테이블 (길이: {})", segment_length),
            0xC0 => println!("      프레임 헤더 (길이: {})", segment_length),
            0xDA => {
                println!("      이미지 데이터 시작");
                break;
            }
            _ => {}
        }

        offset += 2 + segment_length as usize;

        if offset >= data.len() {
            break;
        }
    }
}

fn analyze_bodytext_detailed(
    reader: &mut CfbReader<std::fs::File>,
    is_compressed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 BodyText 상세 분석:");

    let section_data = reader.read_stream("BodyText/Section0")?;

    // Decompress if needed
    let data = if is_compressed {
        decompress_stream(&section_data)?
    } else {
        section_data
    };

    // Parse records
    let mut stream_reader = StreamReader::new(data);
    let mut controls = HashMap::new();
    let mut paragraph_count = 0;

    while stream_reader.remaining() >= 8 {
        let position = stream_reader.position();

        match Record::parse(&mut stream_reader) {
            Ok(record) => {
                let tag = HwpTag::from_u16(record.tag_id());

                match tag {
                    Some(HwpTag::HeaderFooter) => {
                        analyze_header_footer_record(&record, position);
                    }
                    Some(HwpTag::Table) => {
                        analyze_table_record_detailed(&record, position);
                    }
                    Some(HwpTag::ShapeComponentPicture) => {
                        analyze_picture_control(&record, position);
                    }
                    Some(HwpTag::ParaHeader) => {
                        paragraph_count += 1;
                        if paragraph_count <= 5 {
                            // Only show first few
                            analyze_para_header(&record, position);
                        }
                    }
                    _ => {
                        // Count all control types
                        *controls.entry(record.tag_id()).or_insert(0) += 1;
                    }
                }
            }
            Err(_) => break,
        }
    }

    println!("\n  📊 컨트롤 요약:");
    println!("    문단 수: {}", paragraph_count);
    for (tag_id, count) in controls.iter() {
        if *count > 0 {
            let tag_name = HwpTag::from_u16(*tag_id)
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| format!("Unknown_0x{:04X}", tag_id));
            println!("    0x{:04X} ({}): {} 개", tag_id, tag_name, count);
        }
    }

    Ok(())
}

fn analyze_header_footer_record(record: &Record, position: u64) {
    println!("\n  📋 HeaderFooter 레코드 at 0x{:08X}:", position);
    println!("    크기: {} bytes", record.data.len());

    if record.data.len() >= 32 {
        let mut reader = StreamReader::new(record.data.clone());

        // Try to parse header/footer structure
        match (|| -> Result<(), Box<dyn std::error::Error>> {
            let _unknown1 = reader.read_u32()?;
            let _unknown2 = reader.read_u32()?;
            let width = reader.read_u32()?;
            let height = reader.read_u32()?;
            let margin_left = reader.read_u32()?;
            let margin_top = reader.read_u32()?;
            let margin_right = reader.read_u32()?;
            let margin_bottom = reader.read_u32()?;

            println!("      크기: {}x{} HWPU", width, height);
            println!(
                "      여백: L:{} T:{} R:{} B:{} HWPU",
                margin_left, margin_top, margin_right, margin_bottom
            );

            Ok(())
        })() {
            Ok(_) => {}
            Err(_) => {
                println!("      파싱 실패 - 헥스 덤프:");
                print_hex_dump(&record.data, 4);
            }
        }
    }
}

fn analyze_table_record_detailed(record: &Record, position: u64) {
    println!("\n  📊 Table 레코드 at 0x{:08X}:", position);
    println!("    크기: {} bytes", record.data.len());

    if !record.data.is_empty() {
        let mut reader = StreamReader::new(record.data.clone());

        // Try to extract table properties
        match (|| -> Result<(), Box<dyn std::error::Error>> {
            if reader.remaining() >= 4 {
                let table_flag = reader.read_u32()?;
                println!("      테이블 플래그: 0x{:08X}", table_flag);
            }

            if reader.remaining() >= 2 {
                let row_count = reader.read_u16()?;
                println!("      행 수: {}", row_count);
            }

            if reader.remaining() >= 2 {
                let col_count = reader.read_u16()?;
                println!("      열 수: {}", col_count);
            }

            Ok(())
        })() {
            Ok(_) => {}
            Err(_) => {
                println!("      기본 테이블 정보 파싱 실패");
            }
        }
    }
}

fn analyze_picture_control(record: &Record, position: u64) {
    println!("\n  🖼️ Picture 컨트롤 at 0x{:08X}:", position);
    println!("    크기: {} bytes", record.data.len());

    if record.data.len() >= 8 {
        let mut reader = StreamReader::new(record.data.clone());

        match (|| -> Result<(), Box<dyn std::error::Error>> {
            let bin_id = reader.read_u16()?;
            println!("      BinData ID: {}", bin_id);

            let _reserved = reader.read_u16()?;
            let width = reader.read_u32()?;
            let height = reader.read_u32()?;

            println!("      크기: {}x{} HWPU", width, height);

            Ok(())
        })() {
            Ok(_) => {}
            Err(_) => {
                println!("      Picture 정보 파싱 실패");
            }
        }
    }
}

fn analyze_para_header(record: &Record, position: u64) {
    println!("\n  📝 ParaHeader at 0x{:08X}:", position);
    println!("    크기: {} bytes", record.data.len());

    if record.data.len() >= 12 {
        let mut reader = StreamReader::new(record.data.clone());

        match (|| -> Result<(), Box<dyn std::error::Error>> {
            let text_count = reader.read_u32()?;
            let control_mask = reader.read_u32()?;
            let para_shape_id = reader.read_u16()?;
            let style_id = reader.read_u8()?;
            let _div_type = reader.read_u8()?;

            println!("      텍스트 길이: {}", text_count);
            println!("      컨트롤 마스크: 0x{:08X}", control_mask);
            println!("      문단 모양 ID: {}", para_shape_id);
            println!("      스타일 ID: {}", style_id);

            Ok(())
        })() {
            Ok(_) => {}
            Err(_) => {
                println!("      ParaHeader 파싱 실패");
            }
        }
    }
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
