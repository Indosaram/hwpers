use crate::error::Result;
use crate::parser::body_text::BodyText;
use crate::parser::doc_info::DocInfo;
use crate::parser::header::FileHeader;
use crate::parser::record::Record;

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
        self.doc_info
            .bin_data
            .iter()
            .filter(|bd| bd.is_image())
            .collect()
    }

    /// Get all OLE objects in the document
    pub fn get_ole_objects(&self) -> Vec<&crate::model::bin_data::BinData> {
        self.doc_info
            .bin_data
            .iter()
            .filter(|bd| bd.is_ole_object())
            .collect()
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
    // Extended metadata fields
    pub document_title: Option<String>,
    pub document_subject: Option<String>,
    pub document_author: Option<String>,
    pub document_company: Option<String>,
    pub document_keywords: Option<String>,
    pub creation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_print_date: Option<chrono::DateTime<chrono::Utc>>,
    pub revision_number: u32,
    pub edit_time_minutes: u32,
    pub password_protected: bool,
    pub read_only: bool,
    pub compressed: bool,
}

impl DocumentProperties {
    /// Create new document properties with current timestamp
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            section_count: 1,
            page_start_number: 1,
            footnote_start_number: 1,
            endnote_start_number: 1,
            picture_start_number: 1,
            table_start_number: 1,
            equation_start_number: 1,
            total_character_count: 0,
            hangul_character_count: 0,
            english_character_count: 0,
            hanja_character_count: 0,
            japanese_character_count: 0,
            other_character_count: 0,
            symbol_character_count: 0,
            space_character_count: 0,
            total_page_count: 1,
            total_word_count: 0,
            line_count: 0,
            document_title: None,
            document_subject: None,
            document_author: None,
            document_company: None,
            document_keywords: None,
            creation_date: Some(now),
            last_modified_date: Some(now),
            last_print_date: None,
            revision_number: 1,
            edit_time_minutes: 0,
            password_protected: false,
            read_only: false,
            compressed: true, // Default to compressed for smaller files
        }
    }

    /// Set document title
    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.document_title = Some(title);
        self.update_modified_date();
        self
    }

    /// Set document author
    pub fn set_author(&mut self, author: String) -> &mut Self {
        self.document_author = Some(author);
        self.update_modified_date();
        self
    }

    /// Set document subject
    pub fn set_subject(&mut self, subject: String) -> &mut Self {
        self.document_subject = Some(subject);
        self.update_modified_date();
        self
    }

    /// Set document company
    pub fn set_company(&mut self, company: String) -> &mut Self {
        self.document_company = Some(company);
        self.update_modified_date();
        self
    }

    /// Set document keywords
    pub fn set_keywords(&mut self, keywords: String) -> &mut Self {
        self.document_keywords = Some(keywords);
        self.update_modified_date();
        self
    }

    /// Update last modified date to current time
    pub fn update_modified_date(&mut self) {
        self.last_modified_date = Some(chrono::Utc::now());
        self.revision_number += 1;
    }

    /// Mark document as printed
    pub fn mark_printed(&mut self) {
        self.last_print_date = Some(chrono::Utc::now());
    }

    /// Set password protection
    pub fn set_password_protected(&mut self, protected: bool) -> &mut Self {
        self.password_protected = protected;
        self.update_modified_date();
        self
    }

    /// Set read-only status
    pub fn set_read_only(&mut self, read_only: bool) -> &mut Self {
        self.read_only = read_only;
        self.update_modified_date();
        self
    }

    /// Set compression
    pub fn set_compressed(&mut self, compressed: bool) -> &mut Self {
        self.compressed = compressed;
        self.update_modified_date();
        self
    }

    /// Calculate and update character counts from text content
    pub fn calculate_character_counts(&mut self, text: &str) {
        self.total_character_count = 0;
        self.hangul_character_count = 0;
        self.english_character_count = 0;
        self.hanja_character_count = 0;
        self.japanese_character_count = 0;
        self.other_character_count = 0;
        self.symbol_character_count = 0;
        self.space_character_count = 0;

        for ch in text.chars() {
            self.total_character_count += 1;

            if ch.is_whitespace() {
                self.space_character_count += 1;
            } else if is_hangul(ch) {
                self.hangul_character_count += 1;
            } else if ch.is_ascii_alphabetic() {
                self.english_character_count += 1;
            } else if is_hanja(ch) {
                self.hanja_character_count += 1;
            } else if is_japanese(ch) {
                self.japanese_character_count += 1;
            } else if ch.is_ascii_punctuation() || ch.is_ascii_digit() {
                self.symbol_character_count += 1;
            } else {
                self.other_character_count += 1;
            }
        }

        // Update word count (approximate)
        self.total_word_count = text.split_whitespace().count() as u32;
        
        // Update line count
        self.line_count = text.lines().count() as u32;
    }

    /// Add character counts from additional text
    pub fn add_character_counts(&mut self, text: &str) {
        let mut temp_props = DocumentProperties::default();
        temp_props.calculate_character_counts(text);
        
        self.total_character_count += temp_props.total_character_count;
        self.hangul_character_count += temp_props.hangul_character_count;
        self.english_character_count += temp_props.english_character_count;
        self.hanja_character_count += temp_props.hanja_character_count;
        self.japanese_character_count += temp_props.japanese_character_count;
        self.other_character_count += temp_props.other_character_count;
        self.symbol_character_count += temp_props.symbol_character_count;
        self.space_character_count += temp_props.space_character_count;
        self.total_word_count += temp_props.total_word_count;
        self.line_count += temp_props.line_count;
    }

    /// Convert to HWP format bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        // Write basic properties
        writer.write_u16::<LittleEndian>(self.section_count).unwrap();
        writer.write_u16::<LittleEndian>(self.page_start_number).unwrap();
        writer.write_u16::<LittleEndian>(self.footnote_start_number).unwrap();
        writer.write_u16::<LittleEndian>(self.endnote_start_number).unwrap();
        writer.write_u16::<LittleEndian>(self.picture_start_number).unwrap();
        writer.write_u16::<LittleEndian>(self.table_start_number).unwrap();
        writer.write_u16::<LittleEndian>(self.equation_start_number).unwrap();

        // Write character counts
        writer.write_u32::<LittleEndian>(self.total_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.hangul_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.english_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.hanja_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.japanese_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.other_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.symbol_character_count).unwrap();
        writer.write_u32::<LittleEndian>(self.space_character_count).unwrap();

        // Write document metrics
        writer.write_u32::<LittleEndian>(self.total_page_count).unwrap();
        writer.write_u32::<LittleEndian>(self.total_word_count).unwrap();
        writer.write_u32::<LittleEndian>(self.line_count).unwrap();

        // Write revision info
        writer.write_u32::<LittleEndian>(self.revision_number).unwrap();
        writer.write_u32::<LittleEndian>(self.edit_time_minutes).unwrap();

        // Write flags
        let mut flags = 0u32;
        if self.password_protected { flags |= 0x01; }
        if self.read_only { flags |= 0x02; }
        if self.compressed { flags |= 0x04; }
        writer.write_u32::<LittleEndian>(flags).unwrap();

        data
    }
}

/// Check if character is Hangul (Korean)
fn is_hangul(ch: char) -> bool {
    matches!(ch as u32,
        0xAC00..=0xD7AF | // Hangul Syllables
        0x1100..=0x11FF | // Hangul Jamo
        0x3130..=0x318F | // Hangul Compatibility Jamo
        0xA960..=0xA97F | // Hangul Jamo Extended-A
        0xD7B0..=0xD7FF   // Hangul Jamo Extended-B
    )
}

/// Check if character is Hanja (Chinese characters used in Korean)
fn is_hanja(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF | // CJK Unified Ideographs
        0x3400..=0x4DBF | // CJK Extension A
        0x20000..=0x2A6DF | // CJK Extension B
        0x2A700..=0x2B73F | // CJK Extension C
        0x2B740..=0x2B81F | // CJK Extension D
        0x2B820..=0x2CEAF | // CJK Extension E
        0x2CEB0..=0x2EBEF   // CJK Extension F
    )
}

/// Check if character is Japanese (Hiragana, Katakana)
fn is_japanese(ch: char) -> bool {
    matches!(ch as u32,
        0x3040..=0x309F | // Hiragana
        0x30A0..=0x30FF | // Katakana
        0x31F0..=0x31FF   // Katakana Phonetic Extensions
    )
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
    pub fn get_char_formatting<'a>(
        &self,
        document: &'a HwpDocument,
    ) -> Option<&'a crate::model::CharShape> {
        self.char_shape_id
            .and_then(|id| document.get_char_shape(id as usize))
    }

    /// Get the paragraph formatting for this text
    pub fn get_para_formatting<'a>(
        &self,
        document: &'a HwpDocument,
    ) -> Option<&'a crate::model::ParaShape> {
        self.para_shape_id
            .and_then(|id| document.get_para_shape(id as usize))
    }

    /// Get the style for this text
    pub fn get_style<'a>(
        &self,
        document: &'a HwpDocument,
    ) -> Option<&'a crate::model::style::Style> {
        self.style_id.and_then(|id| document.get_style(id as usize))
    }
}

impl DocumentProperties {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        let size = reader.remaining();

        // The record size varies. Read what's available
        let mut props = Self::default();

        if size >= 2 {
            props.section_count = reader.read_u16()?;
        }
        if size >= 4 {
            props.page_start_number = reader.read_u16()?;
        }
        if size >= 6 {
            props.footnote_start_number = reader.read_u16()?;
        }
        if size >= 8 {
            props.endnote_start_number = reader.read_u16()?;
        }
        if size >= 10 {
            props.picture_start_number = reader.read_u16()?;
        }
        if size >= 12 {
            props.table_start_number = reader.read_u16()?;
        }
        if size >= 14 {
            props.equation_start_number = reader.read_u16()?;
        }
        if size >= 18 {
            props.total_character_count = reader.read_u32()?;
        }
        if size >= 22 {
            props.hangul_character_count = reader.read_u32()?;
        }
        if size >= 26 {
            props.english_character_count = reader.read_u32()?;
        }
        if size >= 30 {
            props.hanja_character_count = reader.read_u32()?;
        }
        if size >= 34 {
            props.japanese_character_count = reader.read_u32()?;
        }
        if size >= 38 {
            props.other_character_count = reader.read_u32()?;
        }
        if size >= 42 {
            props.symbol_character_count = reader.read_u32()?;
        }
        if size >= 46 {
            props.space_character_count = reader.read_u32()?;
        }
        if size >= 50 {
            props.total_page_count = reader.read_u32()?;
        }
        if size >= 54 {
            props.total_word_count = reader.read_u32()?;
        }
        if size >= 58 {
            props.line_count = reader.read_u32()?;
        }

        Ok(props)
    }
}
