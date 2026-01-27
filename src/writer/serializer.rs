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
            // HWP format uses BIN{ID:04X}.{ext} format for the stream name
            // The extension is included for proper file type identification
            let stream_name = format!("/BinData/BIN{:04X}.{}", bin_data.bin_id, bin_data.extension.to_lowercase());

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
/// Following real Hangul file structure - DOC_PROPERTIES and ID_MAPPINGS at level 0,
/// other records at level 1
///
/// CRITICAL: Record order must match ID_MAPPINGS declaration order!
/// ID_MAPPINGS declares counts in this order: BinData, FaceNames(x7), BorderFill, CharShape,
/// TabDef, Numbering, Bullet, ParaShape, Style, MemoShape, TrackChangeAuthor, TrackChange
/// The parser expects records to appear in EXACTLY this order after ID_MAPPINGS.
fn serialize_doc_info(doc_info: &crate::parser::doc_info::DocInfo) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // === Level 0 records ===

    // 1. DOC_PROPERTIES (0x10) - level 0
    let props = doc_info
        .properties
        .as_ref()
        .map_or_else(crate::model::document::DocumentProperties::default, |p| {
            p.clone()
        });
    write_record(&mut writer, 0x10, 0, &serialize_document_properties(&props)?)?;

    // 2. ID_MAPPINGS (0x11) - level 0
    // This declares the COUNT of each record type that follows
    write_record(&mut writer, 0x11, 0, &serialize_id_mappings(doc_info)?)?;

    // === Level 1 records (MUST follow ID_MAPPINGS order!) ===

    // 3. BinData (0x12) - MUST be immediately after ID_MAPPINGS!
    // The parser reads BinData count first and expects that many BinData records next
    for bin_data in &doc_info.bin_data {
        write_record(&mut writer, 0x12, 1, &serialize_bin_data_info(bin_data)?)?;
    }

    // 4. FaceNames (0x13)
    for face_name in &doc_info.face_names {
        write_record(&mut writer, 0x13, 1, &serialize_face_name(face_name)?)?;
    }

    // 5. BorderFills (0x14)
    for border_fill in &doc_info.border_fills {
        write_record(&mut writer, 0x14, 1, &serialize_border_fill(border_fill)?)?;
    }

    // 6. CharShapes (0x15)
    for char_shape in &doc_info.char_shapes {
        write_record(&mut writer, 0x15, 1, &serialize_char_shape(char_shape)?)?;
    }

    // 7. TabDefs (0x16)
    for tab_def in &doc_info.tab_defs {
        write_record(&mut writer, 0x16, 1, &serialize_tab_def(tab_def)?)?;
    }

    // 8. Numbering (0x17)
    for numbering in &doc_info.numberings {
        write_record(&mut writer, 0x17, 1, &numbering.to_bytes())?;
    }

    // 9. Bullet (0x18)
    for bullet in &doc_info.bullets {
        write_record(&mut writer, 0x18, 1, &bullet.to_bytes())?;
    }

    // 10. ParaShapes (0x19)
    for para_shape in &doc_info.para_shapes {
        write_record(&mut writer, 0x19, 1, &serialize_para_shape(para_shape)?)?;
    }

    // 11. Styles (0x1A)
    for style in &doc_info.styles {
        write_record(&mut writer, 0x1A, 1, &serialize_style(style)?)?;
    }

    // === Compatibility records (required for HWP 5.0.2.1+) ===

    // 12. COMPATIBLE_DOCUMENT (0x1E) - level 0
    // Indicates document compatibility settings (4 bytes, all zeros = default)
    write_record(&mut writer, 0x1E, 0, &[0u8; 4])?;

    // 13. LAYOUT_COMPATIBILITY (0x1F) - level 1
    // Layout compatibility flags (20 bytes, all zeros = default)
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
        write_section_definition(&mut writer, section.page_def.as_ref())?;

        // Write header/footer controls if present
        if let Some(page_def) = &section.page_def {
            for header_footer in &page_def.header_footer.items {
                write_header_footer_control(&mut writer, header_footer)?;
            }
        }

        // Collect indices of paragraphs that are table cell content
        // These will be written inside serialize_table_control instead of main loop
        let mut cell_para_indices = std::collections::HashSet::new();
        for (i, paragraph) in section.paragraphs.iter().enumerate() {
            if let Some(table) = &paragraph.table_data {
                // Find all cell paragraphs by matching instance_id with cell's list_header_id
                for cell in table.cells_by_row() {
                    for (j, p) in section.paragraphs.iter().enumerate() {
                        if p.instance_id == cell.list_header_id && j != i {
                            cell_para_indices.insert(j);
                        }
                    }
                }
            }
        }

        // Write content paragraphs
        let para_count = section.paragraphs.len();
        for (i, paragraph) in section.paragraphs.iter().enumerate() {
            // Skip paragraphs that are table cell content (written inside table control)
            if cell_para_indices.contains(&i) {
                continue;
            }

            let is_last = i == para_count - 1 ||
                          (i < para_count - 1 && cell_para_indices.contains(&(para_count - 1)));
            write_content_paragraph(&mut writer, paragraph, is_last, &section.paragraphs)?;
        }
    }

    Ok(data)
}

/// Write section definition paragraph (secd + cold controls)
fn write_section_definition<W: Write>(
    writer: &mut W,
    page_def: Option<&crate::model::page_def::PageDef>,
) -> Result<()> {
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

    // PAGE_DEF - use actual page_def if provided, otherwise use A4 defaults
    let page_def_data = if let Some(pd) = page_def {
        pd.to_bytes()
    } else {
        // Default A4 page definition
        let default_pd = crate::model::page_def::PageDef::new_default();
        default_pd.to_bytes()
    };
    write_record(writer, 0x49, 2, &page_def_data)?;

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

/// Write a header/footer control paragraph
fn write_header_footer_control<W: Write>(
    writer: &mut W,
    header_footer: &crate::model::header_footer::HeaderFooter,
) -> Result<()> {
    use crate::model::header_footer::HeaderFooterType;

    // Determine control type
    let ctrl_id = match header_footer.header_footer_type {
        HeaderFooterType::Header => 0x72646568, // 'hedr' in little-endian
        HeaderFooterType::Footer => 0x7274666F, // 'ftro' in little-endian
    };

    // PARA_HEADER for header/footer
    let mut para_header = Vec::new();
    let text_utf16 = string_to_utf16le(&header_footer.text);
    let char_count = (text_utf16.len() / 2 + 1) as u32; // +1 for CR

    para_header.write_u32::<LittleEndian>(char_count)?;
    para_header.write_u32::<LittleEndian>(0x04)?; // controlMask = extended control
    para_header.write_u16::<LittleEndian>(header_footer.para_shape_id)?;
    para_header.write_u8(0)?; // styleId
    para_header.write_u8(0)?; // divideSort
    para_header.write_u16::<LittleEndian>(1)?; // charShapeCount
    para_header.write_u16::<LittleEndian>(0)?; // rangeTagCount
    para_header.write_u16::<LittleEndian>(1)?; // lineAlignCount
    para_header.write_u32::<LittleEndian>(0)?; // instanceId
    para_header.write_u16::<LittleEndian>(0)?; // isMergedByTrack
    write_record(writer, 0x42, 0, &para_header)?;

    // PARA_TEXT with header/footer text
    let mut para_text = text_utf16.clone();
    para_text.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
    write_record(writer, 0x43, 1, &para_text)?;

    // PARA_CHAR_SHAPE
    let mut char_shape_data = Vec::new();
    char_shape_data.write_u32::<LittleEndian>(0)?; // position
    char_shape_data.write_u32::<LittleEndian>(header_footer.char_shape_id as u32)?;
    write_record(writer, 0x44, 1, &char_shape_data)?;

    // PARA_LINE_SEG
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

    // CTRL_HEADER for header/footer
    let mut ctrl_header = Vec::new();
    ctrl_header.write_u32::<LittleEndian>(ctrl_id)?;
    ctrl_header.write_u32::<LittleEndian>(0)?; // properties
    ctrl_header.write_u32::<LittleEndian>(0)?; // instanceId
    write_record(writer, 0x47, 1, &ctrl_header)?;

    // Header/Footer control data (40 bytes)
    let hf_data = serialize_header_footer_control(header_footer,
        header_footer.header_footer_type == HeaderFooterType::Header)?;
    write_record(writer, 0x48, 2, &hf_data)?;

    Ok(())
}

/// Write a content paragraph with text and controls
fn write_content_paragraph<W: Write>(
    writer: &mut W,
    paragraph: &crate::model::paragraph::Paragraph,
    is_last: bool,
    all_paragraphs: &[crate::model::paragraph::Paragraph],
) -> Result<()> {
    // Determine control_mask based on paragraph content
    let control_mask = compute_control_mask(paragraph);

    // Check if this is a control-only paragraph (table, picture, etc.)
    let has_table = paragraph.table_data.is_some();
    let has_picture = paragraph.picture_data.is_some();
    let has_text_box = paragraph.text_box_data.is_some();
    let has_hyperlinks = !paragraph.hyperlinks.is_empty();

    // Build PARA_TEXT content
    let text_utf16 = if has_table {
        // Table marker: 0x0B (table inline char) + 'tbl ' (reversed for little-endian)
        let mut table_text = vec![0x0B, 0x00]; // Extended control marker
        table_text.extend_from_slice(&[0x20, 0x6C, 0x62, 0x74]); // 'tbl ' in UTF-16LE
        table_text.extend_from_slice(&[0x00; 8]); // 8 bytes reserved
        table_text.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
        table_text
    } else if has_picture {
        // Picture marker: extended control + '$pic'
        let mut pic_text = vec![0x0B, 0x00]; // Extended control marker
        pic_text.extend_from_slice(&[0x63, 0x69, 0x70, 0x24]); // '$pic' in little-endian
        pic_text.extend_from_slice(&[0x00; 8]); // 8 bytes reserved
        pic_text.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
        pic_text
    } else if has_text_box {
        // TextBox marker
        let mut tb_text = vec![0x0B, 0x00]; // Extended control marker
        tb_text.extend_from_slice(&[0x78, 0x74, 0x62, 0x64]); // 'dbtx' control
        tb_text.extend_from_slice(&[0x00; 8]); // 8 bytes reserved
        tb_text.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
        tb_text
    } else if has_hyperlinks {
        // Text with hyperlinks - use field markers
        let text_content = paragraph
            .text
            .as_ref()
            .map(|t| t.content.as_str())
            .unwrap_or("");
        build_para_text_with_hyperlinks(text_content, &paragraph.hyperlinks)
    } else {
        // Regular text content
        let text_content = paragraph
            .text
            .as_ref()
            .map(|t| t.content.as_str())
            .unwrap_or("");
        let mut text = string_to_utf16le(text_content);
        text.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
        text
    };

    let char_count = (text_utf16.len() / 2) as u32;

    // Range tag count - hyperlinks use field controls, not range tags
    let range_tag_count: u16 = 0;

    // PARA_HEADER
    let mut para_header = Vec::new();
    let char_count_flags = if is_last {
        char_count | 0x80000000 // lastInList = true
    } else {
        char_count // lastInList = false
    };
    para_header.write_u32::<LittleEndian>(char_count_flags)?;
    para_header.write_u32::<LittleEndian>(control_mask)?;
    para_header.write_u16::<LittleEndian>(paragraph.para_shape_id)?;
    para_header.write_u8(paragraph.style_id)?;
    para_header.write_u8(paragraph.column_type)?;
    para_header.write_u16::<LittleEndian>(paragraph.char_shape_count.max(1))?;
    para_header.write_u16::<LittleEndian>(range_tag_count)?;
    para_header.write_u16::<LittleEndian>(paragraph.line_align_count.max(1))?;
    para_header.write_u32::<LittleEndian>(paragraph.instance_id)?;
    para_header.write_u16::<LittleEndian>(0)?; // isMergedByTrack
    write_record(writer, 0x42, 0, &para_header)?;

    // PARA_TEXT
    write_record(writer, 0x43, 1, &text_utf16)?;

    // PARA_CHAR_SHAPE
    if let Some(char_shapes) = &paragraph.char_shapes {
        let char_shape_data = serialize_para_char_shapes(char_shapes)?;
        write_record(writer, 0x44, 1, &char_shape_data)?;
    } else {
        let char_shape: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        write_record(writer, 0x44, 1, &char_shape)?;
    }

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

    // Write CTRL_HEADER for hyperlinks (klh% field control)
    // Hyperlinks use CTRL_HEADER (0x47) with 'klh%' control ID, not PARA_RANGE_TAG
    for hyperlink in &paragraph.hyperlinks {
        let hyperlink_ctrl_data = serialize_hyperlink_ctrl_header(hyperlink)?;
        write_record(writer, 0x47, 1, &hyperlink_ctrl_data)?;
    }

    // Write CTRL_HEADER and control-specific data for other controls
    if let Some(ctrl_header) = &paragraph.ctrl_header {
        // Serialize control-specific data
        if let Some(table) = &paragraph.table_data {
            // Table uses extended CTRL_HEADER format
            let table_ctrl_header = serialize_table_ctrl_header(ctrl_header, table)?;
            write_record(writer, 0x47, 1, &table_ctrl_header)?;
            serialize_table_control(writer, table, all_paragraphs)?;
        } else if let Some(picture) = &paragraph.picture_data {
            // Non-table controls use basic CTRL_HEADER
            let ctrl_header_data = serialize_ctrl_header(ctrl_header)?;
            write_record(writer, 0x47, 1, &ctrl_header_data)?;
            let picture_data = serialize_picture_control(picture)?;
            write_record(writer, 0x48, 2, &picture_data)?;
        } else if let Some(text_box) = &paragraph.text_box_data {
            let ctrl_header_data = serialize_ctrl_header(ctrl_header)?;
            write_record(writer, 0x47, 1, &ctrl_header_data)?;
            let text_box_data = serialize_text_box_control(text_box)?;
            write_record(writer, 0x48, 2, &text_box_data)?;
        } else {
            // Generic control header
            let ctrl_header_data = serialize_ctrl_header(ctrl_header)?;
            write_record(writer, 0x47, 1, &ctrl_header_data)?;
        }
    }

    Ok(())
}

/// Compute control_mask based on paragraph content
fn compute_control_mask(paragraph: &crate::model::paragraph::Paragraph) -> u32 {
    let mut mask = paragraph.control_mask;

    // Bit 11 (0x800): Extended control present (table, picture, textbox, etc.)
    if paragraph.table_data.is_some()
        || paragraph.picture_data.is_some()
        || paragraph.text_box_data.is_some()
    {
        mask |= 0x800;
    }

    // Bit 2 (0x04): Has field controls (hyperlinks use field markers 0x0003/0x0004)
    if !paragraph.hyperlinks.is_empty() {
        mask |= 0x04;
    }

    mask
}

/// Serialize control header (basic version for non-table controls)
fn serialize_ctrl_header(ctrl_header: &crate::model::CtrlHeader) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    writer.write_u32::<LittleEndian>(ctrl_header.ctrl_id)?;
    writer.write_u32::<LittleEndian>(ctrl_header.properties)?;
    writer.write_u32::<LittleEndian>(ctrl_header.instance_id)?;

    Ok(data)
}

/// Serialize extended CTRL_HEADER for tables (48 bytes to match real HWP)
/// Based on real HWP file analysis
fn serialize_table_ctrl_header(
    ctrl_header: &crate::model::CtrlHeader,
    table: &crate::model::control::Table,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // ctrl_id: 'tbl ' (4 bytes)
    writer.write_u32::<LittleEndian>(ctrl_header.ctrl_id)?;

    // Properties/flags (4 bytes) - based on real HWP: 0x082a2311
    // Bit flags for table behavior
    let properties: u32 = 0x082a2310; // Table properties from real HWP
    writer.write_u32::<LittleEndian>(properties)?;

    // Reserved/unknown (8 bytes of zeros)
    writer.write_u64::<LittleEndian>(0)?;

    // Table dimensions - width and height in HWP units
    let total_width = table.cols as u32 * 5000; // Approx width
    let total_height = table.rows as u32 * 1000; // Approx height
    writer.write_u32::<LittleEndian>(total_width)?;
    writer.write_u32::<LittleEndian>(total_height)?;

    // Unknown/reserved (4 bytes)
    writer.write_u32::<LittleEndian>(0)?;

    // Outer margins (left, right, top, bottom) - each u16
    // These are margins around the entire table
    writer.write_u16::<LittleEndian>(140)?; // left outer margin
    writer.write_u16::<LittleEndian>(140)?; // right outer margin
    writer.write_u16::<LittleEndian>(140)?; // top outer margin
    writer.write_u16::<LittleEndian>(140)?; // bottom outer margin

    // Instance ID or reserved (4 bytes)
    writer.write_u32::<LittleEndian>(0)?;

    // Remaining padding (8 bytes to reach 48 total)
    writer.write_u64::<LittleEndian>(0)?;

    Ok(data)
}

/// Serialize table control (CTRL_HEADER tag 0x47 followed by table-specific data)
/// Based on real HWP file analysis:
/// - CTRL_HEADER (0x47) level 1 - extended header with table properties
/// - TABLE (0x4D) level 2 - table metadata (rows, cols, margins, row heights)
/// - LIST_HEADER (0x48) level 2 per cell - cell container
/// - Cell content paragraphs follow at level 2/3
fn serialize_table_control<W: Write>(
    writer: &mut W,
    table: &crate::model::control::Table,
    all_paragraphs: &[crate::model::paragraph::Paragraph],
) -> Result<()> {
    // Write TABLE record (tag 0x4D) - table metadata
    // Based on real HWP format analysis
    let mut table_data = Vec::new();
    let mut td_writer = Cursor::new(&mut table_data);

    // Table properties (flags)
    // Real HWP uses 0x04000004 for multi-row tables, 0x06000006 for single-row
    let flags: u32 = if table.rows == 1 {
        0x06000006
    } else {
        0x04000004
    };
    td_writer.write_u32::<LittleEndian>(flags)?;

    // Row count (u16) and Column count (u16)
    td_writer.write_u16::<LittleEndian>(table.rows)?;
    td_writer.write_u16::<LittleEndian>(table.cols)?;

    // Cell spacing
    td_writer.write_u16::<LittleEndian>(table.cell_spacing)?;

    // Margins (left, right, top, bottom) - each i16
    td_writer.write_i16::<LittleEndian>(table.left_margin as i16)?;
    td_writer.write_i16::<LittleEndian>(table.right_margin as i16)?;
    td_writer.write_i16::<LittleEndian>(table.top_margin as i16)?;
    td_writer.write_i16::<LittleEndian>(table.bottom_margin as i16)?;

    // Row heights (one u16 per row)
    for _ in 0..table.rows {
        td_writer.write_u16::<LittleEndian>(0)?; // 0 = auto height
    }

    // Border fill ID
    td_writer.write_u16::<LittleEndian>(0)?;

    // Zone info count (u16) - always 0 for simple tables
    td_writer.write_u16::<LittleEndian>(0)?;

    write_record(writer, 0x4D, 2, &table_data)?;

    // Write LIST_HEADER (0x48) for each cell, followed by cell content at correct levels
    for cell in table.cells_by_row() {
        write_table_cell_list_header(writer, cell)?;

        // Find and write the cell content paragraph at level 2
        // Cell paragraphs are linked by matching instance_id with cell's list_header_id
        for paragraph in all_paragraphs {
            if paragraph.instance_id == cell.list_header_id && paragraph.text.is_some() {
                write_cell_content_paragraph(writer, paragraph)?;
                break; // Only one paragraph per cell
            }
        }
    }

    Ok(())
}

/// Write a table cell as LIST_HEADER (tag 0x48)
/// This is the correct format based on real HWP file analysis
fn write_table_cell_list_header<W: Write>(
    writer: &mut W,
    cell: &crate::model::control::TableCell,
) -> Result<()> {
    let mut cell_data = Vec::new();
    let mut c_writer = Cursor::new(&mut cell_data);

    // LIST_HEADER structure for table cell
    // Based on real HWP: 47 bytes typically

    // numPara (u32) - number of paragraphs in this cell
    c_writer.write_u32::<LittleEndian>(1)?;

    // properties (u32)
    c_writer.write_u32::<LittleEndian>(0x00000020)?;

    // Unknown/reserved (u32)
    c_writer.write_u32::<LittleEndian>(0)?;

    // Cell address: col (u16), row (u16)
    // cell_address is (row, col) tuple
    c_writer.write_u16::<LittleEndian>(cell.cell_address.1)?; // col
    c_writer.write_u16::<LittleEndian>(cell.cell_address.0)?; // row

    // Col span and row span (u16 each)
    c_writer.write_u16::<LittleEndian>(cell.col_span)?;
    c_writer.write_u16::<LittleEndian>(cell.row_span)?;

    // Cell width and height (u32 each)
    c_writer.write_u32::<LittleEndian>(cell.width)?;
    c_writer.write_u32::<LittleEndian>(cell.height)?;

    // Margins (u16 each): left, right, top, bottom
    c_writer.write_u16::<LittleEndian>(cell.left_margin)?;
    c_writer.write_u16::<LittleEndian>(cell.right_margin)?;
    c_writer.write_u16::<LittleEndian>(cell.top_margin)?;
    c_writer.write_u16::<LittleEndian>(cell.bottom_margin)?;

    // Border fill ID (u16)
    c_writer.write_u16::<LittleEndian>(cell.border_fill_id)?;

    // Text width (u32) - calculated from cell width minus margins
    let text_width = cell.text_width.max(cell.width.saturating_sub(
        (cell.left_margin as u32) + (cell.right_margin as u32),
    ));
    c_writer.write_u32::<LittleEndian>(text_width)?;

    // Field name length (u16) - 0 for no field name
    c_writer.write_u16::<LittleEndian>(0)?;

    // Unknown padding bytes to match real HWP structure (47 bytes total)
    // Additional 3 bytes of padding
    c_writer.write_u8(0)?;
    c_writer.write_u8(0)?;
    c_writer.write_u8(0)?;

    write_record(writer, 0x48, 2, &cell_data)?;

    Ok(())
}

/// Write table cell content paragraph at level 2
fn write_cell_content_paragraph<W: Write>(
    writer: &mut W,
    paragraph: &crate::model::paragraph::Paragraph,
) -> Result<()> {
    let text_content = paragraph
        .text
        .as_ref()
        .map(|t| t.content.as_str())
        .unwrap_or("");

    let text_utf16 = string_to_utf16le(text_content);
    let mut text_with_marker = text_utf16.clone();
    text_with_marker.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker

    let char_count = (text_with_marker.len() / 2) as u32;
    let char_count_flags = char_count | 0x80000000; // lastInList flag

    // PARA_HEADER (tag 0x42, level 2)
    let mut para_header = Vec::new();
    {
        let mut w = Cursor::new(&mut para_header);
        w.write_u32::<LittleEndian>(char_count_flags)?;
        w.write_u32::<LittleEndian>(0)?; // control_mask
        w.write_u16::<LittleEndian>(paragraph.para_shape_id)?;
        w.write_u8(paragraph.style_id)?;
        w.write_u8(0)?; // column_type
        w.write_u16::<LittleEndian>(paragraph.char_shape_count.max(1))?;
        w.write_u16::<LittleEndian>(0)?; // rangeTagCount
        w.write_u16::<LittleEndian>(1)?; // lineAlignCount
        w.write_u32::<LittleEndian>(paragraph.instance_id)?;
        w.write_u16::<LittleEndian>(0)?; // isMergedByTrack
    }
    write_record(writer, 0x42, 2, &para_header)?;

    // PARA_TEXT (tag 0x43, level 3)
    write_record(writer, 0x43, 3, &text_with_marker)?;

    // PARA_CHAR_SHAPE (tag 0x44, level 3)
    if let Some(char_shapes) = &paragraph.char_shapes {
        let char_shape_data = serialize_para_char_shapes(char_shapes)?;
        write_record(writer, 0x44, 3, &char_shape_data)?;
    } else {
        let default_char_shape: [u8; 8] = [0; 8];
        write_record(writer, 0x44, 3, &default_char_shape)?;
    }

    // PARA_LINE_SEG (tag 0x45, level 3)
    #[rustfmt::skip]
    let line_seg: [u8; 36] = [
        0x00, 0x00, 0x00, 0x00, // textStartPos
        0x00, 0x00, 0x00, 0x00, // lineVerticalPos
        0xE8, 0x03, 0x00, 0x00, // lineHeight = 1000
        0xE8, 0x03, 0x00, 0x00, // textHeight = 1000
        0x52, 0x03, 0x00, 0x00, // baseLineGap = 850
        0x58, 0x02, 0x00, 0x00, // lineSpacing = 600
        0x00, 0x00, 0x00, 0x00, // startMargin
        0x00, 0x27, 0x00, 0x00, // lineWidth
        0x00, 0x00, 0x06, 0x00, // flags
    ];
    write_record(writer, 0x45, 3, &line_seg)?;

    Ok(())
}

/// Serialize text box control
fn serialize_text_box_control(
    text_box: &crate::model::text_box::TextBox,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // ShapeComponent basic structure
    // Control ID for text box
    writer.write_u32::<LittleEndian>(0x78746264)?; // 'dbtx' in little-endian

    // Position and size (calculate right/bottom from x,y,width,height)
    let right = text_box.x + text_box.width as i32;
    let bottom = text_box.y + text_box.height as i32;
    writer.write_i32::<LittleEndian>(text_box.x)?;
    writer.write_i32::<LittleEndian>(text_box.y)?;
    writer.write_i32::<LittleEndian>(right)?;
    writer.write_i32::<LittleEndian>(bottom)?;

    // Width and height
    writer.write_u32::<LittleEndian>(text_box.width)?;
    writer.write_u32::<LittleEndian>(text_box.height)?;

    // Properties (alignment and other flags)
    let properties: u32 = (text_box.alignment as u32)
        | ((text_box.z_order as u32) << 8)
        | ((text_box.opacity as u32) << 16);
    writer.write_u32::<LittleEndian>(properties)?;

    // Border properties
    writer.write_u8(text_box.border_style as u8)?;
    writer.write_u8(text_box.border_width)?;
    writer.write_u32::<LittleEndian>(text_box.border_color)?;

    // Fill properties
    writer.write_u8(text_box.fill_type as u8)?;
    writer.write_u32::<LittleEndian>(text_box.background_color)?;

    // Padding
    writer.write_u16::<LittleEndian>(text_box.padding)?;

    // Style IDs
    writer.write_u16::<LittleEndian>(text_box.char_shape_id)?;
    writer.write_u16::<LittleEndian>(text_box.para_shape_id)?;

    // Rotation
    writer.write_i16::<LittleEndian>(text_box.rotation)?;

    // Text content (UTF-16LE with length prefix)
    let text_utf16 = string_to_utf16le(&text_box.text);
    writer.write_u16::<LittleEndian>((text_utf16.len() / 2) as u16)?;
    writer.write_all(&text_utf16)?;

    Ok(data)
}

// The following functions are kept for future extension
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

/// Serialize paragraph character shapes (0x44)
fn serialize_para_char_shapes(
    char_shapes: &crate::model::para_char_shape::ParaCharShape,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write each character position and shape ID pair (4 bytes position + 4 bytes shape ID)
    for pos_shape in &char_shapes.char_positions {
        writer.write_u32::<LittleEndian>(pos_shape.position)?;
        writer.write_u32::<LittleEndian>(pos_shape.char_shape_id as u32)?;
    }

    Ok(data)
}

/// Serialize hyperlink CTRL_HEADER with 'klh%' control ID
/// Based on analysis of HWP files created by Hancom Hangeul
fn serialize_hyperlink_ctrl_header(
    hyperlink: &crate::model::hyperlink::Hyperlink,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Control ID: 'klh%' (0x25686C6B in little-endian)
    // This is the Field Control for hyperlinks in modern HWP
    writer.write_u32::<LittleEndian>(0x25686C6B)?; // 'klh%'

    // Flags (4 bytes) - 0x00009000 is the standard value
    writer.write_u32::<LittleEndian>(0x00009000)?;

    // Reserved byte
    writer.write_u8(0x00)?;

    // Build URL command string: "<URL>;<new_window>;<param2>;<param3>;"
    let new_window_flag = if hyperlink.open_in_new_window { "1" } else { "0" };
    let command = format!("{};{};0;0;", hyperlink.target_url, new_window_flag);

    // URL string length (character count, u16 little-endian)
    let char_count = command.chars().count() as u16;
    writer.write_u16::<LittleEndian>(char_count)?;

    // URL string (UTF-16LE)
    let url_utf16 = string_to_utf16le(&command);
    writer.write_all(&url_utf16)?;

    // NULL padding (8 bytes for alignment)
    writer.write_all(&[0u8; 8])?;

    Ok(data)
}

/// Build PARA_TEXT with hyperlink field markers
/// Hyperlinks use field start (0x0003) and field end (0x0004) markers
fn build_para_text_with_hyperlinks(
    text_content: &str,
    hyperlinks: &[crate::model::hyperlink::Hyperlink],
) -> Vec<u8> {
    let mut result = Vec::new();

    if hyperlinks.is_empty() {
        // No hyperlinks - just regular text
        result.extend_from_slice(&string_to_utf16le(text_content));
        result.extend_from_slice(&[0x0D, 0x00]); // paragraph end marker
        return result;
    }

    // Sort hyperlinks by start position
    let mut sorted_hyperlinks: Vec<_> = hyperlinks.iter().collect();
    sorted_hyperlinks.sort_by_key(|h| h.start_position);

    let chars: Vec<char> = text_content.chars().collect();
    let mut current_pos = 0u32;

    for hyperlink in sorted_hyperlinks {
        let start = hyperlink.start_position as usize;
        let end = start + hyperlink.length as usize;

        // Write text before hyperlink
        if current_pos < start as u32 {
            let before_text: String = chars[current_pos as usize..start].iter().collect();
            result.extend_from_slice(&string_to_utf16le(&before_text));
        }

        // Field start marker (0x0003) + 14 bytes control data (16 bytes total)
        // Format: char_code(2) + ctrl_id(4) + reserved(8) + field_indicator(2)
        result.extend_from_slice(&[0x03, 0x00]); // Field start
        result.extend_from_slice(&[0x6B, 0x6C, 0x68, 0x25]); // 'klh%' control ID
        result.extend_from_slice(&[0x00; 8]); // 8 bytes reserved
        result.extend_from_slice(&[0x03, 0x00]); // Field indicator (repeats field code)

        // Hyperlink display text
        let link_text: String = if end <= chars.len() {
            chars[start..end].iter().collect()
        } else {
            hyperlink.display_text.clone()
        };
        result.extend_from_slice(&string_to_utf16le(&link_text));

        // Field end marker (0x0004) + 14 bytes control data (16 bytes total)
        // Format: char_code(2) + ctrl_id(4) + reserved(8) + field_indicator(2)
        result.extend_from_slice(&[0x04, 0x00]); // Field end
        result.extend_from_slice(&[0x6B, 0x6C, 0x68, 0x00]); // 'klh\0' (end marker differs from start)
        result.extend_from_slice(&[0x00; 8]); // 8 bytes reserved
        result.extend_from_slice(&[0x04, 0x00]); // Field indicator (repeats field code)

        current_pos = end as u32;
    }

    // Write remaining text after last hyperlink
    if (current_pos as usize) < chars.len() {
        let after_text: String = chars[current_pos as usize..].iter().collect();
        result.extend_from_slice(&string_to_utf16le(&after_text));
    }

    // Paragraph end marker
    result.extend_from_slice(&[0x0D, 0x00]);

    result
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

/// Serialize BinData info record (tag 0x12)
fn serialize_bin_data_info(bin_data: &crate::model::bin_data::BinData) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Properties (2 bytes)
    writer.write_u16::<LittleEndian>(bin_data.properties)?;

    // Absolute file name (length + UTF-16LE)
    let abs_name_utf16 = string_to_utf16le(&bin_data.abs_name);
    writer.write_u16::<LittleEndian>(abs_name_utf16.len() as u16 / 2)?;
    writer.write_all(&abs_name_utf16)?;

    // Relative file name (length + UTF-16LE)
    let rel_name_utf16 = string_to_utf16le(&bin_data.rel_name);
    writer.write_u16::<LittleEndian>(rel_name_utf16.len() as u16 / 2)?;
    writer.write_all(&rel_name_utf16)?;

    // BinData ID (2 bytes)
    writer.write_u16::<LittleEndian>(bin_data.bin_id)?;

    // Extension (length + UTF-16LE)
    let ext_utf16 = string_to_utf16le(&bin_data.extension);
    writer.write_u16::<LittleEndian>(ext_utf16.len() as u16 / 2)?;
    writer.write_all(&ext_utf16)?;

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
    // strike_line_color for HWP 5.0.3.0+ (we use 5.1.1.0)
    writer.write_u32::<LittleEndian>(char_shape.strike_line_color)?;

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

    // Properties (4 bytes)
    writer.write_u32::<LittleEndian>(tab_def.properties)?;

    // Tab count (4 bytes) - required field per hwplib
    writer.write_u32::<LittleEndian>(tab_def.tabs.len() as u32)?;

    // Each tab: position (4) + tab_type (1) + leader_type (1) + reserved (2) = 8 bytes
    for tab in &tab_def.tabs {
        writer.write_u32::<LittleEndian>(tab.position)?;
        writer.write_u8(tab.tab_type)?;
        writer.write_u8(tab.leader_type)?;
        writer.write_u16::<LittleEndian>(0)?; // Reserved 2 bytes per hwplib
    }

    Ok(data)
}

/// Compress data using raw deflate (no zlib header - HWP format requirement)
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    // HWP uses raw deflate compression (not zlib)
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
