use hwpers::{
    writer::style::{ImageAlign, ImageFormat, ImageOptions},
    HwpWriter,
};
use std::fs::File;
use std::io::Write;

/// Create a minimal test PNG image
fn create_test_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // Red pixel data
        0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

/// Create a minimal test JPEG image
fn create_test_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, // JPEG SOI and APP0 marker
        0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, // JFIF header
        0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00,
        0x43, // Quantization table
        // ... minimal JPEG data for tests
        0xFF, 0xD9, // EOI marker
    ]
}

#[test]
fn test_image_format_detection() {
    let png_data = create_test_png();
    let jpeg_data = create_test_jpeg();

    assert_eq!(ImageFormat::from_bytes(&png_data), Some(ImageFormat::Png));
    assert_eq!(ImageFormat::from_bytes(&jpeg_data), Some(ImageFormat::Jpeg));

    // Test invalid data
    assert_eq!(ImageFormat::from_bytes(&[0x00, 0x00]), None);
    assert_eq!(ImageFormat::from_bytes(&[]), None);
}

#[test]
fn test_add_image_from_file() {
    let mut writer = HwpWriter::new();

    // Create temporary test image
    let test_png = create_test_png();
    let mut file = File::create("test_image_tmp.png").unwrap();
    file.write_all(&test_png).unwrap();

    // Add image from file
    writer.add_image("test_image_tmp.png").unwrap();

    let document = writer.document();

    // Check that BinData was added
    assert_eq!(document.doc_info.bin_data.len(), 1);
    let bin_data = &document.doc_info.bin_data[0];
    assert_eq!(bin_data.extension, "png");
    assert_eq!(bin_data.data, test_png);

    // Check that a paragraph with picture control was added
    assert_eq!(document.body_texts[0].sections[0].paragraphs.len(), 1);
    let para = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(para.ctrl_header.is_some());
    assert_eq!(para.control_mask, 2); // Control header present (0x02)
    assert!(para.text.is_none()); // Picture control paragraph has no text

    // Clean up
    std::fs::remove_file("test_image_tmp.png").ok();
}

#[test]
fn test_add_image_from_bytes() {
    let mut writer = HwpWriter::new();
    let test_png = create_test_png();

    writer
        .add_image_from_bytes(&test_png, ImageFormat::Png)
        .unwrap();

    let document = writer.document();

    // Check BinData
    assert_eq!(document.doc_info.bin_data.len(), 1);
    assert_eq!(document.doc_info.bin_data[0].extension, "png");

    // Check picture control paragraph
    let para = &document.body_texts[0].sections[0].paragraphs[0];
    assert!(para.ctrl_header.is_some());
    assert!(para.text.is_none());
}

#[test]
fn test_image_with_options() {
    let mut writer = HwpWriter::new();
    let test_png = create_test_png();

    let options = ImageOptions::new()
        .width(100)
        .height(80)
        .align(ImageAlign::Center)
        .caption("Test Image");

    writer
        .add_image_with_options(&test_png, ImageFormat::Png, &options)
        .unwrap();

    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;

    // Should have 2 paragraphs: picture control + caption
    assert_eq!(paragraphs.len(), 2);

    // Check picture control paragraph
    assert!(paragraphs[0].ctrl_header.is_some());
    assert!(paragraphs[0].text.is_none());

    // Check caption paragraph
    let caption_text = &paragraphs[1].text.as_ref().unwrap().content;
    assert!(caption_text.contains("그림: Test Image"));
}

#[test]
fn test_multiple_images() {
    let mut writer = HwpWriter::new();
    let test_png = create_test_png();

    // Add 3 images
    for i in 0..3 {
        let options = ImageOptions::new().caption(&format!("Image {}", i + 1));
        writer
            .add_image_with_options(&test_png, ImageFormat::Png, &options)
            .unwrap();
    }

    let document = writer.document();

    // Check BinData entries
    assert_eq!(document.doc_info.bin_data.len(), 3);
    for (i, bin_data) in document.doc_info.bin_data.iter().enumerate() {
        let expected_id = (i + 1) as u16; // IDs start from 1
        assert_eq!(bin_data.abs_name, format!("image{}.png", expected_id));
        assert_eq!(bin_data.bin_id, expected_id);
    }

    // Check paragraphs (3 images + 3 captions = 6 paragraphs)
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;
    assert_eq!(paragraphs.len(), 6);
}

#[test]
fn test_image_alignment() {
    let mut writer = HwpWriter::new();
    let test_png = create_test_png();

    // Test different alignments
    let alignments = [
        (ImageAlign::Left, ""),
        (ImageAlign::Center, "      "),
        (ImageAlign::Right, "            "),
    ];

    for (align, _expected_prefix) in alignments.iter() {
        let options = ImageOptions::new().align(*align);
        writer
            .add_image_with_options(&test_png, ImageFormat::Png, &options)
            .unwrap();
    }

    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;

    // Should have 3 picture control paragraphs
    assert_eq!(paragraphs.len(), 3);

    // All should be picture controls
    for para in paragraphs {
        assert!(para.ctrl_header.is_some());
        assert!(para.text.is_none());
    }
}

#[test]
fn test_image_extension_mapping() {
    assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
    assert_eq!(ImageFormat::Png.extension(), "png");
    assert_eq!(ImageFormat::Bmp.extension(), "bmp");
    assert_eq!(ImageFormat::Gif.extension(), "gif");
}

#[test]
fn test_mixed_content_with_images() {
    let mut writer = HwpWriter::new();
    let test_png = create_test_png();

    writer.add_heading("Images in Document", 1).unwrap();
    writer.add_paragraph("Here is an image:").unwrap();

    let options = ImageOptions::new()
        .width(50)
        .height(50)
        .caption("Sample Image");
    writer
        .add_image_with_options(&test_png, ImageFormat::Png, &options)
        .unwrap();

    writer.add_paragraph("Text after the image.").unwrap();

    let document = writer.document();
    let paragraphs = &document.body_texts[0].sections[0].paragraphs;

    // Should have: heading, text, picture control, caption, text = 5 paragraphs
    assert_eq!(paragraphs.len(), 5);
    assert!(paragraphs[0]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("Images in Document"));
    assert!(paragraphs[1]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("Here is an image"));
    assert!(paragraphs[2].ctrl_header.is_some()); // Picture control
    assert!(paragraphs[3]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("그림: Sample Image"));
    assert!(paragraphs[4]
        .text
        .as_ref()
        .unwrap()
        .content
        .contains("Text after the image"));
}
