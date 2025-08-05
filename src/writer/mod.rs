pub mod serializer;

use crate::error::{HwpError, Result};
use crate::model::{
    HwpDocument, 
    document::DocumentProperties,
    paragraph::{Section, Paragraph, ParaText},
    char_shape::{CharShape, FaceName},
    para_shape::ParaShape,
    style::Style,
    border_fill::BorderFill,
    tab_def::TabDef,

};
use crate::parser::{
    header::FileHeader,
    doc_info::DocInfo,
    body_text::BodyText,
};
use std::path::Path;

pub struct HwpWriter {
    document: HwpDocument,
    current_section_idx: usize,
}

impl HwpWriter {
    /// Create a new HWP writer with minimal default structure
    pub fn new() -> Self {
        let header = Self::create_default_header();
        let doc_info = Self::create_default_doc_info();
        let body_texts = vec![Self::create_default_body_text()];
        
        Self {
            document: HwpDocument {
                header,
                doc_info,
                body_texts,
            },
            current_section_idx: 0,
        }
    }

    /// Add a paragraph with plain text
    pub fn add_paragraph(&mut self, text: &str) -> Result<()> {
        let para_text = ParaText {
            content: text.to_string(),
        };

        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id: 0, // Use default paragraph shape
            style_id: 0,      // Use default style
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: None,
        };

        // Get the current section and add paragraph
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Convert the document to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serializer::serialize_document(&self.document)
    }

    /// Save to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)
            .map_err(|e| HwpError::Io(e))?;
        Ok(())
    }

    /// Create default file header
    fn create_default_header() -> FileHeader {
        FileHeader::new_default()
    }

    /// Create default document info with minimal required data
    fn create_default_doc_info() -> DocInfo {
        DocInfo {
            properties: Some(DocumentProperties::default()),
            face_names: vec![
                FaceName::new_default("맑은 고딕".to_string()),
            ],
            char_shapes: vec![
                CharShape::new_default(), // Default 12pt font
            ],
            para_shapes: vec![
                ParaShape::new_default(), // Default left-aligned paragraph
            ],
            styles: vec![
                Style::new_default(),
            ],
            border_fills: vec![
                BorderFill::new_default(),
            ],
            tab_defs: vec![
                TabDef::new_default(),
            ],
            numberings: Vec::new(),
            bullets: Vec::new(),
            bin_data: Vec::new(),
        }
    }

    /// Create default body text with one empty section
    fn create_default_body_text() -> BodyText {
        let section = Section {
            paragraphs: Vec::new(),
            section_def: None,
            page_def: None,
        };

        BodyText {
            sections: vec![section],
        }
    }
}
impl HwpWriter {
    /// Create a writer from an existing HwpDocument
    pub fn from_document(document: HwpDocument) -> Self {
        Self {
            document,
            current_section_idx: 0,
        }
    }

    /// Get a reference to the underlying document
    pub fn document(&self) -> &HwpDocument {
        &self.document
    }
}

impl Default for HwpWriter {
    fn default() -> Self {
        Self::new()
    }
}