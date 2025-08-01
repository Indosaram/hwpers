use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct BinData {
    pub properties: u16,
    pub abs_name: String,
    pub rel_name: String,
    pub bin_id: u16,
    pub extension: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum BinDataType {
    Link = 0,
    Embedding = 1,
    Storage = 2,
}

impl BinData {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 6 {
            return Err(crate::error::HwpError::ParseError(format!(
                "BinData record too small: {} bytes",
                reader.remaining()
            )));
        }

        let properties = reader.read_u16()?;

        // Read absolute file name
        let abs_name_len = reader.read_u16()? as usize;
        let abs_name = if reader.remaining() >= abs_name_len * 2 {
            reader.read_string(abs_name_len * 2)?
        } else {
            String::new()
        };

        // Read relative file name
        let rel_name_len = reader.read_u16()? as usize;
        let rel_name = if reader.remaining() >= rel_name_len * 2 {
            reader.read_string(rel_name_len * 2)?
        } else {
            String::new()
        };

        if reader.remaining() < 2 {
            return Err(crate::error::HwpError::ParseError(
                "Insufficient data for BinData ID".to_string(),
            ));
        }

        let bin_id = reader.read_u16()?;

        // Read extension
        let extension_len = reader.read_u16()? as usize;
        let extension = if reader.remaining() >= extension_len * 2 {
            reader.read_string(extension_len * 2)?
        } else {
            String::new()
        };

        // Read remaining data as binary content
        let mut data = Vec::new();
        while reader.remaining() > 0 {
            data.push(reader.read_u8()?);
        }

        Ok(Self {
            properties,
            abs_name,
            rel_name,
            bin_id,
            extension,
            data,
        })
    }

    pub fn get_type(&self) -> BinDataType {
        match self.properties & 0x03 {
            0 => BinDataType::Link,
            1 => BinDataType::Embedding,
            2 => BinDataType::Storage,
            _ => BinDataType::Link, // Default fallback
        }
    }

    pub fn is_compressed(&self) -> bool {
        (self.properties & 0x04) != 0
    }

    pub fn is_access_by_path(&self) -> bool {
        (self.properties & 0x08) != 0
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self.extension.to_lowercase().as_str(),
            "bmp" | "gif" | "jpg" | "jpeg" | "png" | "tif" | "tiff" | "wmf" | "emf"
        )
    }

    pub fn is_ole_object(&self) -> bool {
        self.extension.to_lowercase() == "ole"
    }

    pub fn get_data(&self) -> Result<Vec<u8>> {
        if self.is_compressed() {
            // Decompress the data if needed
            crate::utils::compression::decompress_stream(&self.data)
        } else {
            Ok(self.data.clone())
        }
    }
}
