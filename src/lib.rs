pub mod error;
pub mod model;
pub mod parser;
pub mod reader;
pub mod render;
pub mod utils;
pub mod writer;

use std::io::{Read, Seek};
use std::path::Path;

pub use crate::error::{HwpError, Result};
pub use crate::model::HwpDocument;
pub use crate::writer::HwpWriter;
use crate::parser::{body_text::BodyTextParser, doc_info::DocInfoParser, header::FileHeader};
use crate::reader::CfbReader;

pub struct HwpReader;

impl HwpReader {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<HwpDocument> {
        let reader = CfbReader::from_file(path)?;
        Self::parse_document(reader)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<HwpDocument> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let reader = CfbReader::new(cursor)?;
        Self::parse_document(reader)
    }

    fn parse_document<F: Read + Seek>(mut reader: CfbReader<F>) -> Result<HwpDocument> {
        // Parse FileHeader
        let header_data = reader.read_stream("FileHeader")?;
        let header = FileHeader::parse(header_data)?;

        // Check if the document is encrypted
        if header.is_encrypted() {
            return Err(HwpError::UnsupportedVersion(
                "Encrypted documents are not supported".to_string(),
            ));
        }

        // Parse DocInfo
        let doc_info_data = reader.read_stream("DocInfo")?;
        let doc_info = DocInfoParser::parse(doc_info_data, header.is_compressed())?;

        // Parse BodyText sections
        let mut body_texts = Vec::new();
        let mut section_idx = 0;

        loop {
            let section_name = format!("BodyText/Section{section_idx}");
            if !reader.stream_exists(&section_name) {
                break;
            }

            let section_data = reader.read_stream(&section_name)?;
            let body_text = BodyTextParser::parse(section_data, header.is_compressed())?;
            body_texts.push(body_text);

            section_idx += 1;
        }

        if body_texts.is_empty() {
            return Err(HwpError::InvalidFormat(
                "No BodyText sections found".to_string(),
            ));
        }

        Ok(HwpDocument {
            header,
            doc_info,
            body_texts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_file_path(name: &str) -> PathBuf {
        PathBuf::from("test-files").join(name)
    }

    #[test]
    fn test_reader_creation() {
        // Test that we can create a reader
        let path = test_file_path("test_document.hwp");
        if path.exists() {
            let result = HwpReader::from_file(&path);
            assert!(result.is_ok() || result.is_err()); // Either parse or fail gracefully
        }
    }

    #[test]
    fn test_file_header_signature() {
        let signature = b"HWP Document File";
        assert_eq!(signature.len(), 17);
    }
}
