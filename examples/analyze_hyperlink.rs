#![allow(clippy::unused_io_amount)]
#![allow(clippy::if_same_then_else)]

use hwpers::HwpReader;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 하이퍼링크 HWP 파일 구조 분석 ===\n");

    // Parse both generated and test hyperlink documents
    println!("🔍 Writer로 생성된 파일 분석:");
    analyze_document("hyperlink_document.hwp")?;

    println!("\n🔍 테스트 파일 분석:");
    analyze_document("test-files/hyperlink_document.hwp")?;

    Ok(())
}

fn analyze_document(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  파일: {}", file_path);

    // Parse the hyperlink document
    let document = HwpReader::from_file(file_path)?;

    println!("📄 문서 정보:");
    println!("  - 섹션 수: {}", document.body_texts.len());

    if let Some(body_text) = document.body_texts.first() {
        println!(
            "  - 첫 번째 섹션의 문단 수: {}",
            body_text.sections[0].paragraphs.len()
        );

        // Analyze hyperlinks in paragraphs
        let mut total_hyperlinks = 0;
        let mut hyperlink_types = std::collections::HashMap::new();

        for (i, paragraph) in body_text.sections[0].paragraphs.iter().enumerate() {
            if !paragraph.hyperlinks.is_empty() {
                total_hyperlinks += paragraph.hyperlinks.len();

                println!("\n📎 문단 {} 하이퍼링크 정보:", i + 1);
                for (j, hyperlink) in paragraph.hyperlinks.iter().enumerate() {
                    println!(
                        "  링크 {}: \"{}\" -> \"{}\"",
                        j + 1,
                        hyperlink.display_text,
                        hyperlink.target_url
                    );
                    println!("    유형: {:?}", hyperlink.hyperlink_type);
                    println!("    표시 방식: {:?}", hyperlink.display_mode);
                    println!("    텍스트 색상: 0x{:06X}", hyperlink.text_color);
                    println!("    밑줄: {}", hyperlink.underline);
                    println!(
                        "    시작 위치: {}, 길이: {}",
                        hyperlink.start_position, hyperlink.length
                    );

                    // Count by type
                    *hyperlink_types
                        .entry(format!("{:?}", hyperlink.hyperlink_type))
                        .or_insert(0) += 1;

                    if let Some(tooltip) = &hyperlink.tooltip {
                        println!("    툴팁: \"{}\"", tooltip);
                    }
                }

                // Show paragraph text if available
                if let Some(text) = &paragraph.text {
                    println!("  문단 텍스트: \"{:?}\"", text);
                }
            }
        }

        println!("\n📊 하이퍼링크 통계:");
        println!("  - 총 하이퍼링크 수: {}", total_hyperlinks);
        for (link_type, count) in hyperlink_types {
            println!("  - {}: {} 개", link_type, count);
        }
    }

    // Now let's analyze the raw binary structure
    println!("\n🔍 원시 바이너리 구조 분석:");
    analyze_raw_structure(file_path)?;

    Ok(())
}

fn analyze_raw_structure(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This is a simplified analysis. In practice, we'd need to parse the CFB structure
    println!("  파일: {}", file_path);

    let mut file = File::open(file_path)?;
    let mut buffer = vec![0; 1024]; // Read first 1KB for analysis
    file.read(&mut buffer)?;

    // Look for our known patterns
    println!("  CFB 헤더 확인:");
    if buffer.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        println!("    ✅ 올바른 CFB (Compound File Binary) 헤더 발견");
    } else {
        println!("    ❌ CFB 헤더를 찾을 수 없습니다");
    }

    // Search for hyperlink control ID pattern in the file
    file.seek(SeekFrom::Start(0))?;
    let mut full_buffer = Vec::new();
    file.read_to_end(&mut full_buffer)?;

    // Look for hyperlink control ID: 0x20687367 ('gsh ' in little-endian)
    let hyperlink_ctrl_id = [0x67, 0x73, 0x68, 0x20]; // 'gsh ' in little-endian bytes
    let mut hyperlink_positions = Vec::new();

    for (i, window) in full_buffer.windows(4).enumerate() {
        if window == hyperlink_ctrl_id {
            hyperlink_positions.push(i);
        }
    }

    println!("  하이퍼링크 컨트롤 ID 검색 결과:");
    if hyperlink_positions.is_empty() {
        println!("    🔍 하이퍼링크 컨트롤 ID (0x20687367, 'gsh ')를 찾을 수 없습니다");
    } else {
        println!(
            "    ✅ {} 개의 하이퍼링크 컨트롤 ID 발견:",
            hyperlink_positions.len()
        );
        for (i, pos) in hyperlink_positions.iter().enumerate() {
            println!(
                "      #{}: 파일 오프셋 0x{:08X} ({} bytes)",
                i + 1,
                pos,
                pos
            );

            // Show some context around the control ID
            let start = pos.saturating_sub(16);
            let end = std::cmp::min(pos + 32, full_buffer.len());
            let context = &full_buffer[start..end];

            print!("      컨텍스트: ");
            for (j, &byte) in context.iter().enumerate() {
                if start + j == *pos {
                    print!("[{:02X}] ", byte);
                } else if start + j >= *pos && start + j < pos + 4 {
                    print!("{:02X} ", byte);
                } else {
                    print!("{:02X} ", byte);
                }
            }
            println!();
        }
    }

    // Look for record tag 0x54 patterns
    println!("  레코드 태그 0x54 (ParaRangeTag) 검색:");
    let mut tag_54_positions = Vec::new();

    // HWP records have a specific structure, but let's do a simple search first
    for (i, window) in full_buffer.windows(4).enumerate() {
        // Look for little-endian 0x54 followed by reasonable level (0x00) and size
        if window[0] == 0x54 && window[1] == 0x00 {
            tag_54_positions.push(i);
        }
    }

    if tag_54_positions.is_empty() {
        println!("    🔍 레코드 태그 0x54를 찾을 수 없습니다");
    } else {
        println!(
            "    ✅ {} 개의 가능한 0x54 레코드 발견:",
            tag_54_positions.len()
        );
        for (i, pos) in tag_54_positions.iter().take(10).enumerate() {
            // Show first 10 only
            println!("      #{}: 파일 오프셋 0x{:08X}", i + 1, pos);

            if pos + 8 < full_buffer.len() {
                let header_bytes = &full_buffer[*pos..*pos + 8];
                let tag = u16::from_le_bytes([header_bytes[0], header_bytes[1]]);
                let level = u16::from_le_bytes([header_bytes[2], header_bytes[3]]);
                let size = u32::from_le_bytes([
                    header_bytes[4],
                    header_bytes[5],
                    header_bytes[6],
                    header_bytes[7],
                ]);
                println!(
                    "        태그: 0x{:04X}, 레벨: {}, 크기: {} bytes",
                    tag, level, size
                );
            }
        }
    }

    // Look for UTF-16 encoded hyperlink text
    println!("  UTF-16 하이퍼링크 텍스트 검색:");
    let search_terms = vec![
        "구글 방문하기",
        "GitHub 저장소",
        "Rust 공식 사이트",
        "문의하기",
        "특별한 링크",
    ];

    for term in search_terms {
        let utf16_bytes = term
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect::<Vec<u8>>();

        for (i, window) in full_buffer.windows(utf16_bytes.len()).enumerate() {
            if window == utf16_bytes {
                println!("    ✅ \"{}\" 발견: 오프셋 0x{:08X}", term, i);
                break;
            }
        }
    }

    println!("  총 파일 크기: {} bytes", full_buffer.len());

    Ok(())
}
