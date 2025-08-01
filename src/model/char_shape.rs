use crate::parser::record::Record;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct CharShape {
    pub face_name_ids: [u16; 7],
    pub ratios: [u8; 7],
    pub char_spaces: [i8; 7],
    pub relative_sizes: [u8; 7],
    pub char_offsets: [i8; 7],
    pub base_size: i32,
    pub properties: u32,
    pub shadow_gap_x: i8,
    pub shadow_gap_y: i8,
    pub text_color: u32,
    pub underline_color: u32,
    pub shade_color: u32,
    pub shadow_color: u32,
    pub border_fill_id: u16,
}

impl CharShape {
    pub fn is_bold(&self) -> bool {
        // Bold is bit 0 of properties
        self.properties & 0x1 != 0
    }
    
    pub fn is_italic(&self) -> bool {
        // Italic is bit 1 of properties
        self.properties & 0x2 != 0
    }
    
    pub fn is_underline(&self) -> bool {
        // Underline type is bits 2-4, non-zero means underlined
        (self.properties >> 2) & 0x7 != 0
    }
    
    pub fn is_strikethrough(&self) -> bool {
        // Strikethrough type is bits 5-7, non-zero means strikethrough
        (self.properties >> 5) & 0x7 != 0
    }
    
    pub fn get_outline_type(&self) -> u8 {
        // Outline type is bits 8-10
        ((self.properties >> 8) & 0x7) as u8
    }
    
    pub fn get_shadow_type(&self) -> u8 {
        // Shadow type is bits 11-12
        ((self.properties >> 11) & 0x3) as u8
    }
    
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        // Check minimum size
        if reader.remaining() < 72 {
            return Err(crate::error::HwpError::ParseError(
                format!("CharShape record too small: {} bytes", reader.remaining())
            ));
        }
        
        let mut face_name_ids = [0u16; 7];
        let mut ratios = [0u8; 7];
        let mut char_spaces = [0i8; 7];
        let mut relative_sizes = [0u8; 7];
        let mut char_offsets = [0i8; 7];
        
        for i in 0..7 {
            face_name_ids[i] = reader.read_u16()?;
        }
        for i in 0..7 {
            ratios[i] = reader.read_u8()?;
        }
        for i in 0..7 {
            char_spaces[i] = reader.read_u8()? as i8;
        }
        for i in 0..7 {
            relative_sizes[i] = reader.read_u8()?;
        }
        for i in 0..7 {
            char_offsets[i] = reader.read_u8()? as i8;
        }
        
        Ok(Self {
            face_name_ids,
            ratios,
            char_spaces,
            relative_sizes,
            char_offsets,
            base_size: reader.read_i32()?,
            properties: reader.read_u32()?,
            shadow_gap_x: reader.read_u8()? as i8,
            shadow_gap_y: reader.read_u8()? as i8,
            text_color: reader.read_u32()?,
            underline_color: reader.read_u32()?,
            shade_color: reader.read_u32()?,
            shadow_color: reader.read_u32()?,
            border_fill_id: reader.read_u16()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FaceName {
    pub properties: u8,
    pub font_name: String,
    pub substitute_font_type: u8,
    pub substitute_font_name: String,
    pub panose: Option<[u8; 10]>,
    pub default_font_name: String,
}

impl FaceName {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        if reader.remaining() < 7 {
            return Err(crate::error::HwpError::ParseError(
                "FaceName record too small".to_string()
            ));
        }
        
        let properties = reader.read_u8()?;
        let font_name_len = reader.read_u16()? as usize;
        
        if reader.remaining() < font_name_len * 2 {
            return Err(crate::error::HwpError::ParseError(
                "Invalid font name length".to_string()
            ));
        }
        let font_name = reader.read_string(font_name_len * 2)?;
        
        // Default values for optional fields
        let mut substitute_font_type = 0;
        let mut substitute_font_name = String::new();
        let mut panose = None;
        let mut default_font_name = String::new();
        
        // Read optional fields if available
        if reader.remaining() >= 3 {
            substitute_font_type = reader.read_u8()?;
            if reader.remaining() >= 2 {
                let substitute_font_name_len = reader.read_u16()? as usize;
                if reader.remaining() >= substitute_font_name_len * 2 {
                    substitute_font_name = reader.read_string(substitute_font_name_len * 2)?;
                }
            }
        }
        
        // Read panose if flag is set and data available
        if properties & 0x80 != 0 && reader.remaining() >= 10 {
            let mut p = [0u8; 10];
            for i in 0..10 {
                p[i] = reader.read_u8()?;
            }
            panose = Some(p);
        }
        
        // Read default font name if available
        if reader.remaining() >= 2 {
            let default_font_name_len = reader.read_u16()? as usize;
            if reader.remaining() >= default_font_name_len * 2 {
                default_font_name = reader.read_string(default_font_name_len * 2)?;
            }
        }
        
        Ok(Self {
            properties,
            font_name,
            substitute_font_type,
            substitute_font_name,
            panose,
            default_font_name,
        })
    }
}