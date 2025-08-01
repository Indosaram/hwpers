use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct Style {
    pub name: String,
    pub english_name: String,
    pub properties: u8,
    pub next_style_id: u8,
    pub lang_id: u16,
    pub para_shape_id: u16,
    pub char_shape_id: u16,
}

impl Style {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 10 {
            return Err(crate::error::HwpError::ParseError(format!(
                "Style record too small: {} bytes",
                reader.remaining()
            )));
        }

        // Read name length and name
        let name_len = reader.read_u16()? as usize;
        if reader.remaining() < name_len * 2 {
            return Err(crate::error::HwpError::ParseError(
                "Invalid style name length".to_string(),
            ));
        }
        let name = reader.read_string(name_len * 2)?;

        // Read English name length and name
        let english_name = if reader.remaining() >= 2 {
            let english_name_len = reader.read_u16()? as usize;
            if reader.remaining() >= english_name_len * 2 {
                reader.read_string(english_name_len * 2)?
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Read remaining fields
        if reader.remaining() < 8 {
            return Err(crate::error::HwpError::ParseError(
                "Insufficient data for style properties".to_string(),
            ));
        }

        let properties = reader.read_u8()?;
        let next_style_id = reader.read_u8()?;
        let lang_id = reader.read_u16()?;
        let para_shape_id = reader.read_u16()?;
        let char_shape_id = reader.read_u16()?;

        Ok(Self {
            name,
            english_name,
            properties,
            next_style_id,
            lang_id,
            para_shape_id,
            char_shape_id,
        })
    }
}
