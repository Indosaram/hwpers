pub mod crypto;
pub mod error;
pub mod hwpx;
pub mod model;
pub mod parser;
pub mod preview;
pub mod reader;
pub mod render;
pub mod utils;
pub mod writer;

use std::io::{Read, Seek};
use std::path::Path;

pub use crate::crypto::decrypt_distribution_stream;
pub use crate::error::{HwpError, Result};
pub use crate::hwpx::{HwpxReader, HwpxWriter};
pub use crate::model::HwpDocument;
use crate::parser::{body_text::BodyTextParser, doc_info::DocInfoParser, header::FileHeader};
pub use crate::preview::{PreviewImage, PreviewText, SummaryInfo};
use crate::reader::CfbReader;
pub use crate::writer::style;
pub use crate::writer::HwpWriter;

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
        let header_data = reader.read_stream("FileHeader")?;
        let header = FileHeader::parse(header_data)?;

        if header.is_encrypted() {
            return Err(HwpError::UnsupportedVersion(
                "Password-encrypted documents are not supported".to_string(),
            ));
        }

        let distribution_record = if header.is_distribute() {
            Some(Self::read_distribution_record(
                &mut reader,
                header.is_compressed(),
            )?)
        } else {
            None
        };

        let doc_info_data = reader.read_stream("DocInfo")?;
        let doc_info_decrypted =
            Self::decrypt_stream(doc_info_data, &header, distribution_record.as_deref())?;
        let doc_info = DocInfoParser::parse(doc_info_decrypted, header.is_compressed())?;

        let mut body_texts = Vec::new();
        let mut section_idx = 0;

        let stream_prefix = if header.is_distribute() {
            "ViewText/Section"
        } else {
            "BodyText/Section"
        };

        loop {
            let section_name = format!("{stream_prefix}{section_idx}");
            if !reader.stream_exists(&section_name) {
                break;
            }

            let section_data = reader.read_stream(&section_name)?;
            let section_decrypted =
                Self::decrypt_stream(section_data, &header, distribution_record.as_deref())?;
            let body_text = BodyTextParser::parse(section_decrypted, header.is_compressed())?;
            body_texts.push(body_text);

            section_idx += 1;
        }

        if body_texts.is_empty() {
            return Err(HwpError::InvalidFormat(
                "No BodyText sections found".to_string(),
            ));
        }

        let preview_text = Self::read_preview_text(&mut reader).ok();
        let preview_image = Self::read_preview_image(&mut reader).ok();
        let summary_info = Self::read_summary_info(&mut reader).ok();

        Ok(HwpDocument {
            header,
            doc_info,
            body_texts,
            preview_text,
            preview_image,
            summary_info,
        })
    }

    fn read_preview_text<F: Read + Seek>(reader: &mut CfbReader<F>) -> Result<PreviewText> {
        let data = reader.read_stream("PrvText")?;
        PreviewText::from_bytes(&data)
    }

    fn read_preview_image<F: Read + Seek>(reader: &mut CfbReader<F>) -> Result<PreviewImage> {
        let data = reader.read_stream("PrvImage")?;
        Ok(PreviewImage::from_bytes(data))
    }

    fn read_summary_info<F: Read + Seek>(reader: &mut CfbReader<F>) -> Result<SummaryInfo> {
        let data = reader.read_stream("\x05HwpSummaryInformation")?;
        SummaryInfo::from_bytes(&data)
    }

    fn read_distribution_record<F: Read + Seek>(
        reader: &mut CfbReader<F>,
        is_compressed: bool,
    ) -> Result<Vec<u8>> {
        let doc_info_data = reader.read_stream("DocInfo")?;

        let decompressed = if is_compressed {
            crate::utils::decompress(&doc_info_data)?
        } else {
            doc_info_data
        };

        if decompressed.len() < 260 {
            return Err(HwpError::ParseError(
                "DocInfo too short to contain distribution data".to_string(),
            ));
        }

        Ok(decompressed[..260].to_vec())
    }

    fn decrypt_stream(
        data: Vec<u8>,
        _header: &FileHeader,
        distribution_record: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        if let Some(dist_record) = distribution_record {
            if data.len() < 260 {
                return Ok(data);
            }
            let encrypted_data = &data[260..];
            decrypt_distribution_stream(encrypted_data, dist_record)
        } else {
            Ok(data)
        }
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
        let path = test_file_path("test_document.hwp");
        if path.exists() {
            let result = HwpReader::from_file(&path);
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_file_header_signature() {
        let signature = b"HWP Document File";
        assert_eq!(signature.len(), 17);
    }
}
