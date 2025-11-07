use crate::error::Result;
use crate::model::HwpDocument;
use crate::utils::encoding::string_to_utf16le;
use byteorder::{LittleEndian, WriteBytesExt};
use cfb::CompoundFile;
use flate2::{write::ZlibEncoder, Compression};
use std::io::{Cursor, Write};

/// Serialize an HWP document to bytes
pub fn serialize_document(document: &HwpDocument) -> Result<Vec<u8>> {
    // Use template-based approach to maintain 512-byte sectors
    // Embed the template file at compile time to avoid runtime file access issues
    let template_data = include_bytes!("../minimal_base_template.hwp").to_vec();

    // Use the template as base
    let mut cursor = Cursor::new(template_data);
    {
        let mut cfb = CompoundFile::open(&mut cursor)
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;

        // Clear existing streams and recreate with new data
        // Note: cfb crate doesn't support deleting streams, so we'll overwrite them

        // Serialize FileHeader - open existing stream instead of creating
        let header_data = serialize_file_header(&document.header)?;
        let mut header_stream = cfb
            .open_stream("FileHeader")
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
        header_stream.set_len(0)?; // Clear existing content
        header_stream.write_all(&header_data)?;

        // Serialize DocInfo - open existing stream
        let doc_info_data = serialize_doc_info(&document.doc_info)?;
        let compressed_doc_info = if document.header.is_compressed() {
            compress_data(&doc_info_data)?
        } else {
            doc_info_data
        };
        let mut doc_info_stream = cfb
            .open_stream("DocInfo")
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
        doc_info_stream.set_len(0)?;
        doc_info_stream.write_all(&compressed_doc_info)?;

        // BodyText directory should already exist in template

        // Serialize BodyText sections
        for (i, body_text) in document.body_texts.iter().enumerate() {
            let section_data = serialize_body_text(body_text)?;
            let compressed_section = if document.header.is_compressed() {
                compress_data(&section_data)?
            } else {
                section_data
            };

            let section_path = format!("BodyText/Section{i}");
            let mut section_stream = cfb
                .open_stream(&section_path)
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            section_stream.set_len(0)?;
            section_stream.write_all(&compressed_section)?;
        }

        // Update HwpSummaryInformation stream if it exists
        if cfb.exists("HwpSummaryInformation") {
            let summary_data = create_summary_information()?;
            let mut summary_stream = cfb
                .open_stream("HwpSummaryInformation")
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            summary_stream.set_len(0)?;
            summary_stream.write_all(&summary_data)?;
        }

        // Update PrvText stream if it exists
        if cfb.exists("PrvText") {
            let prv_text = create_preview_text(document)?;
            let mut prv_stream = cfb
                .open_stream("PrvText")
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            prv_stream.set_len(0)?;
            prv_stream.write_all(&prv_text)?;
        }

        // Update DocOptions/_LinkDoc stream if it exists
        if cfb.exists("DocOptions/_LinkDoc") {
            let doc_options = create_doc_options()?;
            let mut options_stream = cfb
                .open_stream("DocOptions/_LinkDoc")
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            options_stream.set_len(0)?;
            options_stream.write_all(&doc_options)?;
        }

        // ViewText stream doesn't exist in template, skip it

        // Update PrvImage stream if it exists
        if cfb.exists("PrvImage") {
            let prv_image = create_prv_image()?;
            let mut prv_image_stream = cfb
                .open_stream("PrvImage")
                .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            prv_image_stream.set_len(0)?;
            prv_image_stream.write_all(&prv_image)?;
        }

        // Create BinData storage if there are images
        if !document.doc_info.bin_data.is_empty() {
            // Ensure BinData storage exists
            if !cfb.exists("BinData") {
                cfb.create_storage("BinData")
                    .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
            }

            // Write each binary data item
            for bin_data in &document.doc_info.bin_data {
                let stream_name =
                    format!("BinData/BIN{:04}.{}", bin_data.bin_id, bin_data.extension);
                let mut stream = cfb
                    .create_stream(&stream_name)
                    .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
                stream.write_all(&bin_data.data)?;
            }
        }

        // Scripts storage should already exist in template

        // Flush the CFB file
        cfb.flush()
            .map_err(|e| crate::error::HwpError::Io(std::io::Error::other(e)))?;
    }

    Ok(cursor.into_inner())
}

/// Serialize FileHeader to bytes
fn serialize_file_header(header: &crate::parser::header::FileHeader) -> Result<Vec<u8>> {
    Ok(header.to_bytes())
}

/// Serialize DocInfo to bytes
fn serialize_doc_info(doc_info: &crate::parser::doc_info::DocInfo) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write document properties if available
    if let Some(props) = &doc_info.properties {
        write_record(&mut writer, 0x10, &serialize_document_properties(props)?)?;
    }

    // Write ID mappings (required for compatibility)
    write_record(&mut writer, 0x11, &serialize_id_mappings()?)?;

    // Write face names
    for face_name in &doc_info.face_names {
        write_record(&mut writer, 0x13, &serialize_face_name(face_name)?)?;
    }

    // Write character shapes
    for char_shape in &doc_info.char_shapes {
        write_record(&mut writer, 0x15, &serialize_char_shape(char_shape)?)?;
    }

    // Write paragraph shapes
    for para_shape in &doc_info.para_shapes {
        write_record(&mut writer, 0x19, &serialize_para_shape(para_shape)?)?;
    }

    // Write styles
    for style in &doc_info.styles {
        write_record(&mut writer, 0x1A, &serialize_style(style)?)?;
    }

    // Write border fills
    for border_fill in &doc_info.border_fills {
        write_record(&mut writer, 0x14, &serialize_border_fill(border_fill)?)?;
    }

    // Write tab definitions
    for tab_def in &doc_info.tab_defs {
        write_record(&mut writer, 0x16, &serialize_tab_def(tab_def)?)?;
    }

    Ok(data)
}

/// Serialize BodyText to bytes
fn serialize_body_text(body_text: &crate::parser::body_text::BodyText) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    for (section_idx, section) in body_text.sections.iter().enumerate() {
        // Write section define for first section
        if section_idx == 0 {
            if let Some(_section_def) = &section.section_def {
                // Serialize section definition
                let section_data = Vec::new(); // TODO: properly serialize section def
                write_record(&mut writer, 0x42, &section_data)?;
            } else {
                // Write empty section define
                write_record(&mut writer, 0x42, &[])?;
            }
        }

        // Write page definition if available
        if let Some(page_def) = &section.page_def {
            write_record(&mut writer, 0x57, &serialize_page_def(page_def)?)?;
        }

        for paragraph in &section.paragraphs {
            // Write paragraph header record (0x50)
            write_record(&mut writer, 0x50, &serialize_paragraph_header(paragraph)?)?;

            // Write paragraph text record (0x51) if text exists
            if let Some(text) = &paragraph.text {
                write_record(&mut writer, 0x51, &serialize_paragraph_text(text)?)?;
            }

            // Write character shape record (0x52) if exists
            if let Some(char_shapes) = &paragraph.char_shapes {
                write_record(&mut writer, 0x52, &serialize_para_char_shapes(char_shapes)?)?;
            }

            // Write hyperlink as ParaRangeTag if exists
            if !paragraph.hyperlinks.is_empty() {
                // Hyperlinks are stored as ParaRangeTag (0x54)
                for hyperlink in &paragraph.hyperlinks {
                    write_record(
                        &mut writer,
                        0x54,
                        &serialize_para_range_tag_hyperlink(hyperlink)?,
                    )?;
                }
            }

            // Write picture control if exists
            if let Some(picture) = &paragraph.picture_data {
                // Pictures use CtrlHeader (0x55)
                write_record(&mut writer, 0x55, &serialize_picture_control(picture)?)?;
            }
        }

        // Write header/footer controls after section
        if let Some(page_def) = &section.page_def {
            // Write header controls
            for header in page_def.header_footer.headers() {
                write_record(
                    &mut writer,
                    0x55,
                    &serialize_header_footer_control(header, true)?,
                )?;
            }

            // Write footer controls
            for footer in page_def.header_footer.footers() {
                write_record(
                    &mut writer,
                    0x55,
                    &serialize_header_footer_control(footer, false)?,
                )?;
            }
        }
    }

    Ok(data)
}

/// Serialize paragraph header (0x50)
fn serialize_paragraph_header(paragraph: &crate::model::paragraph::Paragraph) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write paragraph header
    writer.write_u32::<LittleEndian>(paragraph.control_mask)?;
    writer.write_u16::<LittleEndian>(paragraph.para_shape_id)?;
    writer.write_u8(paragraph.style_id)?;
    writer.write_u8(paragraph.column_type)?;
    writer.write_u16::<LittleEndian>(paragraph.char_shape_count)?;
    writer.write_u16::<LittleEndian>(paragraph.range_tag_count)?;
    writer.write_u16::<LittleEndian>(paragraph.line_align_count)?;
    writer.write_u32::<LittleEndian>(paragraph.instance_id)?;

    // Reserved/Unknown bytes (needed for compatibility)
    writer.write_u32::<LittleEndian>(0)?; // Reserved

    Ok(data)
}

/// Serialize paragraph text (0x51)
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
fn write_record<W: Write>(writer: &mut W, tag: u16, data: &[u8]) -> Result<()> {
    let level = 0u16;
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

/// Serialize document properties
fn serialize_document_properties(
    props: &crate::model::document::DocumentProperties,
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // Write basic properties (simplified)
    writer.write_u16::<LittleEndian>(props.section_count)?;
    writer.write_u16::<LittleEndian>(props.total_page_count as u16)?;

    Ok(data)
}

/// Serialize ID mappings (HWPTAG_ID_MAPPINGS = 0x11)
fn serialize_id_mappings() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // ID mappings format
    // This record contains mappings between various IDs used in the document

    // Bin Data Count
    writer.write_u32::<LittleEndian>(0)?; // No binary data

    // Korean Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one Korean font

    // English Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one English font

    // Hanja Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one Hanja font

    // Japanese Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one Japanese font

    // Other Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one other font

    // Symbol Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one symbol font

    // User Font Count
    writer.write_u32::<LittleEndian>(1)?; // At least one user font

    // Border Fill Count
    writer.write_u32::<LittleEndian>(1)?; // At least one border fill

    // Char Shape Count
    writer.write_u32::<LittleEndian>(1)?; // At least one char shape

    // Tab Def Count
    writer.write_u32::<LittleEndian>(1)?; // At least one tab def

    // Numbering Count
    writer.write_u32::<LittleEndian>(0)?; // No numbering

    // Bullet Count
    writer.write_u32::<LittleEndian>(0)?; // No bullets

    // Para Shape Count
    writer.write_u32::<LittleEndian>(1)?; // At least one para shape

    // Style Count
    writer.write_u32::<LittleEndian>(1)?; // At least one style

    // Memo Shape Count (if version >= 5.0.2.1)
    writer.write_u32::<LittleEndian>(0)?; // No memo shapes

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

/// Compress data using zlib
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed_data = encoder.finish()?;
    Ok(compressed_data)
}

/// Create HwpSummaryInformation stream
fn create_summary_information() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = Cursor::new(&mut data);

    // HWP Summary Information format
    // This is a simplified version with minimal required fields

    // Write format version
    writer.write_u32::<LittleEndian>(0x00000005)?; // Version 5

    // Write creation date/time (current time as FILETIME)
    let filetime = 0x01D9A0B0A0000000u64; // Placeholder timestamp
    writer.write_u64::<LittleEndian>(filetime)?;

    // Write last saved date/time
    writer.write_u64::<LittleEndian>(filetime)?;

    // Write revision number
    writer.write_u32::<LittleEndian>(1)?;

    // Write total editing time in minutes
    writer.write_u32::<LittleEndian>(0)?;

    // Write last printed date/time
    writer.write_u64::<LittleEndian>(0)?;

    // Write creation date/time (duplicate)
    writer.write_u64::<LittleEndian>(filetime)?;

    // Write page count
    writer.write_u32::<LittleEndian>(1)?;

    // Write word count
    writer.write_u32::<LittleEndian>(0)?;

    // Write character count
    writer.write_u32::<LittleEndian>(0)?;

    // Write security (0 = none)
    writer.write_u32::<LittleEndian>(0)?;

    Ok(data)
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

/// Create PrvImage stream (preview image)
fn create_prv_image() -> Result<Vec<u8>> {
    // PrvImage contains a preview image of the document
    // For now, we'll create an empty/minimal placeholder
    // In a real implementation, this would contain a bitmap or other image format

    // Return empty data for now - Hangul will handle missing preview
    Ok(Vec::new())
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
