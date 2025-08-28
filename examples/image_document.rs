use hwpers::{HwpWriter, writer::style::{ImageOptions, ImageAlign, ImageFormat}};
use std::fs::File;
use std::io::Write;

/// Create a simple test PNG image
fn create_test_image() -> Vec<u8> {
    // Create a minimal 1x1 red PNG image
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,  // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,  // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,  // 8-bit RGB
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,  // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,  // Red pixel data
        0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,  // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating HWP document with images...\n");

    // Create test images
    let test_png = create_test_image();
    
    // Save test image to file for add_image test
    let mut file = File::create("test_image.png")?;
    file.write_all(&test_png)?;

    // Create a new HWP writer
    let mut writer = HwpWriter::new();

    // Add title
    writer.add_heading("이미지 예제 문서", 1)?;
    
    // Add image from file
    writer.add_heading("파일에서 이미지 추가", 2)?;
    writer.add_paragraph("다음은 파일에서 읽어온 이미지입니다:")?;
    writer.add_image("test_image.png")?;
    
    writer.add_paragraph("")?; // Empty line

    // Add image from bytes
    writer.add_heading("바이트 배열에서 이미지 추가", 2)?;
    writer.add_paragraph("메모리에서 직접 이미지를 추가합니다:")?;
    writer.add_image_from_bytes(&test_png, ImageFormat::Png)?;
    
    writer.add_paragraph("")?;

    // Add image with custom options
    writer.add_heading("커스텀 옵션으로 이미지 추가", 2)?;
    
    // Centered image with size
    writer.add_paragraph("가운데 정렬된 이미지 (50x50mm):")?;
    let centered_options = ImageOptions::new()
        .width(50)
        .height(50)
        .align(ImageAlign::Center)
        .caption("가운데 정렬된 이미지");
    writer.add_image_with_options(&test_png, ImageFormat::Png, &centered_options)?;
    
    writer.add_paragraph("")?;

    // Right-aligned image
    writer.add_paragraph("오른쪽 정렬된 이미지:")?;
    let right_options = ImageOptions::new()
        .width(30)
        .height(30)
        .align(ImageAlign::Right)
        .caption("오른쪽 정렬");
    writer.add_image_with_options(&test_png, ImageFormat::Png, &right_options)?;
    
    writer.add_paragraph("")?;

    // Inline image with text wrapping
    writer.add_heading("텍스트와 함께 배치", 2)?;
    writer.add_paragraph("이 단락은 이미지 앞에 있습니다.")?;
    
    let inline_options = ImageOptions::new()
        .width(40)
        .height(40)
        .align(ImageAlign::InlineWithText)
        .wrap_text(true)
        .caption("텍스트와 함께 배치된 이미지");
    writer.add_image_with_options(&test_png, ImageFormat::Png, &inline_options)?;
    
    writer.add_paragraph("이 단락은 이미지 뒤에 있으며, 실제 HWP에서는 이미지 주변으로 텍스트가 흐르게 됩니다.")?;

    // Multiple images
    writer.add_heading("여러 이미지", 2)?;
    writer.add_paragraph("같은 줄에 여러 이미지를 배치할 수 있습니다:")?;
    
    for i in 1..=3 {
        let options = ImageOptions::new()
            .width(20)
            .height(20)
            .caption(&format!("이미지 {i}"));
        writer.add_image_with_options(&test_png, ImageFormat::Png, &options)?;
    }

    // Save to file
    writer.save_to_file("image_document.hwp")?;

    println!("✅ Created image_document.hwp with various image examples");
    println!("\nDocument contains:");
    println!("- Images from file");
    println!("- Images from bytes");
    println!("- Images with different alignments");
    println!("- Images with captions");
    println!("- Multiple images");
    
    // Clean up test image
    std::fs::remove_file("test_image.png").ok();

    Ok(())
}