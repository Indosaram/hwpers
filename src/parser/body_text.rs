use crate::error::Result;
use crate::model::{
    CtrlHeader, ListHeader, PageDef, ParaCharShape, ParaLineSeg, ParaText, Paragraph, Section,
    SectionDef,
};
use crate::parser::record::{HwpTag, Record};
use crate::reader::StreamReader;
use crate::utils::compression::decompress_stream;

pub struct BodyTextParser;

impl BodyTextParser {
    pub fn parse(data: Vec<u8>, is_compressed: bool) -> Result<BodyText> {
        let data = if is_compressed {
            decompress_stream(&data)?
        } else {
            data
        };

        let mut reader = StreamReader::new(data);
        let mut sections = Vec::new();
        let mut current_section = Section::default();
        let mut current_paragraph: Option<Paragraph> = None;

        let mut first_section = true;

        while reader.remaining() >= 4 {
            // Need at least 4 bytes for record header
            let record = match Record::parse(&mut reader) {
                Ok(r) => r,
                Err(_) => break, // Stop parsing on error
            };

            match HwpTag::from_u16(record.tag_id()) {
                // Page Definition - only appears once at the beginning
                Some(HwpTag::PageDef) => {
                    current_section.page_def = PageDef::from_record(&record).ok();
                }

                // SectionDefine (0x42) - Actually marks paragraph start in this document
                Some(HwpTag::SectionDefine) => {
                    // First one is the actual section definition
                    if first_section {
                        current_section.section_def = SectionDef::from_record(&record).ok();
                        first_section = false;
                    } else {
                        // Subsequent ones mark new paragraphs
                        if let Some(para) = current_paragraph.take() {
                            current_section.paragraphs.push(para);
                        }
                        current_paragraph = Some(Paragraph::default());
                    }
                }

                // Tag 0x43 - Contains text content
                Some(HwpTag::ColumnDefine) => {
                    if let Some(ref mut para) = current_paragraph {
                        if let Ok(text) = ParaText::from_record(&record) {
                            para.text = Some(text);
                        }
                    }
                }

                // TableControl (0x44) - Contains paragraph properties
                Some(HwpTag::TableControl) => {
                    if let Some(ref mut para) = current_paragraph {
                        // Try to parse as paragraph header
                        if let Ok(new_para) = Paragraph::from_header_record(&record) {
                            // Copy properties from parsed paragraph
                            para.control_mask = new_para.control_mask;
                            para.para_shape_id = new_para.para_shape_id;
                            para.style_id = new_para.style_id;
                            para.column_type = new_para.column_type;
                            para.char_shape_count = new_para.char_shape_count;
                            para.range_tag_count = new_para.range_tag_count;
                            para.line_align_count = new_para.line_align_count;
                            para.instance_id = new_para.instance_id;
                        }
                    }
                }

                // Standard paragraph records (if they exist)
                Some(HwpTag::ParaHeader) => {
                    if let Some(para) = current_paragraph.take() {
                        current_section.paragraphs.push(para);
                    }
                    match Paragraph::from_header_record(&record) {
                        Ok(para) => current_paragraph = Some(para),
                        Err(_) => {} // Skip invalid paragraph headers
                    }
                }
                Some(HwpTag::ParaText) => {
                    if let Some(ref mut para) = current_paragraph {
                        para.text = Some(ParaText::from_record(&record)?);
                    }
                }
                Some(HwpTag::ParaCharShape) => {
                    if let Some(ref mut para) = current_paragraph {
                        para.char_shapes = ParaCharShape::from_record(&record).ok();
                    }
                }
                Some(HwpTag::ParaLineSeg) => {
                    if let Some(ref mut para) = current_paragraph {
                        para.line_segments = ParaLineSeg::from_record(&record).ok();
                    }
                }

                // Control Records
                Some(HwpTag::ListHeader) => {
                    if let Some(ref mut para) = current_paragraph {
                        para.list_header = ListHeader::from_record(&record).ok();
                    }
                }
                Some(HwpTag::CtrlHeader) => {
                    if let Some(ref mut para) = current_paragraph {
                        para.ctrl_header = CtrlHeader::from_record(&record).ok();
                    }
                }

                _ => {
                    // Skip other tags for now
                }
            }
        }

        // Add last paragraph and section
        if let Some(para) = current_paragraph {
            current_section.paragraphs.push(para);
        }
        // Always add the section even if empty - there's at least one section
        sections.push(current_section);

        Ok(BodyText { sections })
    }
}

#[derive(Debug, Default)]
pub struct BodyText {
    pub sections: Vec<Section>,
}

impl BodyText {
    pub fn extract_text(&self) -> String {
        let mut result = String::new();

        for section in &self.sections {
            for para in &section.paragraphs {
                if let Some(ref text) = para.text {
                    result.push_str(&text.content);
                    result.push('\n');
                }
            }
        }

        result
    }
}
