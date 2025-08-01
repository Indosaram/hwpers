use crate::error::Result;
use crate::reader::StreamReader;
use crate::parser::record::{Record, HwpTag};
use crate::model::{DocumentProperties, FaceName, CharShape, ParaShape};
use crate::model::style::Style;
use crate::model::border_fill::BorderFill;
use crate::model::tab_def::TabDef;
use crate::model::numbering::{Numbering, Bullet};
use crate::model::bin_data::BinData;
use crate::utils::compression::decompress_stream;

pub struct DocInfoParser;

impl DocInfoParser {
    pub fn parse(data: Vec<u8>, is_compressed: bool) -> Result<DocInfo> {
        let data = if is_compressed {
            decompress_stream(&data)?
        } else {
            data
        };
        
        let mut reader = StreamReader::new(data);
        let mut doc_info = DocInfo::default();
        
        while reader.remaining() >= 4 {  // Need at least 4 bytes for record header
            let record = match Record::parse(&mut reader) {
                Ok(r) => r,
                Err(_) => break,  // Stop parsing on error
            };
            
            match HwpTag::from_u16(record.tag_id()) {
                Some(HwpTag::DocumentProperties) => {
                    doc_info.properties = Some(DocumentProperties::from_record(&record)?);
                }
                Some(HwpTag::FaceName) => {
                    doc_info.face_names.push(FaceName::from_record(&record)?);
                }
                Some(HwpTag::CharShape) => {
                    doc_info.char_shapes.push(CharShape::from_record(&record)?);
                }
                Some(HwpTag::ParaShape) => {
                    doc_info.para_shapes.push(ParaShape::from_record(&record)?);
                }
                Some(HwpTag::Style) => {
                    doc_info.styles.push(Style::from_record(&record)?);
                }
                Some(HwpTag::BorderFill) => {
                    doc_info.border_fills.push(BorderFill::from_record(&record)?);
                }
                Some(HwpTag::TabDef) => {
                    doc_info.tab_defs.push(TabDef::from_record(&record)?);
                }
                Some(HwpTag::Numbering) => {
                    doc_info.numberings.push(Numbering::from_record(&record)?);
                }
                Some(HwpTag::Bullet) => {
                    doc_info.bullets.push(Bullet::from_record(&record)?);
                }
                Some(HwpTag::BinData) => {
                    doc_info.bin_data.push(BinData::from_record(&record)?);
                }
                _ => {
                    // Skip unknown or unimplemented tags
                }
            }
        }
        
        Ok(doc_info)
    }
}

#[derive(Debug, Default)]
pub struct DocInfo {
    pub properties: Option<DocumentProperties>,
    pub face_names: Vec<FaceName>,
    pub char_shapes: Vec<CharShape>,
    pub para_shapes: Vec<ParaShape>,
    pub styles: Vec<Style>,
    pub border_fills: Vec<BorderFill>,
    pub tab_defs: Vec<TabDef>,
    pub numberings: Vec<Numbering>,
    pub bullets: Vec<Bullet>,
    pub bin_data: Vec<BinData>,
}