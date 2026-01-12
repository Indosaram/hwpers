use crate::error::{HwpError, Result};

#[derive(Debug, Clone)]
pub struct PreviewText {
    pub content: String,
}

impl PreviewText {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Ok(Self {
                content: String::new(),
            });
        }

        if !data.len().is_multiple_of(2) {
            return Err(HwpError::ParseError(
                "PreviewText data length must be even (UTF-16LE)".to_string(),
            ));
        }

        let u16_data: Vec<u16> = data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let content = String::from_utf16_lossy(&u16_data)
            .trim_end_matches('\0')
            .to_string();

        Ok(Self { content })
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_preview_text() {
        let result = PreviewText::from_bytes(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_utf16le_decode() {
        let data: Vec<u8> = "Hello"
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        let result = PreviewText::from_bytes(&data).unwrap();
        assert_eq!(result.text(), "Hello");
    }
}
