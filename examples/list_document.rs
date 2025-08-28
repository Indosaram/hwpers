use hwpers::{HwpWriter, writer::style::ListType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with lists...\n");

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("목록 예제 문서", 1)?;
    
    // Simple bullet list
    writer.add_heading("글머리 기호 목록", 2)?;
    writer.add_list(
        &[
            "첫 번째 항목",
            "두 번째 항목",
            "세 번째 항목",
        ],
        ListType::Bullet,
    )?;
    
    writer.add_paragraph("")?; // Empty line

    // Numbered list
    writer.add_heading("번호 목록", 2)?;
    writer.add_list(
        &[
            "항목 하나",
            "항목 둘",
            "항목 셋",
            "항목 넷",
        ],
        ListType::Numbered,
    )?;
    
    writer.add_paragraph("")?;

    // Alphabetic list
    writer.add_heading("알파벳 목록", 2)?;
    writer.add_list(
        &[
            "Apple",
            "Banana",
            "Cherry",
            "Date",
        ],
        ListType::Alphabetic,
    )?;
    
    writer.add_paragraph("")?;

    // Korean list
    writer.add_heading("한글 목록", 2)?;
    writer.add_list(
        &[
            "가을 하늘",
            "나비 날개",
            "다람쥐 집",
            "라일락 꽃",
        ],
        ListType::Korean,
    )?;
    
    writer.add_paragraph("")?;

    // Roman numeral list
    writer.add_heading("로마 숫자 목록", 2)?;
    writer.add_list(
        &[
            "Introduction",
            "Background",
            "Methodology",
            "Results",
            "Conclusion",
        ],
        ListType::Roman,
    )?;
    
    writer.add_paragraph("")?;

    // Manual list building with custom items
    writer.add_heading("수동 목록 구성", 2)?;
    writer.start_list(ListType::Numbered)?;
    
    writer.add_list_item("기본 항목")?;
    
    // Nested list
    writer.start_nested_list(ListType::Bullet)?;
    writer.add_list_item("중첩된 첫 번째 항목")?;
    writer.add_list_item("중첩된 두 번째 항목")?;
    writer.end_list()?; // End nested list
    
    writer.add_list_item("다시 메인 목록으로")?;
    
    // Another nested list
    writer.start_nested_list(ListType::Alphabetic)?;
    writer.add_list_item("또 다른 중첩 목록")?;
    writer.add_list_item("알파벳 순서로")?;
    writer.end_list()?; // End nested list
    
    writer.add_list_item("마지막 메인 항목")?;
    writer.end_list()?; // End main list

    // Save to file
    writer.save_to_file("list_document.hwp")?;

    println!("✅ Created list_document.hwp with various list types");
    println!("\nDocument contains:");
    println!("- Bullet lists");
    println!("- Numbered lists");
    println!("- Alphabetic lists");
    println!("- Korean numbering lists");
    println!("- Roman numeral lists");
    println!("- Nested lists");

    Ok(())
}