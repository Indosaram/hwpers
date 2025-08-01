use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Default)]
pub struct Section {
    pub paragraphs: Vec<Paragraph>,
    pub section_def: Option<crate::model::SectionDef>,
    pub page_def: Option<crate::model::PageDef>,
}

#[derive(Debug, Default)]
pub struct Paragraph {
    pub text: Option<ParaText>,
    pub control_mask: u32,
    pub para_shape_id: u16,
    pub style_id: u8,
    pub column_type: u8,
    pub char_shape_count: u16,
    pub range_tag_count: u16,
    pub line_align_count: u16,
    pub instance_id: u32,
    pub char_shapes: Option<crate::model::ParaCharShape>,
    pub line_segments: Option<crate::model::ParaLineSeg>,
    pub list_header: Option<crate::model::ListHeader>,
    pub ctrl_header: Option<crate::model::CtrlHeader>,
}

impl Paragraph {
    pub fn from_header_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        // For tag 0x42, we have a simpler structure
        if record.tag_id() == 0x42 {
            // This is a minimal paragraph header
            return Ok(Self::default());
        }

        // Standard paragraph header (tag 0x50)
        if reader.remaining() < 18 {
            return Err(crate::error::HwpError::ParseError(
                "Insufficient data for paragraph header".to_string(),
            ));
        }

        Ok(Self {
            control_mask: reader.read_u32()?,
            para_shape_id: reader.read_u16()?,
            style_id: reader.read_u8()?,
            column_type: reader.read_u8()?,
            char_shape_count: reader.read_u16()?,
            range_tag_count: reader.read_u16()?,
            line_align_count: reader.read_u16()?,
            instance_id: reader.read_u32()?,
            ..Default::default()
        })
    }

    pub fn parse_char_shapes(&mut self, _record: &Record) -> Result<()> {
        // Character shape parsing logic would go here
        // For now, we'll skip the implementation
        Ok(())
    }
}

#[derive(Debug)]
pub struct ParaText {
    pub content: String,
}

impl ParaText {
    pub fn from_record(record: &Record) -> Result<Self> {
        // Check if this is a table marker record
        if record.tag_id() == 0x43 && record.data.len() == 18 {
            // Check for the specific table marker pattern
            if record.data[0] == 0x0B
                && record.data[1] == 0x00
                && record.data[2] == 0x20
                && record.data[3] == 0x6C
                && record.data[4] == 0x62
                && record.data[5] == 0x74
            {
                // This is a table marker, return empty text
                return Ok(Self {
                    content: String::new(),
                });
            }
        }

        let mut reader = record.data_reader();
        let mut content = String::new();
        let mut chars = Vec::new();

        // Read all UTF-16LE characters
        while reader.remaining() >= 2 {
            let ch = reader.read_u16()?;
            chars.push(ch);
        }

        // Process characters based on record type
        if record.tag_id() == 0x43 {
            // For tag 0x43, we need special handling
            let mut i = 0;
            while i < chars.len() {
                let ch = chars[i];

                // Check for control sequences
                if ch == 0x0002 && i + 1 < chars.len() {
                    // This might be a control sequence, check what follows
                    let next = chars[i + 1];
                    if next == 0x6364 || next == 0x6C64 {
                        // Skip this metadata sequence
                        // Look for the end (0x0000 0x0000 pattern)
                        while i < chars.len()
                            && !(i + 1 < chars.len() && chars[i] == 0 && chars[i + 1] == 0)
                        {
                            i += 1;
                        }
                        // Skip the zeros
                        while i < chars.len() && chars[i] == 0 {
                            i += 1;
                        }
                        continue;
                    }
                }

                // Process normal characters
                match ch {
                    0x0000 => {
                        // Skip null characters
                    }
                    0x0001..=0x0008 => {
                        // Skip other control characters
                    }
                    0x0009 => {
                        // Tab character - check if this is followed by form field markers
                        if i + 2 < chars.len()
                            && chars[i + 2] == 0x0000
                            && (chars[i + 1] == 0x0480 || chars[i + 1] == 0x0264)
                        {
                            // ɤ followed by null
                            // This is a form field marker, skip the entire sequence
                            // Skip until we find normal text again (not tab, space, or control chars)
                            while i < chars.len()
                                && (chars[i] == 0x0009
                                    || chars[i] == 0x0020
                                    || chars[i] == 0x0480
                                    || chars[i] == 0x0100
                                    || chars[i] == 0x0264
                                    || chars[i] == 0x0000
                                    || chars[i] == 0x0001)
                            {
                                i += 1;
                            }
                            i -= 1; // Adjust because loop will increment
                            continue;
                        } else {
                            content.push('\t'); // Regular tab
                        }
                    }
                    0x000A => content.push('\n'), // Line feed
                    0x000D => content.push('\r'), // Carriage return
                    0x000E..=0x001F => {
                        // Skip other control characters
                    }
                    0x0264 => {
                        // ɤ character - check if part of form field
                        if i + 1 < chars.len() && chars[i + 1] == 0x0100 {
                            // Skip form field marker
                            i += 1; // Skip the Ā
                            continue;
                        } else {
                            // Regular character
                            if let Some(unicode_char) = std::char::from_u32(ch as u32) {
                                content.push(unicode_char);
                            }
                        }
                    }
                    0x0480 => {
                        // Ҁ character - check if part of form field
                        if i + 1 < chars.len() && chars[i + 1] == 0x0100 {
                            // Skip form field marker
                            i += 1; // Skip the Ā
                            continue;
                        } else {
                            // Regular character
                            if let Some(unicode_char) = std::char::from_u32(ch as u32) {
                                content.push(unicode_char);
                            }
                        }
                    }
                    0xF020..=0xF07F => {
                        // Extended control characters - skip
                    }
                    _ => {
                        // Regular characters
                        if let Some(unicode_char) = std::char::from_u32(ch as u32) {
                            content.push(unicode_char);
                        }
                    }
                }
                i += 1;
            }
        } else {
            // Standard text processing for other tags
            for &ch in &chars {
                match ch {
                    0x0000..=0x001F => {
                        // Control characters
                        match ch {
                            0x000A => content.push('\n'), // Line feed
                            0x000D => content.push('\r'), // Carriage return
                            0x0009 => content.push('\t'), // Tab
                            _ => {}                       // Skip other control characters
                        }
                    }
                    0xF020..=0xF07F => {
                        // Extended control characters - skip for now
                    }
                    _ => {
                        // Regular characters
                        if let Some(unicode_char) = std::char::from_u32(ch as u32) {
                            content.push(unicode_char);
                        }
                    }
                }
            }
        }

        Ok(Self { content })
    }
}
