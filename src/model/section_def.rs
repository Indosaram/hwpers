use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct SectionDef {
    pub properties: u32,
    pub column_gap: u16,
    pub vertical_line_align: u16,
    pub horizontal_line_align: u16,
    pub default_tab_stop: u32,
    pub numbering_shape_id: u16,
    pub page_starting_number: u16,
    pub image_starting_number: u16,
    pub table_starting_number: u16,
    pub equation_starting_number: u16,
    pub default_language: u16,
}

impl SectionDef {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 26 {
            return Err(crate::error::HwpError::ParseError(format!(
                "SectionDef record too small: {} bytes",
                reader.remaining()
            )));
        }

        Ok(Self {
            properties: reader.read_u32()?,
            column_gap: reader.read_u16()?,
            vertical_line_align: reader.read_u16()?,
            horizontal_line_align: reader.read_u16()?,
            default_tab_stop: reader.read_u32()?,
            numbering_shape_id: reader.read_u16()?,
            page_starting_number: reader.read_u16()?,
            image_starting_number: reader.read_u16()?,
            table_starting_number: reader.read_u16()?,
            equation_starting_number: reader.read_u16()?,
            default_language: if reader.remaining() >= 2 {
                reader.read_u16()?
            } else {
                0
            },
        })
    }

    pub fn column_count(&self) -> u16 {
        ((self.properties >> 20) & 0xFF) as u16 + 1
    }

    pub fn is_hide_header(&self) -> bool {
        (self.properties & 0x01) != 0
    }

    pub fn is_hide_footer(&self) -> bool {
        (self.properties & 0x02) != 0
    }

    pub fn is_hide_page_number(&self) -> bool {
        (self.properties & 0x04) != 0
    }

    /// Create a new default SectionDef for writing
    pub fn new_default() -> Self {
        Self {
            properties: 0,
            column_gap: 567, // 2mm gap
            vertical_line_align: 0,
            horizontal_line_align: 0,
            default_tab_stop: 5669, // 20mm tab stops
            numbering_shape_id: 0,
            page_starting_number: 1,
            image_starting_number: 1,
            table_starting_number: 1,
            equation_starting_number: 1,
            default_language: 0x0412, // Korean
        }
    }

    /// Serialize to bytes for HWP file
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        writer.write_u32::<LittleEndian>(self.properties).unwrap();
        writer.write_u16::<LittleEndian>(self.column_gap).unwrap();
        writer
            .write_u16::<LittleEndian>(self.vertical_line_align)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.horizontal_line_align)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.default_tab_stop)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.numbering_shape_id)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.page_starting_number)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.image_starting_number)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.table_starting_number)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.equation_starting_number)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.default_language)
            .unwrap();

        data
    }
}
