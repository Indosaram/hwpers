use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct ListHeader {
    pub paragraph_count: i32,
    pub properties: u32,
    pub text_width: i32,
    pub text_height: i32,
    pub padding: [u8; 8],
}

impl ListHeader {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        if reader.remaining() < 20 {
            return Err(crate::error::HwpError::ParseError(
                format!("ListHeader record too small: {} bytes", reader.remaining())
            ));
        }
        
        let paragraph_count = reader.read_i32()?;
        let properties = reader.read_u32()?;
        let text_width = reader.read_i32()?;
        let text_height = reader.read_i32()?;
        
        let mut padding = [0u8; 8];
        if reader.remaining() >= 8 {
            for i in 0..8 {
                padding[i] = reader.read_u8()?;
            }
        }
        
        Ok(Self {
            paragraph_count,
            properties,
            text_width,
            text_height,
            padding,
        })
    }
    
    pub fn is_multi_column(&self) -> bool {
        (self.properties & 0x01) != 0
    }
    
    pub fn has_line_wrap(&self) -> bool {
        (self.properties & 0x02) != 0
    }
    
    pub fn is_editable_at_form_mode(&self) -> bool {
        (self.properties & 0x04) != 0
    }
}