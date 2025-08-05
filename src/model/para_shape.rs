use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct ParaShape {
    pub properties1: u32,
    pub left_margin: i32,
    pub right_margin: i32,
    pub indent: i32,
    pub top_para_space: i32,
    pub bottom_para_space: i32,
    pub line_space: i32,
    pub tab_def_id: u16,
    pub numbering_id: u16,
    pub border_fill_id: u16,
    pub border_left_space: i16,
    pub border_right_space: i16,
    pub border_top_space: i16,
    pub border_bottom_space: i16,
    pub properties2: u32,
    pub properties3: u32,
    pub line_space_type: u32,
}

impl ParaShape {
    pub fn get_alignment(&self) -> u8 {
        // Alignment is stored in bits 2-4 of properties1
        ((self.properties1 >> 2) & 0x7) as u8
    }

    pub fn get_line_spacing_percent(&self) -> i32 {
        // Line spacing depends on line_space_type
        match self.line_space_type {
            0 => self.line_space, // Percentage (e.g., 160 = 160%)
            1 => self.line_space, // Fixed value in HWP units
            2 => self.line_space, // At least this value
            _ => 100,             // Default 100%
        }
    }
}

impl ParaShape {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 54 {
            return Err(crate::error::HwpError::ParseError(format!(
                "ParaShape record too small: {} bytes",
                reader.remaining()
            )));
        }

        Ok(Self {
            properties1: reader.read_u32()?,
            left_margin: reader.read_i32()?,
            right_margin: reader.read_i32()?,
            indent: reader.read_i32()?,
            top_para_space: reader.read_i32()?,
            bottom_para_space: reader.read_i32()?,
            line_space: reader.read_i32()?,
            tab_def_id: reader.read_u16()?,
            numbering_id: reader.read_u16()?,
            border_fill_id: reader.read_u16()?,
            border_left_space: reader.read_u16()? as i16,
            border_right_space: reader.read_u16()? as i16,
            border_top_space: reader.read_u16()? as i16,
            border_bottom_space: reader.read_u16()? as i16,
            properties2: reader.read_u32()?,
            properties3: reader.read_u32()?,
            line_space_type: reader.read_u32()?,
        })
    }

    /// Create a new default ParaShape for writing
    pub fn new_default() -> Self {
        Self {
            properties1: 0x04,      // Left alignment (bits 2-4 = 001)
            left_margin: 0,
            right_margin: 0,
            indent: 0,
            top_para_space: 0,
            bottom_para_space: 0,
            line_space: 160,        // 160% line spacing
            tab_def_id: 0,
            numbering_id: 0,
            border_fill_id: 0,
            border_left_space: 0,
            border_right_space: 0,
            border_top_space: 0,
            border_bottom_space: 0,
            properties2: 0,
            properties3: 0,
            line_space_type: 0,     // Percentage
        }
    }
}