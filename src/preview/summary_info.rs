use crate::error::{HwpError, Result};

#[derive(Debug, Clone, Default)]
pub struct SummaryInfo {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub keywords: Option<String>,
    pub comments: Option<String>,
    pub last_saved_by: Option<String>,
    pub revision_number: Option<String>,
    pub creation_date: Option<i64>,
    pub last_saved_date: Option<i64>,
    pub page_count: Option<i32>,
    pub word_count: Option<i32>,
    pub char_count: Option<i32>,
}

const PROPERTY_SET_HEADER_SIZE: usize = 28;
const PROPERTY_ID_TITLE: u32 = 0x02;
const PROPERTY_ID_SUBJECT: u32 = 0x03;
const PROPERTY_ID_AUTHOR: u32 = 0x04;
const PROPERTY_ID_KEYWORDS: u32 = 0x05;
const PROPERTY_ID_COMMENTS: u32 = 0x06;
const PROPERTY_ID_LAST_SAVED_BY: u32 = 0x08;
const PROPERTY_ID_REVISION_NUMBER: u32 = 0x09;
const PROPERTY_ID_CREATION_DATE: u32 = 0x0C;
const PROPERTY_ID_LAST_SAVED_DATE: u32 = 0x0D;
const PROPERTY_ID_PAGE_COUNT: u32 = 0x0E;
const PROPERTY_ID_WORD_COUNT: u32 = 0x0F;
const PROPERTY_ID_CHAR_COUNT: u32 = 0x10;

const VT_LPSTR: u32 = 0x1E;
const VT_FILETIME: u32 = 0x40;
const VT_I4: u32 = 0x03;

impl SummaryInfo {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < PROPERTY_SET_HEADER_SIZE {
            return Err(HwpError::ParseError(
                "SummaryInfo data too short".to_string(),
            ));
        }

        let byte_order = u16::from_le_bytes([data[0], data[1]]);
        if byte_order != 0xFFFE {
            return Err(HwpError::ParseError(format!(
                "Invalid OLE property byte order: 0x{:04X}",
                byte_order
            )));
        }

        let mut info = SummaryInfo::default();

        if data.len() < 48 {
            return Ok(info);
        }

        let section_offset = u32::from_le_bytes([data[44], data[45], data[46], data[47]]) as usize;

        if data.len() < section_offset + 8 {
            return Ok(info);
        }

        let property_count = u32::from_le_bytes([
            data[section_offset + 4],
            data[section_offset + 5],
            data[section_offset + 6],
            data[section_offset + 7],
        ]) as usize;

        let properties_start = section_offset + 8;

        for i in 0..property_count {
            let prop_offset = properties_start + i * 8;
            if data.len() < prop_offset + 8 {
                break;
            }

            let prop_id = u32::from_le_bytes([
                data[prop_offset],
                data[prop_offset + 1],
                data[prop_offset + 2],
                data[prop_offset + 3],
            ]);
            let prop_value_offset = u32::from_le_bytes([
                data[prop_offset + 4],
                data[prop_offset + 5],
                data[prop_offset + 6],
                data[prop_offset + 7],
            ]) as usize;

            let absolute_offset = section_offset + prop_value_offset;
            if data.len() < absolute_offset + 8 {
                continue;
            }

            let prop_type = u32::from_le_bytes([
                data[absolute_offset],
                data[absolute_offset + 1],
                data[absolute_offset + 2],
                data[absolute_offset + 3],
            ]);

            match prop_id {
                PROPERTY_ID_TITLE => {
                    info.title = Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_SUBJECT => {
                    info.subject = Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_AUTHOR => {
                    info.author = Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_KEYWORDS => {
                    info.keywords = Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_COMMENTS => {
                    info.comments = Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_LAST_SAVED_BY => {
                    info.last_saved_by =
                        Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_REVISION_NUMBER => {
                    info.revision_number =
                        Self::read_string_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_CREATION_DATE => {
                    info.creation_date =
                        Self::read_filetime_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_LAST_SAVED_DATE => {
                    info.last_saved_date =
                        Self::read_filetime_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_PAGE_COUNT => {
                    info.page_count = Self::read_i32_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_WORD_COUNT => {
                    info.word_count = Self::read_i32_property(data, absolute_offset, prop_type);
                }
                PROPERTY_ID_CHAR_COUNT => {
                    info.char_count = Self::read_i32_property(data, absolute_offset, prop_type);
                }
                _ => {}
            }
        }

        Ok(info)
    }

    fn read_string_property(data: &[u8], offset: usize, prop_type: u32) -> Option<String> {
        if prop_type != VT_LPSTR {
            return None;
        }

        if data.len() < offset + 8 {
            return None;
        }

        let str_len = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if str_len == 0 || data.len() < offset + 8 + str_len {
            return None;
        }

        let str_bytes = &data[offset + 8..offset + 8 + str_len];
        let trimmed = str_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        String::from_utf8(trimmed).ok()
    }

    fn read_filetime_property(data: &[u8], offset: usize, prop_type: u32) -> Option<i64> {
        if prop_type != VT_FILETIME {
            return None;
        }

        if data.len() < offset + 12 {
            return None;
        }

        let low = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as i64;
        let high = u32::from_le_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]) as i64;

        Some((high << 32) | low)
    }

    fn read_i32_property(data: &[u8], offset: usize, prop_type: u32) -> Option<i32> {
        if prop_type != VT_I4 {
            return None;
        }

        if data.len() < offset + 8 {
            return None;
        }

        Some(i32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]))
    }

    pub fn has_metadata(&self) -> bool {
        self.title.is_some()
            || self.author.is_some()
            || self.subject.is_some()
            || self.keywords.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_summary_info() {
        let result = SummaryInfo::from_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_byte_order() {
        let data = vec![0x00, 0x00];
        let result = SummaryInfo::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_minimal_valid_header() {
        let mut data = vec![0xFE, 0xFF];
        data.extend(vec![0u8; 46]);
        let result = SummaryInfo::from_bytes(&data);
        assert!(result.is_ok());
        assert!(!result.unwrap().has_metadata());
    }
}
