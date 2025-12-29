use crate::error::Result;
use crate::model::HwpDocument;
use crate::utils::encoding::string_to_utf16le;
use byteorder::{LittleEndian, WriteBytesExt};
use cfb::CompoundFile;
use flate2::{write::DeflateEncoder, Compression};
use std::io::{Cursor, Write};

/// Serialize an HWP document to bytes
pub fn serialize_document(document: &HwpDocument) -> Result<Vec<u8>> {
    // Create CFB file from scratch (no template dependency)
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);

    // Use CFB version 3 (512-byte sectors) for HWP compatibility
    let mut cfb = CompoundFile::create_with_version(cfb::Version::V3, cursor)
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;

    // Create required storages
    cfb.create_storage("/BodyText")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    cfb.create_storage("/DocOptions")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    cfb.create_storage("/Scripts")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;

    // Create and write FileHeader stream (256 bytes, uncompressed)
    let header_data = serialize_file_header(&document.header)?;
    let mut header_stream = cfb
        .create_stream("/FileHeader")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    header_stream.write_all(&header_data)?;
    drop(header_stream);

    // Serialize and write DocInfo stream
    let doc_info_data = serialize_doc_info(&document.doc_info)?;
    let final_doc_info = if document.header.is_compressed() {
        compress_data(&doc_info_data)?
    } else {
        doc_info_data
    };
    let mut doc_info_stream = cfb
        .create_stream("/DocInfo")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    doc_info_stream.write_all(&final_doc_info)?;
    drop(doc_info_stream);

    // Serialize BodyText sections
    for (i, body_text) in document.body_texts.iter().enumerate() {
        let section_data = serialize_body_text(body_text)?;
        let final_section = if document.header.is_compressed() {
            compress_data(&section_data)?
        } else {
            section_data
        };

        let section_path = format!("/BodyText/Section{i}");
        let mut section_stream = cfb
            .create_stream(&section_path)
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
        section_stream.write_all(&final_section)?;
        drop(section_stream);
    }

    // Create BinData storage and streams if there are images
    if !document.doc_info.bin_data.is_empty() {
        cfb.create_storage("/BinData")
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;

        for bin_data in &document.doc_info.bin_data {
            let stream_name = format!("/BinData/BIN{:04X}.{}", bin_data.bin_id, bin_data.extension);

            // Compress binary data if document uses compression
            let final_data = if document.header.is_compressed() {
                compress_data(&bin_data.data)?
            } else {
                bin_data.data.clone()
            };

            let mut stream = cfb
                .create_stream(&stream_name)
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            stream.write_all(&final_data)?;
            drop(stream);
        }
    }

    // Create PrvText stream (preview text)
    let prv_text = create_preview_text(document)?;
    let mut prv_stream = cfb
        .create_stream("/PrvText")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    prv_stream.write_all(&prv_text)?;
    drop(prv_stream);

    // Create PrvImage stream (empty but required for compatibility)
    let mut prv_image_stream = cfb
        .create_stream("/PrvImage")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    prv_image_stream.write_all(&[])?;
    drop(prv_image_stream);

    // Create DocOptions/_LinkDoc stream
    let doc_options = create_doc_options()?;
    let mut options_stream = cfb
        .create_stream("/DocOptions/_LinkDoc")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    options_stream.write_all(&doc_options)?;
    drop(options_stream);

    // Create Scripts/JScriptVersion stream (uncompressed, matching hwplib blank.hwp)
    // 8 bytes: version 1 in little-endian
    let jscript_version: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut script_ver_stream = cfb
        .create_stream("/Scripts/JScriptVersion")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    script_ver_stream.write_all(&jscript_version)?;
    drop(script_ver_stream);

    // Create Scripts/DefaultJScript stream (uncompressed, matching hwplib blank.hwp)
    // 272 bytes: UTF-16LE encoded JavaScript with standard HWP document bindings
    #[rustfmt::skip]
    let default_jscript: [u8; 272] = [
        0x4F, 0x00, 0x00, 0x00, 0x76, 0x00, 0x61, 0x00, 0x72, 0x00, 0x20, 0x00, 0x44, 0x00, 0x6F, 0x00,
        0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x73, 0x00, 0x20, 0x00,
        0x3D, 0x00, 0x20, 0x00, 0x58, 0x00, 0x48, 0x00, 0x77, 0x00, 0x70, 0x00, 0x44, 0x00, 0x6F, 0x00,
        0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x73, 0x00, 0x3B, 0x00,
        0x0D, 0x00, 0x0A, 0x00, 0x76, 0x00, 0x61, 0x00, 0x72, 0x00, 0x20, 0x00, 0x44, 0x00, 0x6F, 0x00,
        0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x20, 0x00, 0x3D, 0x00,
        0x20, 0x00, 0x44, 0x00, 0x6F, 0x00, 0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00,
        0x74, 0x00, 0x73, 0x00, 0x2E, 0x00, 0x41, 0x00, 0x63, 0x00, 0x74, 0x00, 0x69, 0x00, 0x76, 0x00,
        0x65, 0x00, 0x5F, 0x00, 0x58, 0x00, 0x48, 0x00, 0x77, 0x00, 0x70, 0x00, 0x44, 0x00, 0x6F, 0x00,
        0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x3B, 0x00, 0x0D, 0x00,
        0x0A, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x66, 0x00, 0x75, 0x00, 0x6E, 0x00, 0x63, 0x00, 0x74, 0x00,
        0x69, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x20, 0x00, 0x4F, 0x00, 0x6E, 0x00, 0x44, 0x00, 0x6F, 0x00,
        0x63, 0x00, 0x75, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x5F, 0x00, 0x4E, 0x00,
        0x65, 0x00, 0x77, 0x00, 0x28, 0x00, 0x29, 0x00, 0x0D, 0x00, 0x0A, 0x00, 0x7B, 0x00, 0x0D, 0x00,
        0x0A, 0x00, 0x09, 0x00, 0x2F, 0x00, 0x2F, 0x00, 0x74, 0x00, 0x6F, 0x00, 0x64, 0x00, 0x6F, 0x00,
        0x20, 0x00, 0x3A, 0x00, 0x20, 0x00, 0x0D, 0x00, 0x0A, 0x00, 0x7D, 0x00, 0x0D, 0x00, 0x0A, 0x00,
        0x0D, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    let mut script_stream = cfb
        .create_stream("/Scripts/DefaultJScript")
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    script_stream.write_all(&default_jscript)?;
    drop(script_stream);

    // Flush and return the CFB data
    cfb.flush()
        .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;

    Ok(cfb.into_inner().into_inner())
}

/// Serialize FileHeader to bytes
fn serialize_file_header(header: &crate::parser::header::FileHeader) -> Result<Vec<u8>> {
    Ok(header.to_bytes())
}

/// Serialize DocInfo to bytes
fn serialize_doc_info(doc_info: &crate::parser::doc_info::DocInfo) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write document properties (always required) - level 0
    let props = doc_info
        .properties
        .as_ref()
        .map_or_else(crate::model::document::DocumentProperties::default, |p| {
            p.clone()
        });
    write_record(
        &mut writer,
        0x10,
        0,
        &serialize_document_properties(&props)?,
    )?;

    // Write ID mappings (required for compatibility) - level 0
    write_record(&mut writer, 0x11, 0, &serialize_id_mappings(doc_info)?)?;

    // Write face names - level 1
    for face_name in &doc_info.face_names {
        write_record(&mut writer, 0x13, 1, &serialize_face_name(face_name)?)?;
    }

    // Write border fills - level 1
    for border_fill in &doc_info.border_fills {
        write_record(&mut writer, 0x14, 1, &serialize_border_fill(border_fill)?)?;
    }

    // Write character shapes - level 1
    for char_shape in &doc_info.char_shapes {
        write_record(&mut writer, 0x15, 1, &serialize_char_shape(char_shape)?)?;
    }

    // Write tab definitions - level 1
    for tab_def in &doc_info.tab_defs {
        write_record(&mut writer, 0x16, 1, &serialize_tab_def(tab_def)?)?;
    }

    // Write paragraph shapes - level 1
    for para_shape in &doc_info.para_shapes {
        write_record(&mut writer, 0x19, 1, &serialize_para_shape(para_shape)?)?;
    }

    // Write styles - level 1
    for style in &doc_info.styles {
        write_record(&mut writer, 0x1A, 1, &serialize_style(style)?)?;
    }

    // Write COMPATIBLE_DOCUMENT (0x1E) - required for HWP compatibility
    // Value 0 = current HWP version
    write_record(&mut writer, 0x1E, 0, &[0u8; 4])?;

    // Write LAYOUT_COMPATIBILITY (0x1F) - required for HWP compatibility
    // 20 bytes, all zeros = default compatibility
    write_record(&mut writer, 0x1F, 1, &[0u8; 20])?;

    Ok(data)
}

/// Serialize BodyText to bytes
/// HWP BodyText tags (HWPTAG_BEGIN = 0x10, so 0x42 = 0x10 + 50):
/// - 0x42 = PARA_HEADER
/// - 0x43 = PARA_TEXT
/// - 0x44 = PARA_CHAR_SHAPE
/// - 0x45 = PARA_LINE_SEG
/// - 0x47 = CTRL_HEADER
/// - 0x49 = PAGE_DEF
/// - 0x4A = FOOTNOTE_SHAPE
/// - 0x4B = PAGE_BORDER_FILL
fn serialize_body_text(body_text: &crate::parser::body_text::BodyText) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    for section in &body_text.sections {
        // First, write section definition paragraph (required for HWP structure)
        write_section_definition(&mut writer)?;

        // Then write content paragraphs
        let para_count = section.paragraphs.len();
        for (i, paragraph) in section.paragraphs.iter().enumerate() {
            let is_last = i == para_count - 1;
            write_content_paragraph(&mut writer, paragraph, is_last)?;
        }
    }

    Ok(data)
}

/// Write section definition paragraph (secd + cold controls)
fn write_section_definition<W: Write>(writer: &mut W) -> Result<()> {
    // PARA_HEADER for section definition (charCount=17, has section control)
    #[rustfmt::skip]
    let para_header: [u8; 24] = [
        0x11, 0x00, 0x00, 0x00, // charCount=17, lastInList=false (NOT last!)
        0x04, 0x00, 0x00, 0x00, // controlMask = has section define
        0x00, 0x00,             // paraShapeId = 0
        0x00,                   // styleId = 0
        0x03,                   // divideSort = 3
        0x01, 0x00,             // charShapeCount = 1
        0x00, 0x00,             // rangeTagCount = 0
        0x01, 0x00,             // lineAlignCount = 1
        0x00, 0x00, 0x00, 0x00, // instanceId = 0
        0x00, 0x00,             // isMergedByTrack = 0
    ];
    write_record(writer, 0x42, 0, &para_header)?;

    // PARA_TEXT with section/column control characters
    #[rustfmt::skip]
    let para_text: [u8; 34] = [
        0x02, 0x00,             // Extended control marker
        0x64, 0x63, 0x65, 0x73, // 'secd' (section define)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8 bytes reserved
        0x02, 0x00,             // Extended control marker
        0x02, 0x00,             // Another marker
        0x64, 0x6C, 0x6F, 0x63, // 'cold' (column define)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8 bytes reserved
        0x02, 0x00,             // Section end marker
        0x0D, 0x00,             // Paragraph end (carriage return)
    ];
    write_record(writer, 0x43, 1, &para_text)?;

    // PARA_CHAR_SHAPE
    let char_shape: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    write_record(writer, 0x44, 1, &char_shape)?;

    // PARA_LINE_SEG (line layout info)
    #[rustfmt::skip]
    let line_seg: [u8; 36] = [
        0x00, 0x00, 0x00, 0x00, // textStartPos
        0x00, 0x00, 0x00, 0x00, // lineVerticalPos
        0xE8, 0x03, 0x00, 0x00, // lineHeight = 1000
        0xE8, 0x03, 0x00, 0x00, // textHeight = 1000
        0x52, 0x03, 0x00, 0x00, // baseLineGap = 850
        0x58, 0x02, 0x00, 0x00, // lineSpacing = 600
        0x00, 0x00, 0x00, 0x00, // startMargin
        0x18, 0xA6, 0x00, 0x00, // lineWidth = 42520
        0x00, 0x00, 0x06, 0x00, // flags
    ];
    write_record(writer, 0x45, 1, &line_seg)?;

    // CTRL_HEADER for 'secd' (section define)
    #[rustfmt::skip]
    let ctrl_secd: [u8; 38] = [
        0x64, 0x63, 0x65, 0x73, // 'secd'
        0x00, 0x00, 0x00, 0x00,
        0x6E, 0x04, 0x00, 0x00, 0x00, 0x00, 0x40, 0x1F,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    write_record(writer, 0x47, 1, &ctrl_secd)?;

    // PAGE_DEF (A4 size page definition)
    #[rustfmt::skip]
    let page_def: [u8; 40] = [
        0x88, 0xE8, 0x00, 0x00, // width = 59528 (A4)
        0xDC, 0x48, 0x01, 0x00, // height = 84188 (A4)
        0x38, 0x21, 0x00, 0x00, // left margin = 8504
        0x38, 0x21, 0x00, 0x00, // right margin = 8504
        0x24, 0x16, 0x00, 0x00, // top margin = 5668
        0x9C, 0x10, 0x00, 0x00, // bottom margin = 4252
        0x9C, 0x10, 0x00, 0x00, // header margin = 4252
        0x9C, 0x10, 0x00, 0x00, // footer margin = 4252
        0x00, 0x00, 0x00, 0x00, // gutter = 0
        0x00, 0x00, 0x00, 0x00, // properties = 0
    ];
    write_record(writer, 0x49, 2, &page_def)?;

    // FOOTNOTE_SHAPE x2
    #[rustfmt::skip]
    let footnote1: [u8; 28] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x29, 0x00, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        0x52, 0x03, 0x37, 0x02, 0x1B, 0x01, 0x01, 0x01,
        0x00, 0x00, 0x00, 0x00,
    ];
    write_record(writer, 0x4A, 2, &footnote1)?;

    #[rustfmt::skip]
    let footnote2: [u8; 28] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x29, 0x00, 0x01, 0x00, 0xF8, 0x2F, 0xE0, 0x00,
        0x52, 0x03, 0x37, 0x02, 0x00, 0x00, 0x01, 0x01,
        0x00, 0x00, 0x00, 0x00,
    ];
    write_record(writer, 0x4A, 2, &footnote2)?;

    // PAGE_BORDER_FILL x3
    #[rustfmt::skip]
    let border_fill: [u8; 14] = [
        0x01, 0x00, 0x00, 0x00, 0x89, 0x05, 0x89, 0x05,
        0x89, 0x05, 0x89, 0x05, 0x01, 0x00,
    ];
    write_record(writer, 0x4B, 2, &border_fill)?;
    write_record(writer, 0x4B, 2, &border_fill)?;
    write_record(writer, 0x4B, 2, &border_fill)?;

    // CTRL_HEADER for 'cold' (column define)
    #[rustfmt::skip]
    let ctrl_cold: [u8; 16] = [
        0x64, 0x6C, 0x6F, 0x63, // 'cold'
        0x04, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];
    write_record(writer, 0x47, 1, &ctrl_cold)?;

    Ok(())
}

/// Write a content paragraph with text
fn write_content_paragraph<W: Write>(
    writer: &mut W,
    paragraph: &crate::model::paragraph::Paragraph,
    is_last: bool,
) -> Result<()> {
    // Get text content
    let text_content = paragraph
        .text
        .as_ref()
        .map(|t| t.content.as_str())
        .unwrap_or("");

    // Convert to UTF-16LE and add paragraph end marker
    let mut text_utf16 = string_to_utf16le(text_content);
    text_utf16.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
    let char_count = (text_utf16.len() / 2) as u32;

    // PARA_HEADER
    let mut para_header = Vec::new();
    let char_count_flags = if is_last {
        char_count | 0x80000000 // lastInList = true
    } else {
        char_count // lastInList = false
    };
    para_header.write_u32::<LittleEndian>(char_count_flags)?;
    para_header.write_u32::<LittleEndian>(0)?; // controlMask
    para_header.write_u16::<LittleEndian>(0)?; // paraShapeId
    para_header.write_u8(0)?; // styleId
    para_header.write_u8(0)?; // divideSort
    para_header.write_u16::<LittleEndian>(1)?; // charShapeCount
    para_header.write_u16::<LittleEndian>(0)?; // rangeTagCount
    para_header.write_u16::<LittleEndian>(1)?; // lineAlignCount
    para_header.write_u32::<LittleEndian>(0)?; // instanceId
    para_header.write_u16::<LittleEndian>(0)?; // isMergedByTrack
    write_record(writer, 0x42, 0, &para_header)?;

    // PARA_TEXT
    write_record(writer, 0x43, 1, &text_utf16)?;

    // PARA_CHAR_SHAPE
    let char_shape: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    write_record(writer, 0x44, 1, &char_shape)?;

    // PARA_LINE_SEG (basic line layout)
    #[rustfmt::skip]
    let line_seg: [u8; 36] = [
        0x00, 0x00, 0x00, 0x00, // textStartPos
        0x00, 0x00, 0x00, 0x00, // lineVerticalPos
        0xE8, 0x03, 0x00, 0x00, // lineHeight = 1000
        0xE8, 0x03, 0x00, 0x00, // textHeight = 1000
        0x52, 0x03, 0x00, 0x00, // baseLineGap = 850
        0x58, 0x02, 0x00, 0x00, // lineSpacing = 600
        0x00, 0x00, 0x00, 0x00, // startMargin
        0x18, 0xA6, 0x00, 0x00, // lineWidth = 42520
        0x00, 0x00, 0x06, 0x00, // flags
    ];
    write_record(writer, 0x45, 1, &line_seg)?;

    Ok(())
}

// The following functions are kept for future extension (hyperlinks, images, headers/footers)
#[allow(dead_code)]
/// Serialize paragraph header (HWPTAG_PARA_HEADER = 0x42)
/// Structure: characterCount(4) + controlMask(4) + paraShapeId(2) + styleId(1) +
///            divideSort(1) + charShapeCount(2) + rangeTagCount(2) + lineAlignCount(2) +
///            instanceId(4) + isMergedByTrack(2) = 24 bytes
fn serialize_paragraph_header(paragraph: &crate::model::paragraph::Paragraph) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Calculate character count from text (including control chars)
    let char_count = paragraph
        .text
        .as_ref()
        .map(|t| t.content.len() as u32 / 2) // UTF-16 chars
        .unwrap_or(0);

    // Character count with lastInList flag (bit 31)
    // For simple paragraphs, set lastInList = true
    let char_count_with_flags = char_count | 0x80000000; // Set lastInList bit
    writer.write_u32::<LittleEndian>(char_count_with_flags)?;

    // Control mask
    writer.write_u32::<LittleEndian>(paragraph.control_mask)?;

    // Para shape ID
    writer.write_u16::<LittleEndian>(paragraph.para_shape_id)?;

    // Style ID
    writer.write_u8(paragraph.style_id)?;

    // Divide sort (column type)
    writer.write_u8(paragraph.column_type)?;

    // Char shape count
    writer.write_u16::<LittleEndian>(paragraph.char_shape_count.max(1))?;

    // Range tag count
    writer.write_u16::<LittleEndian>(paragraph.range_tag_count)?;

    // Line align count
    writer.write_u16::<LittleEndian>(paragraph.line_align_count.max(1))?;

    // Instance ID
    writer.write_u32::<LittleEndian>(paragraph.instance_id)?;

    // IsMergedByTrack (2 bytes for HWP 5.0.3.2+)
    writer.write_u16::<LittleEndian>(0)?;

    Ok(data)
}

/// Serialize paragraph text (0x51)
#[allow(dead_code)]
fn serialize_paragraph_text(text: &crate::model::paragraph::ParaText) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Convert text to UTF-16LE
    let utf16_bytes = string_to_utf16le(&text.content);

    // Write text content
    writer.write_all(&utf16_bytes)?;

    Ok(data)
}

/// Serialize paragraph character shapes (0x52)
#[allow(dead_code)]
fn serialize_para_char_shapes(
    char_shapes: &crate::model::para_char_shape::ParaCharShape,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write number of character shape positions
    writer.write_u32::<LittleEndian>(char_shapes.char_positions.len() as u32)?;

    // Write each character position and shape ID
    for pos_shape in &char_shapes.char_positions {
        writer.write_u32::<LittleEndian>(pos_shape.position)?;
        writer.write_u16::<LittleEndian>(pos_shape.char_shape_id)?;
    }

    Ok(data)
}

/// Serialize hyperlink as ParaRangeTag (0x54)
#[allow(dead_code)]
fn serialize_para_range_tag_hyperlink(
    hyperlink: &crate::model::hyperlink::Hyperlink,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // ParaRangeTag structure for hyperlink (based on actual file analysis)
    // Control ID for hyperlink: 'gsh ' (0x20687367 in little-endian)
    writer.write_u32::<LittleEndian>(0x20687367)?; // 'gsh '

    // Fixed header (28 bytes of mostly zeros/fixed values)
    for _ in 0..7 {
        writer.write_u32::<LittleEndian>(0)?;
    }

    // Hyperlink data starts at offset 0x20 (32 bytes)
    // Hyperlink type (u16)
    writer.write_u16::<LittleEndian>(hyperlink.hyperlink_type as u16)?;

    // Flags (typically 0x000001FF)
    let mut flags = 0x000001FF;
    if hyperlink.open_in_new_window {
        flags |= 0x00000200;
    }
    writer.write_u16::<LittleEndian>((flags & 0xFFFF) as u16)?; // Lower 16 bits
    writer.write_u16::<LittleEndian>((flags >> 16) as u16)?; // Upper 16 bits

    // Color info (default: 0x80008000)
    writer.write_u32::<LittleEndian>(0x80008000)?;

    // Write strings as UTF-16LE with length prefix
    // Display text
    let display_text_utf16 = string_to_utf16le(&hyperlink.display_text);
    writer.write_u16::<LittleEndian>((display_text_utf16.len() / 2) as u16)?;
    writer.write_all(&display_text_utf16)?;

    // Target URL
    let target_url_utf16 = string_to_utf16le(&hyperlink.target_url);
    writer.write_u16::<LittleEndian>((target_url_utf16.len() / 2) as u16)?;
    writer.write_all(&target_url_utf16)?;

    // Tooltip (optional)
    if let Some(tooltip) = &hyperlink.tooltip {
        let tooltip_utf16 = string_to_utf16le(tooltip);
        writer.write_u16::<LittleEndian>((tooltip_utf16.len() / 2) as u16)?;
        writer.write_all(&tooltip_utf16)?;
    } else {
        writer.write_u16::<LittleEndian>(0)?; // No tooltip
    }

    Ok(data)
}

/// Serialize page definition (0x57)
#[allow(dead_code)]
fn serialize_page_def(page_def: &crate::model::page_def::PageDef) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write page dimensions
    writer.write_u32::<LittleEndian>(page_def.width)?;
    writer.write_u32::<LittleEndian>(page_def.height)?;

    // Write margins
    writer.write_u32::<LittleEndian>(page_def.left_margin)?;
    writer.write_u32::<LittleEndian>(page_def.right_margin)?;
    writer.write_u32::<LittleEndian>(page_def.top_margin)?;
    writer.write_u32::<LittleEndian>(page_def.bottom_margin)?;
    writer.write_u32::<LittleEndian>(page_def.header_margin)?;
    writer.write_u32::<LittleEndian>(page_def.footer_margin)?;
    writer.write_u32::<LittleEndian>(page_def.gutter_margin)?;

    // Write properties
    writer.write_u32::<LittleEndian>(page_def.properties)?;

    // Write shape IDs
    writer.write_u16::<LittleEndian>(page_def.footnote_shape_id)?;
    writer.write_u16::<LittleEndian>(page_def.page_border_fill_id)?;

    // Note: Header/Footer are separate controls, not part of PageDef
    // They should be written as separate CtrlHeader records with 'head'/'foot' IDs

    Ok(data)
}

/// Serialize header/footer control
#[allow(dead_code)]
fn serialize_header_footer_control(
    header_footer: &crate::model::header_footer::HeaderFooter,
    _is_header: bool,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Based on actual file analysis, HeaderFooter is a 40-byte structure with 10 u32 fields
    // Field 1: Unknown (observed: 0x0000E888)
    writer.write_u32::<LittleEndian>(0x0000E888)?;

    // Field 2: Unknown (observed: 0x000148DA)
    writer.write_u32::<LittleEndian>(0x000148DA)?;

    // Field 3 & 4: Height (observed same value twice: 0x00002138 = 8504 HWPU = ~85mm)
    writer.write_u32::<LittleEndian>(header_footer.height)?;
    writer.write_u32::<LittleEndian>(header_footer.height)?;

    // Field 5: Left margin (observed: 0x00001624 = 5668 HWPU = ~56.68mm)
    writer.write_u32::<LittleEndian>(header_footer.margin)?;

    // Fields 6-8: Top/Right/Bottom margins (observed: 0x0000109C = 4252 HWPU = ~42.52mm)
    writer.write_u32::<LittleEndian>(header_footer.margin)?;
    writer.write_u32::<LittleEndian>(header_footer.margin)?;
    writer.write_u32::<LittleEndian>(header_footer.margin)?;

    // Fields 9-10: Reserved/padding (observed: 0x00000000)
    writer.write_u32::<LittleEndian>(0)?;
    writer.write_u32::<LittleEndian>(0)?;

    // Note: The actual text content is stored elsewhere, not in this 40-byte structure
    // This structure only defines the layout properties

    Ok(data)
}

/// Serialize picture control  
#[allow(dead_code)]
fn serialize_picture_control(picture: &crate::model::control::Picture) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Control header for picture
    // Control ID for picture: '$pic'
    writer.write_u32::<LittleEndian>(0x63697024)?; // '$pic' in little-endian (0x24706963 in big-endian)

    // Control instance ID
    writer.write_u32::<LittleEndian>(picture.instance_id)?;

    // Control attributes
    writer.write_u32::<LittleEndian>(picture.properties)?;

    // Position and size
    writer.write_i32::<LittleEndian>(picture.left)?;
    writer.write_i32::<LittleEndian>(picture.top)?;
    writer.write_i32::<LittleEndian>(picture.right)?;
    writer.write_i32::<LittleEndian>(picture.bottom)?;

    // Margins
    writer.write_i16::<LittleEndian>(picture.outer_margin_left as i16)?;
    writer.write_i16::<LittleEndian>(picture.outer_margin_right as i16)?;
    writer.write_i16::<LittleEndian>(picture.outer_margin_top as i16)?;
    writer.write_i16::<LittleEndian>(picture.outer_margin_bottom as i16)?;

    // Binary item ID and border fill
    writer.write_u16::<LittleEndian>(picture.bin_item_id)?;
    writer.write_u16::<LittleEndian>(picture.border_fill_id)?;

    // Image dimensions
    writer.write_u32::<LittleEndian>(picture.image_width)?;
    writer.write_u32::<LittleEndian>(picture.image_height)?;

    Ok(data)
}

/// Write a record with header and data
fn write_record<W: Write>(writer: &mut W, tag: u16, level: u16, data: &[u8]) -> Result<()> {
    let size = data.len() as u32;

    if size < 0xFFF {
        // Pack into single u32: tag(10) | level(10) | size(12)
        let header = (tag as u32) | ((level as u32) << 10) | (size << 20);
        writer.write_u32::<LittleEndian>(header)?;
    } else {
        // Extended size format
        let header = (tag as u32) | ((level as u32) << 10) | (0xFFF << 20);
        writer.write_u32::<LittleEndian>(header)?;
        writer.write_u32::<LittleEndian>(size)?;
    }

    writer.write_all(data)?;
    Ok(())
}

/// Serialize document properties (26 bytes for HWP compatibility)
fn serialize_document_properties(
    props: &crate::model::document::DocumentProperties,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // 7 u16 fields (14 bytes)
    writer.write_u16::<LittleEndian>(props.section_count.max(1))?; // Must be at least 1
    writer.write_u16::<LittleEndian>(props.page_start_number.max(1))?; // Default 1
    writer.write_u16::<LittleEndian>(props.footnote_start_number.max(1))?; // Default 1
    writer.write_u16::<LittleEndian>(props.endnote_start_number.max(1))?; // Default 1
    writer.write_u16::<LittleEndian>(props.picture_start_number.max(1))?; // Default 1
    writer.write_u16::<LittleEndian>(props.table_start_number.max(1))?; // Default 1
    writer.write_u16::<LittleEndian>(props.equation_start_number)?; // Can be 0

    // 3 u32 fields (12 bytes) - list numbering/bullet related
    writer.write_u32::<LittleEndian>(0)?; // List ID numbering
    writer.write_u32::<LittleEndian>(0)?; // Bullet ID numbering
    writer.write_u32::<LittleEndian>(0)?; // Reserved or caret position

    Ok(data)
}

/// Serialize ID mappings (HWPTAG_ID_MAPPINGS = 0x11)
/// ID_MAPPINGS counts must EXACTLY match the number of actual records that follow
fn serialize_id_mappings(doc_info: &crate::parser::doc_info::DocInfo) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Bin Data Count
    writer.write_u32::<LittleEndian>(doc_info.bin_data.len() as u32)?;

    // Font counts per language category (7 categories)
    // IMPORTANT: Total face names written = sum of all category counts
    // We write ALL face names under Korean category for simplicity
    let face_name_count = doc_info.face_names.len() as u32;
    writer.write_u32::<LittleEndian>(face_name_count)?; // Korean
    writer.write_u32::<LittleEndian>(0)?; // English
    writer.write_u32::<LittleEndian>(0)?; // Hanja
    writer.write_u32::<LittleEndian>(0)?; // Japanese
    writer.write_u32::<LittleEndian>(0)?; // Other
    writer.write_u32::<LittleEndian>(0)?; // Symbol
    writer.write_u32::<LittleEndian>(0)?; // User

    // Border Fill Count
    writer.write_u32::<LittleEndian>(doc_info.border_fills.len().max(1) as u32)?;

    // Char Shape Count
    writer.write_u32::<LittleEndian>(doc_info.char_shapes.len().max(1) as u32)?;

    // Tab Def Count
    writer.write_u32::<LittleEndian>(doc_info.tab_defs.len().max(1) as u32)?;

    // Numbering Count
    writer.write_u32::<LittleEndian>(doc_info.numberings.len() as u32)?;

    // Bullet Count
    writer.write_u32::<LittleEndian>(doc_info.bullets.len() as u32)?;

    // Para Shape Count
    writer.write_u32::<LittleEndian>(doc_info.para_shapes.len().max(1) as u32)?;

    // Style Count
    writer.write_u32::<LittleEndian>(doc_info.styles.len().max(1) as u32)?;

    // Memo Shape Count
    writer.write_u32::<LittleEndian>(0)?;

    // TrackChange Author Count (required for HWP 5.0.2.1+)
    writer.write_u32::<LittleEndian>(0)?;

    // TrackChange Count
    writer.write_u32::<LittleEndian>(0)?;

    Ok(data)
}

/// Serialize face name
fn serialize_face_name(face_name: &crate::model::char_shape::FaceName) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    writer.write_u8(face_name.properties)?;

    let font_name_utf16 = string_to_utf16le(&face_name.font_name);
    writer.write_u16::<LittleEndian>(font_name_utf16.len() as u16 / 2)?;
    writer.write_all(&font_name_utf16)?;

    Ok(data)
}

/// Serialize character shape
fn serialize_char_shape(char_shape: &crate::model::char_shape::CharShape) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write face name IDs
    for &id in &char_shape.face_name_ids {
        writer.write_u16::<LittleEndian>(id)?;
    }

    // Write ratios
    for &ratio in &char_shape.ratios {
        writer.write_u8(ratio)?;
    }

    // Write character spaces
    for &space in &char_shape.char_spaces {
        writer.write_u8(space as u8)?;
    }

    // Write relative sizes
    for &size in &char_shape.relative_sizes {
        writer.write_u8(size)?;
    }

    // Write character offsets
    for &offset in &char_shape.char_offsets {
        writer.write_u8(offset as u8)?;
    }

    writer.write_i32::<LittleEndian>(char_shape.base_size)?;
    writer.write_u32::<LittleEndian>(char_shape.properties)?;
    writer.write_u8(char_shape.shadow_gap_x as u8)?;
    writer.write_u8(char_shape.shadow_gap_y as u8)?;
    writer.write_u32::<LittleEndian>(char_shape.text_color)?;
    writer.write_u32::<LittleEndian>(char_shape.underline_color)?;
    writer.write_u32::<LittleEndian>(char_shape.shade_color)?;
    writer.write_u32::<LittleEndian>(char_shape.shadow_color)?;
    writer.write_u16::<LittleEndian>(char_shape.border_fill_id)?;

    // Reserved bytes (needed for 72-byte size)
    writer.write_u16::<LittleEndian>(0)?;

    Ok(data)
}

/// Serialize paragraph shape
fn serialize_para_shape(para_shape: &crate::model::para_shape::ParaShape) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    writer.write_u32::<LittleEndian>(para_shape.properties1)?;
    writer.write_i32::<LittleEndian>(para_shape.left_margin)?;
    writer.write_i32::<LittleEndian>(para_shape.right_margin)?;
    writer.write_i32::<LittleEndian>(para_shape.indent)?;
    writer.write_i32::<LittleEndian>(para_shape.top_para_space)?;
    writer.write_i32::<LittleEndian>(para_shape.bottom_para_space)?;
    writer.write_i32::<LittleEndian>(para_shape.line_space)?;
    writer.write_u16::<LittleEndian>(para_shape.tab_def_id)?;
    writer.write_u16::<LittleEndian>(para_shape.numbering_id)?;
    writer.write_u16::<LittleEndian>(para_shape.border_fill_id)?;
    writer.write_i16::<LittleEndian>(para_shape.border_left_space)?;
    writer.write_i16::<LittleEndian>(para_shape.border_right_space)?;
    writer.write_i16::<LittleEndian>(para_shape.border_top_space)?;
    writer.write_i16::<LittleEndian>(para_shape.border_bottom_space)?;
    writer.write_u32::<LittleEndian>(para_shape.properties2)?;
    writer.write_u32::<LittleEndian>(para_shape.properties3)?;
    writer.write_u32::<LittleEndian>(para_shape.line_space_type)?;

    Ok(data)
}

/// Serialize style
fn serialize_style(style: &crate::model::style::Style) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write name
    let name_utf16 = string_to_utf16le(&style.name);
    writer.write_u16::<LittleEndian>(name_utf16.len() as u16 / 2)?;
    writer.write_all(&name_utf16)?;

    // Write English name
    let english_name_utf16 = string_to_utf16le(&style.english_name);
    writer.write_u16::<LittleEndian>(english_name_utf16.len() as u16 / 2)?;
    writer.write_all(&english_name_utf16)?;

    writer.write_u8(style.properties)?;
    writer.write_u8(style.next_style_id)?;
    writer.write_u16::<LittleEndian>(style.lang_id)?;
    writer.write_u16::<LittleEndian>(style.para_shape_id)?;
    writer.write_u16::<LittleEndian>(style.char_shape_id)?;

    Ok(data)
}

/// Serialize border fill
fn serialize_border_fill(border_fill: &crate::model::border_fill::BorderFill) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    writer.write_u16::<LittleEndian>(border_fill.properties)?;

    // Write border lines
    serialize_border_line(&mut writer, &border_fill.left)?;
    serialize_border_line(&mut writer, &border_fill.right)?;
    serialize_border_line(&mut writer, &border_fill.top)?;
    serialize_border_line(&mut writer, &border_fill.bottom)?;
    serialize_border_line(&mut writer, &border_fill.diagonal)?;

    // Write fill info (simplified)
    writer.write_u8(border_fill.fill_info.fill_type as u8)?;
    writer.write_u32::<LittleEndian>(border_fill.fill_info.back_color)?;
    writer.write_u32::<LittleEndian>(border_fill.fill_info.pattern_color)?;
    writer.write_u8(border_fill.fill_info.pattern_type as u8)?;

    Ok(data)
}

/// Serialize border line
fn serialize_border_line<W: Write>(
    writer: &mut W,
    border_line: &crate::model::border_fill::BorderLine,
) -> Result<()> {
    writer.write_u8(border_line.line_type)?;
    writer.write_u8(border_line.thickness)?;
    writer.write_u32::<LittleEndian>(border_line.color)?;
    Ok(())
}

/// Serialize tab definition
fn serialize_tab_def(tab_def: &crate::model::tab_def::TabDef) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    writer.write_u32::<LittleEndian>(tab_def.properties)?;

    for tab in &tab_def.tabs {
        writer.write_u32::<LittleEndian>(tab.position)?;
        writer.write_u8(tab.tab_type)?;
        writer.write_u8(tab.leader_type)?;
    }

    Ok(data)
}

/// Compress data using raw deflate (no zlib header - HWP format requirement)
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed_data = encoder.finish()?;
    Ok(compressed_data)
}

/// Create PrvText stream (preview text)
fn create_preview_text(document: &HwpDocument) -> Result<Vec<u8>> {
    let mut preview_text = String::new();

    // Extract first 1000 characters of text for preview
    for body_text in &document.body_texts {
        for section in &body_text.sections {
            for paragraph in &section.paragraphs {
                if let Some(text) = &paragraph.text {
                    preview_text.push_str(&text.content);
                    if preview_text.len() > 1000 {
                        // Find the last character boundary at or before position 1000
                        let mut truncate_pos = 1000.min(preview_text.len());
                        while truncate_pos > 0 && !preview_text.is_char_boundary(truncate_pos) {
                            truncate_pos -= 1;
                        }
                        preview_text.truncate(truncate_pos);
                        break;
                    }
                }
            }
            if preview_text.len() >= 1000 {
                break;
            }
        }
        if preview_text.len() >= 1000 {
            break;
        }
    }

    // Convert to UTF-16LE
    let utf16_bytes = string_to_utf16le(&preview_text);
    Ok(utf16_bytes)
}

/// Create DocOptions stream
fn create_doc_options() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // DocOptions format - minimal version
    // This contains various document display options

    // Write version
    writer.write_u32::<LittleEndian>(0x00000001)?; // Version 1

    // Write default view options
    writer.write_u32::<LittleEndian>(0x00000000)?; // View mode
    writer.write_u32::<LittleEndian>(0x00000064)?; // Zoom level (100%)
    writer.write_u32::<LittleEndian>(0x00000000)?; // View flags

    // Write default edit options
    writer.write_u32::<LittleEndian>(0x00000001)?; // Edit mode
    writer.write_u32::<LittleEndian>(0x00000000)?; // Edit flags

    Ok(data)
}
