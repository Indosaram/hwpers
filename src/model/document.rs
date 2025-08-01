use crate::parser::header::FileHeader;
use crate::parser::doc_info::DocInfo;
use crate::parser::body_text::BodyText;
use crate::parser::record::Record;
use crate::error::Result;

#[derive(Debug)]
pub struct HwpDocument {
    pub header: FileHeader,
    pub doc_info: DocInfo,
    pub body_texts: Vec<BodyText>,
}


impl HwpDocument {
    pub fn sections(&self) -> impl Iterator<Item = &crate::model::Section> {
        self.body_texts.iter().flat_map(|bt| bt.sections.iter())
    }
    
    pub fn extract_text(&self) -> String {
        let mut result = String::new();
        
        for body_text in &self.body_texts {
            result.push_str(&body_text.extract_text());
        }
        
        result
    }

    
    /// Get a character shape by ID
    pub fn get_char_shape(&self, id: usize) -> Option<&crate::model::CharShape> {
        self.doc_info.char_shapes.get(id)
    }
    
    /// Get a paragraph shape by ID
    pub fn get_para_shape(&self, id: usize) -> Option<&crate::model::ParaShape> {
        self.doc_info.para_shapes.get(id)
    }
    
    /// Get a style by ID
    pub fn get_style(&self, id: usize) -> Option<&crate::model::style::Style> {
        self.doc_info.styles.get(id)
    }
    
    /// Get a border fill by ID
    pub fn get_border_fill(&self, id: usize) -> Option<&crate::model::border_fill::BorderFill> {
        self.doc_info.border_fills.get(id)
    }
    
    /// Get a tab definition by ID
    pub fn get_tab_def(&self, id: usize) -> Option<&crate::model::tab_def::TabDef> {
        self.doc_info.tab_defs.get(id)
    }
    
    /// Get a numbering definition by ID
    pub fn get_numbering(&self, id: usize) -> Option<&crate::model::numbering::Numbering> {
        self.doc_info.numberings.get(id)
    }
    
    /// Get a bullet definition by ID
    pub fn get_bullet(&self, id: usize) -> Option<&crate::model::numbering::Bullet> {
        self.doc_info.bullets.get(id)
    }
    
    /// Get binary data by ID
    pub fn get_bin_data(&self, id: u16) -> Option<&crate::model::bin_data::BinData> {
        self.doc_info.bin_data.iter().find(|bd| bd.bin_id == id)
    }
    
    /// Get a font face by ID
    pub fn get_face_name(&self, id: usize) -> Option<&crate::model::FaceName> {
        self.doc_info.face_names.get(id)
    }
    
    /// Get document properties
    pub fn get_properties(&self) -> Option<&crate::model::DocumentProperties> {
        self.doc_info.properties.as_ref()
    }
    
    /// Get bin data list for embedded objects
    pub fn get_bin_data_list(&self) -> Option<&Vec<crate::model::bin_data::BinData>> {
        if self.doc_info.bin_data.is_empty() {
            None
        } else {
            Some(&self.doc_info.bin_data)
        }
    }
    
    /// Extract text with formatting information
    pub fn extract_formatted_text(&self) -> Vec<FormattedText> {
        let mut result = Vec::new();
        
        for body_text in &self.body_texts {
            for section in &body_text.sections {
                for paragraph in &section.paragraphs {
                    if let Some(para_text) = &paragraph.text {
                        let formatted = FormattedText {
                            text: para_text.content.clone(),
                            char_shape_id: None, // Paragraph doesn't directly have char_shape_id
                            para_shape_id: Some(paragraph.para_shape_id),
                            style_id: Some(paragraph.style_id),
                        };
                        result.push(formatted);
                    }
                }
            }
        }
        
        result
    }
    
    /// Get all images in the document
    pub fn get_images(&self) -> Vec<&crate::model::bin_data::BinData> {
        self.doc_info.bin_data.iter().filter(|bd| bd.is_image()).collect()
    }
    
    /// Get all OLE objects in the document
    pub fn get_ole_objects(&self) -> Vec<&crate::model::bin_data::BinData> {
        self.doc_info.bin_data.iter().filter(|bd| bd.is_ole_object()).collect()
    }
}

#[derive(Debug, Default)]
pub struct DocumentProperties {
    pub section_count: u16,
    pub page_start_number: u16,
    pub footnote_start_number: u16,
    pub endnote_start_number: u16,
    pub picture_start_number: u16,
    pub table_start_number: u16,
    pub equation_start_number: u16,
    pub total_character_count: u32,
    pub hangul_character_count: u32,
    pub english_character_count: u32,
    pub hanja_character_count: u32,
    pub japanese_character_count: u32,
    pub other_character_count: u32,
    pub symbol_character_count: u32,
    pub space_character_count: u32,
    pub total_page_count: u32,
    pub total_word_count: u32,
    pub line_count: u32,
}

#[derive(Debug, Clone)]
pub struct FormattedText {
    pub text: String,
    pub char_shape_id: Option<u16>,
    pub para_shape_id: Option<u16>,
    pub style_id: Option<u8>,
}

impl FormattedText {
    /// Get the character formatting for this text
    pub fn get_char_formatting<'a>(&self, document: &'a HwpDocument) -> Option<&'a crate::model::CharShape> {
        self.char_shape_id.and_then(|id| document.get_char_shape(id as usize))
    }
    
    /// Get the paragraph formatting for this text
    pub fn get_para_formatting<'a>(&self, document: &'a HwpDocument) -> Option<&'a crate::model::ParaShape> {
        self.para_shape_id.and_then(|id| document.get_para_shape(id as usize))
    }
    
    /// Get the style for this text
    pub fn get_style<'a>(&self, document: &'a HwpDocument) -> Option<&'a crate::model::style::Style> {
        self.style_id.and_then(|id| document.get_style(id as usize))
    }
}

impl DocumentProperties {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        let size = reader.remaining();
        
        // The record size varies. Read what's available
        let mut props = Self::default();
        
        if size >= 2 { props.section_count = reader.read_u16()?; }
        if size >= 4 { props.page_start_number = reader.read_u16()?; }
        if size >= 6 { props.footnote_start_number = reader.read_u16()?; }
        if size >= 8 { props.endnote_start_number = reader.read_u16()?; }
        if size >= 10 { props.picture_start_number = reader.read_u16()?; }
        if size >= 12 { props.table_start_number = reader.read_u16()?; }
        if size >= 14 { props.equation_start_number = reader.read_u16()?; }
        if size >= 18 { props.total_character_count = reader.read_u32()?; }
        if size >= 22 { props.hangul_character_count = reader.read_u32()?; }
        if size >= 26 { props.english_character_count = reader.read_u32()?; }
        if size >= 30 { props.hanja_character_count = reader.read_u32()?; }
        if size >= 34 { props.japanese_character_count = reader.read_u32()?; }
        if size >= 38 { props.other_character_count = reader.read_u32()?; }
        if size >= 42 { props.symbol_character_count = reader.read_u32()?; }
        if size >= 46 { props.space_character_count = reader.read_u32()?; }
        if size >= 50 { props.total_page_count = reader.read_u32()?; }
        if size >= 54 { props.total_word_count = reader.read_u32()?; }
        if size >= 58 { props.line_count = reader.read_u32()?; }
        
        Ok(props)
    }
}