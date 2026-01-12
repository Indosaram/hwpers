use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

use crate::error::{HwpError, Result};
use crate::model::{
    para_char_shape::{CharPositionShape, ParaCharShape},
    CharShape, DocumentProperties, FaceName, HwpDocument, ParaShape, ParaText, Paragraph, Section,
};
use crate::parser::body_text::BodyText;
use crate::parser::doc_info::DocInfo;
use crate::parser::header::FileHeader;

use super::xml_types::{self, HcfVersion, Head, Run, Section as XmlSection, XmlParagraph};

pub struct HwpxReader;

impl HwpxReader {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<HwpDocument> {
        let file = File::open(path).map_err(HwpError::Io)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<HwpDocument> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        Self::from_reader(cursor)
    }

    fn from_reader<R: Read + std::io::Seek>(reader: R) -> Result<HwpDocument> {
        let mut archive = ZipArchive::new(reader)
            .map_err(|e| HwpError::InvalidFormat(format!("Invalid HWPX archive: {}", e)))?;

        let version = Self::read_version(&mut archive)?;
        let head = Self::read_header(&mut archive)?;
        let sections = Self::read_sections(&mut archive)?;

        let header = Self::create_file_header(&version);
        let doc_info = Self::convert_head_to_doc_info(&head);
        let body_texts = Self::convert_sections_to_body_texts(&sections, &head);

        Ok(HwpDocument {
            header,
            doc_info,
            body_texts,
            preview_text: None,
            preview_image: None,
            summary_info: None,
        })
    }

    fn read_version<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<HcfVersion> {
        let xml = Self::read_xml_file(archive, "version.xml")?;
        xml_types::parse_version(&xml)
            .map_err(|e| HwpError::ParseError(format!("Failed to parse version.xml: {}", e)))
    }

    fn read_header<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<Head> {
        let xml = Self::read_xml_file(archive, "Contents/header.xml")?;
        xml_types::parse_head(&xml)
            .map_err(|e| HwpError::ParseError(format!("Failed to parse header.xml: {}", e)))
    }

    fn read_sections<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<XmlSection>> {
        let mut sections = Vec::new();
        let mut idx = 0;

        loop {
            let filename = format!("Contents/section{}.xml", idx);
            match Self::read_xml_file(archive, &filename) {
                Ok(xml) => {
                    let section = xml_types::parse_section(&xml).map_err(|e| {
                        HwpError::ParseError(format!("Failed to parse {}: {}", filename, e))
                    })?;
                    sections.push(section);
                    idx += 1;
                }
                Err(_) => break,
            }
        }

        if sections.is_empty() {
            return Err(HwpError::InvalidFormat(
                "No section files found in HWPX".to_string(),
            ));
        }

        Ok(sections)
    }

    fn read_xml_file<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
        filename: &str,
    ) -> Result<String> {
        let mut file = archive
            .by_name(filename)
            .map_err(|_| HwpError::NotFound(format!("File not found in archive: {}", filename)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(HwpError::Io)?;

        Ok(contents)
    }

    fn create_file_header(version: &HcfVersion) -> FileHeader {
        let mut header = FileHeader::new_default();

        let version_str = version
            .version
            .as_deref()
            .or(version.xml_version.as_deref())
            .unwrap_or("5.0");

        let version_parts: Vec<u8> = version_str
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if !version_parts.is_empty() {
            header.set_version(
                version_parts.first().copied().unwrap_or(5),
                version_parts.get(1).copied().unwrap_or(0),
                version_parts.get(2).copied().unwrap_or(0),
                version_parts.get(3).copied().unwrap_or(0),
            );
        } else if let Some(ref major) = version.major {
            let major_val: u8 = major.parse().unwrap_or(5);
            let minor_val: u8 = version
                .minor
                .as_ref()
                .and_then(|m| m.parse().ok())
                .unwrap_or(0);
            header.set_version(major_val, minor_val, 0, 0);
        }

        header
    }

    fn convert_head_to_doc_info(head: &Head) -> DocInfo {
        let mut doc_info = DocInfo::default();

        if let Some(ref_list) = &head.ref_list {
            if let Some(fontfaces) = &ref_list.fontfaces {
                for fontface in &fontfaces.items {
                    for font in &fontface.fonts {
                        doc_info.face_names.push(FaceName {
                            properties: 0,
                            font_name: font.face.clone(),
                            substitute_font_type: 0,
                            substitute_font_name: String::new(),
                            panose: None,
                            default_font_name: String::new(),
                        });
                    }
                }
            }

            if let Some(char_props) = &ref_list.char_properties {
                for char_pr in &char_props.items {
                    let mut char_shape = CharShape::new_default();
                    if let Some(height) = char_pr.height {
                        char_shape.base_size = (height * 10) as i32;
                    }
                    if char_pr.bold == Some(true) {
                        char_shape.properties |= 0x01;
                    }
                    if char_pr.italic == Some(true) {
                        char_shape.properties |= 0x02;
                    }
                    if let Some(ref underline) = char_pr.underline {
                        if underline != "NONE" {
                            char_shape.properties |= 0x04;
                        }
                    }
                    if let Some(ref strikeout) = char_pr.strikeout {
                        if strikeout != "NONE" {
                            char_shape.properties |= 0x08;
                        }
                    }
                    if let Some(ref color_str) = char_pr.text_color {
                        if let Some(color) = Self::parse_color(color_str) {
                            char_shape.text_color = color;
                        }
                    }
                    doc_info.char_shapes.push(char_shape);
                }
            }

            if let Some(para_props) = &ref_list.para_properties {
                for para_pr in &para_props.items {
                    let mut para_shape = ParaShape::new_default();
                    if let Some(ref align) = para_pr.align {
                        let align_value: u32 = match align.as_str() {
                            "left" => 0,
                            "center" => 1,
                            "right" => 2,
                            "justify" => 3,
                            _ => 0,
                        };
                        para_shape.properties1 =
                            (para_shape.properties1 & !0x1C) | (align_value << 2);
                    }
                    doc_info.para_shapes.push(para_shape);
                }
            }
        }

        if let Some(begin_num) = &head.begin_num {
            let mut props = DocumentProperties::new();
            props.page_start_number = begin_num.page.unwrap_or(1) as u16;
            props.footnote_start_number = begin_num.footnote.unwrap_or(1) as u16;
            props.endnote_start_number = begin_num.endnote.unwrap_or(1) as u16;
            props.picture_start_number = begin_num.pic.unwrap_or(1) as u16;
            props.table_start_number = begin_num.tbl.unwrap_or(1) as u16;
            props.equation_start_number = begin_num.equation.unwrap_or(1) as u16;
            doc_info.properties = Some(props);
        }

        doc_info
    }

    fn convert_sections_to_body_texts(sections: &[XmlSection], _head: &Head) -> Vec<BodyText> {
        sections
            .iter()
            .map(|xml_section| {
                let paragraphs: Vec<Paragraph> = xml_section
                    .paragraphs
                    .iter()
                    .map(|xml_para| Self::convert_paragraph(xml_para))
                    .collect();

                BodyText {
                    sections: vec![Section {
                        paragraphs,
                        section_def: None,
                        page_def: None,
                    }],
                }
            })
            .collect()
    }

    fn convert_paragraph(xml_para: &XmlParagraph) -> Paragraph {
        let (text_content, char_positions) = Self::extract_text_and_char_shapes(&xml_para.runs);

        let char_shapes = if char_positions.is_empty() {
            None
        } else {
            Some(ParaCharShape { char_positions })
        };

        Paragraph {
            text: if text_content.is_empty() {
                None
            } else {
                Some(ParaText {
                    content: text_content,
                })
            },
            control_mask: 0,
            para_shape_id: xml_para.para_pr_id_ref.unwrap_or(0) as u16,
            style_id: xml_para.style_id_ref.unwrap_or(0) as u8,
            char_shapes,
            ..Default::default()
        }
    }

    fn parse_color(color_str: &str) -> Option<u32> {
        let color_str = color_str.trim();
        if color_str.starts_with('#') && color_str.len() == 7 {
            u32::from_str_radix(&color_str[1..], 16).ok()
        } else if color_str == "none" {
            Some(0x000000)
        } else {
            None
        }
    }

    fn extract_text_and_char_shapes(runs: &[Run]) -> (String, Vec<CharPositionShape>) {
        let mut text_content = String::new();
        let mut char_positions = Vec::new();
        let mut current_pos: u32 = 0;
        let mut last_char_pr_id: Option<u32> = None;

        for run in runs {
            if let Some(ref text) = run.text {
                let char_pr_id = run.char_pr_id_ref.unwrap_or(0);

                if last_char_pr_id != Some(char_pr_id) && char_pr_id > 0 {
                    char_positions.push(CharPositionShape {
                        position: current_pos,
                        char_shape_id: char_pr_id as u16,
                    });
                    last_char_pr_id = Some(char_pr_id);
                }

                current_pos += text.chars().count() as u32;
                text_content.push_str(text);
            }
        }

        (text_content, char_positions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hwpx_reader_nonexistent_file() {
        let result = HwpxReader::from_file("nonexistent.hwpx");
        assert!(result.is_err());
    }
}
