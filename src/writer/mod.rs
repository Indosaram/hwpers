pub mod serializer;
pub mod style;

use crate::error::{HwpError, Result};
use crate::model::{
    border_fill::BorderFill,
    char_shape::{CharShape, FaceName},
    document::DocumentProperties,
    para_shape::ParaShape,
    paragraph::{ParaText, Paragraph, Section},
    style::Style,
    tab_def::TabDef,
    HwpDocument,
};
use crate::parser::{body_text::BodyText, doc_info::DocInfo, header::FileHeader};
use std::path::Path;

pub struct HwpWriter {
    document: HwpDocument,
    current_section_idx: usize,
    /// Instance ID counter for generating unique IDs
    next_instance_id: u32,
    /// Current list state
    current_list_type: Option<style::ListType>,
    current_list_level: u32,
    current_list_index: u32,
    list_stack: Vec<(style::ListType, u32)>,
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
            next_instance_id: 1,
            current_list_type: None,
            current_list_level: 0,
            current_list_index: 0,
            list_stack: Vec::new(),
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
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks: Vec::new(),
        };

        // Get the current section and add paragraph
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a paragraph with custom text style
    pub fn add_paragraph_with_style(&mut self, text: &str, style: &style::TextStyle) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};

        let para_text = ParaText {
            content: text.to_string(),
        };

        // Get or create font for the style
        let face_name_id = if let Some(font_name) = &style.font_name {
            self.ensure_font(font_name)?
        } else {
            0 // Use default font
        };

        // Create character shape from style
        let char_shape = style.to_char_shape(face_name_id);
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create character shape information for the paragraph
        let char_shapes = ParaCharShape {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        };

        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: self.next_instance_id(),
            char_shapes: Some(char_shapes),
            line_segments: None,
            list_header: None,
            ctrl_header: None,
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks: Vec::new(),
        };

        // Get the current section and add paragraph
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a heading with specified level (1-6)
    pub fn add_heading(&mut self, text: &str, level: u8) -> Result<()> {
        let heading_style = style::HeadingStyle::for_level(level);
        self.add_paragraph_with_style(text, &heading_style.text_style)
    }

    /// Add a simple table from 2D string array
    pub fn add_simple_table(&mut self, data: &[Vec<&str>]) -> Result<()> {
        if data.is_empty() {
            return Err(HwpError::InvalidInput(
                "Table data cannot be empty".to_string(),
            ));
        }

        let rows = data.len() as u32;
        let cols = data[0].len() as u32;

        // Create table builder and populate with data
        let mut table_builder = style::TableBuilder::new(self, rows, cols);

        for (row_idx, row) in data.iter().enumerate() {
            for (col_idx, cell_text) in row.iter().enumerate() {
                table_builder = table_builder.set_cell(row_idx as u32, col_idx as u32, cell_text);
            }
        }

        table_builder.finish()
    }

    /// Create a table builder for advanced table creation
    pub fn add_table(&mut self, rows: u32, cols: u32) -> style::TableBuilder<'_> {
        style::TableBuilder::new(self, rows, cols)
    }

    /// Add a simple list with specified type
    pub fn add_list(&mut self, items: &[&str], list_type: style::ListType) -> Result<()> {
        self.start_list(list_type)?;
        for item in items {
            self.add_list_item(item)?;
        }
        self.end_list()
    }

    /// Start a list with specified type
    pub fn start_list(&mut self, list_type: style::ListType) -> Result<()> {
        // For now, we'll implement lists as styled paragraphs with appropriate prefixes
        // In a full implementation, this would create proper list structures
        self.current_list_type = Some(list_type);
        self.current_list_level = 0;
        self.current_list_index = 0;
        Ok(())
    }

    /// Add an item to the current list
    pub fn add_list_item(&mut self, text: &str) -> Result<()> {
        if let Some(list_type) = &self.current_list_type {
            self.current_list_index += 1;
            let prefix =
                self.get_list_prefix(list_type, self.current_list_index, self.current_list_level);
            let full_text = format!("{} {}", prefix, text);

            // Create a list style with appropriate indentation
            let list_style = style::TextStyle::new(); // Could add indentation styling here
            self.add_paragraph_with_style(&full_text, &list_style)?;
        } else {
            return Err(HwpError::InvalidInput(
                "No active list. Call start_list() first.".to_string(),
            ));
        }
        Ok(())
    }

    /// Start a nested list
    pub fn start_nested_list(&mut self, list_type: style::ListType) -> Result<()> {
        self.current_list_level += 1;
        self.list_stack.push((
            self.current_list_type
                .clone()
                .unwrap_or(style::ListType::Bullet),
            self.current_list_index,
        ));
        self.current_list_type = Some(list_type);
        self.current_list_index = 0;
        Ok(())
    }

    /// End the current list
    pub fn end_list(&mut self) -> Result<()> {
        if self.current_list_level > 0 {
            // Return to parent list
            if let Some((parent_type, parent_index)) = self.list_stack.pop() {
                self.current_list_type = Some(parent_type);
                self.current_list_index = parent_index;
                self.current_list_level -= 1;
            }
        } else {
            // End the top-level list
            self.current_list_type = None;
            self.current_list_index = 0;
        }
        Ok(())
    }

    /// Get the appropriate prefix for a list item
    fn get_list_prefix(&self, list_type: &style::ListType, index: u32, level: u32) -> String {
        let indent = "  ".repeat(level as usize);
        match list_type {
            style::ListType::Bullet => {
                let symbol = match level {
                    0 => "•",
                    1 => "◦",
                    _ => "▪",
                };
                format!("{}{}", indent, symbol)
            }
            style::ListType::Numbered => format!("{}{}.", indent, index),
            style::ListType::Alphabetic => {
                let letter = ((index - 1) % 26) as u8 + b'a';
                format!("{}{}).", indent, letter as char)
            }
            style::ListType::Roman => {
                let roman = self.to_roman(index);
                format!("{}{}.", indent, roman)
            }
            style::ListType::Korean => {
                let korean_nums = ["가", "나", "다", "라", "마", "바", "사", "아", "자", "차"];
                let korean = korean_nums
                    .get((index - 1) as usize % korean_nums.len())
                    .unwrap_or(&"가");
                format!("{}{}).", indent, korean)
            }
            style::ListType::Custom(format) => format!("{}{}", indent, format),
        }
    }

    /// Convert number to Roman numerals
    fn to_roman(&self, mut num: u32) -> String {
        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let symbols = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];

        let mut result = String::new();
        for (i, &value) in values.iter().enumerate() {
            while num >= value {
                result.push_str(symbols[i]);
                num -= value;
            }
        }
        result.to_lowercase()
    }

    /// Add an image from file path
    pub fn add_image<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let image_data = std::fs::read(path)?;
        let format = style::ImageFormat::from_bytes(&image_data).unwrap_or(style::ImageFormat::Png);
        let options = style::ImageOptions::new();
        self.add_image_with_options(&image_data, format, &options)
    }

    /// Add an image from byte data
    pub fn add_image_from_bytes(&mut self, data: &[u8], format: style::ImageFormat) -> Result<()> {
        let options = style::ImageOptions::new();
        self.add_image_with_options(data, format, &options)
    }

    /// Add an image with custom options
    pub fn add_image_with_options(
        &mut self,
        data: &[u8],
        format: style::ImageFormat,
        options: &style::ImageOptions,
    ) -> Result<()> {
        use crate::model::bin_data::BinData;
        use crate::model::control::Picture;
        use crate::model::ctrl_header::{ControlType, CtrlHeader};

        // Create binary data entry
        let bin_data = BinData {
            properties: 0,
            abs_name: String::new(),
            rel_name: format!("image_{}.{}", self.next_instance_id(), format.extension()),
            bin_id: self.document.doc_info.bin_data.len() as u16,
            extension: format.extension().to_string(),
            data: data.to_vec(),
        };

        // Add to document's binary data collection
        self.document.doc_info.bin_data.push(bin_data.clone());

        // Calculate dimensions (convert mm to HWPUNIT)
        let hwp_scale = 7200.0 / 25.4;
        let width = options.width.unwrap_or(50) as f32 * hwp_scale; // Default 50mm
        let height = options.height.unwrap_or(50) as f32 * hwp_scale; // Default 50mm

        // Create picture control
        let picture = Picture {
            properties: 0,
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
            z_order: 0,
            outer_margin_left: 0,
            outer_margin_right: 0,
            outer_margin_top: 0,
            outer_margin_bottom: 0,
            instance_id: self.next_instance_id(),
            bin_item_id: bin_data.bin_id,
            border_fill_id: 0,
            image_width: width as u32,
            image_height: height as u32,
        };

        // Create control header
        let ctrl_header = CtrlHeader {
            ctrl_id: ControlType::Gso as u32, // Gso is for graphics/drawing objects including images
            properties: 0,
            instance_id: self.next_instance_id(),
        };

        // Create paragraph containing the image
        let paragraph = Paragraph {
            text: options.caption.as_ref().map(|caption| ParaText {
                content: caption.clone(),
            }),
            control_mask: 1, // Indicates control is present
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: self.next_instance_id(),
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: Some(picture),
            text_box_data: None,
            hyperlinks: Vec::new(),
        };

        // Add the paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a hyperlink to URL
    pub fn add_hyperlink(&mut self, display_text: &str, url: &str) -> Result<()> {
        use crate::model::hyperlink::{Hyperlink, HyperlinkDisplay, HyperlinkType};

        let hyperlink = Hyperlink {
            hyperlink_type: HyperlinkType::Url,
            display_text: display_text.to_string(),
            target_url: url.to_string(),
            tooltip: None,
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x0000FF,    // Blue
            visited_color: 0x800080, // Purple
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: display_text.len() as u32,
        };

        self.add_hyperlink_with_options(hyperlink)
    }

    /// Add an email hyperlink
    pub fn add_email_link(&mut self, display_text: &str, email: &str) -> Result<()> {
        use crate::model::hyperlink::{Hyperlink, HyperlinkDisplay, HyperlinkType};

        let hyperlink = Hyperlink {
            hyperlink_type: HyperlinkType::Email,
            display_text: display_text.to_string(),
            target_url: format!("mailto:{}", email),
            tooltip: Some(format!("이메일 보내기: {}", email)),
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x0000FF,
            visited_color: 0x800080,
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: display_text.len() as u32,
        };

        self.add_hyperlink_with_options(hyperlink)
    }

    /// Add a file hyperlink
    pub fn add_file_link(&mut self, display_text: &str, file_path: &str) -> Result<()> {
        use crate::model::hyperlink::{Hyperlink, HyperlinkDisplay, HyperlinkType};

        let hyperlink = Hyperlink {
            hyperlink_type: HyperlinkType::File,
            display_text: display_text.to_string(),
            target_url: file_path.to_string(),
            tooltip: Some(format!("파일 열기: {}", file_path)),
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x008000, // Green for file links
            visited_color: 0x800080,
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: display_text.len() as u32,
        };

        self.add_hyperlink_with_options(hyperlink)
    }

    /// Add a hyperlink with custom options
    pub fn add_hyperlink_with_options(
        &mut self,
        hyperlink: crate::model::hyperlink::Hyperlink,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};

        // Create a styled paragraph for the hyperlink
        let hyperlink_style = style::TextStyle::new()
            .color(hyperlink.text_color)
            .underline();

        let para_text = ParaText {
            content: hyperlink.display_text.clone(),
        };

        // Get or create font for the hyperlink style
        let char_shape = hyperlink_style.to_char_shape(0); // Use default font
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create character shape information
        let char_shapes = ParaCharShape {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        };

        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: self.next_instance_id(),
            char_shapes: Some(char_shapes),
            line_segments: None,
            list_header: None,
            ctrl_header: None,
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks: vec![hyperlink],
        };

        // Add the paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a bookmark hyperlink
    pub fn add_bookmark_link(&mut self, display_text: &str, bookmark_name: &str) -> Result<()> {
        use crate::model::hyperlink::{Hyperlink, HyperlinkDisplay, HyperlinkType};

        let hyperlink = Hyperlink {
            hyperlink_type: HyperlinkType::Bookmark,
            display_text: display_text.to_string(),
            target_url: format!("#{}", bookmark_name),
            tooltip: Some(format!("이동: {}", bookmark_name)),
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x800080, // Purple for internal links
            visited_color: 0x800080,
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: display_text.len() as u32,
        };

        self.add_hyperlink_with_options(hyperlink)
    }

    /// Add a custom hyperlink with specific options
    #[allow(clippy::too_many_arguments)]
    pub fn add_custom_hyperlink(
        &mut self,
        display_text: &str,
        hyperlink_type: crate::model::hyperlink::HyperlinkType,
        target_url: &str,
        display_mode: crate::model::hyperlink::HyperlinkDisplay,
        text_color: u32,
        underline: bool,
        new_window: bool,
    ) -> Result<()> {
        use crate::model::hyperlink::Hyperlink;

        let hyperlink = Hyperlink {
            hyperlink_type,
            display_text: display_text.to_string(),
            target_url: target_url.to_string(),
            tooltip: None,
            display_mode,
            text_color,
            visited_color: 0x800080,
            underline,
            visited: false,
            open_in_new_window: new_window,
            start_position: 0,
            length: display_text.len() as u32,
        };

        self.add_hyperlink_with_options(hyperlink)
    }

    /// Add a paragraph with multiple hyperlinks
    pub fn add_paragraph_with_hyperlinks(
        &mut self,
        text: &str,
        hyperlinks: Vec<crate::model::hyperlink::Hyperlink>,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};

        let para_text = ParaText {
            content: text.to_string(),
        };

        // Create character shape
        let char_shape = style::TextStyle::new().to_char_shape(0);
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create character shape information
        let char_shapes = ParaCharShape {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        };

        // Create paragraph
        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 1,
            instance_id: 0,
            char_shapes: Some(char_shapes),
            line_segments: None,
            list_header: None,
            ctrl_header: None,
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks,
        };

        // Add the paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a header to the current section
    pub fn add_header(&mut self, text: &str) {
        use crate::model::header_footer::HeaderFooter;

        // Create header footer collection if not exists
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let header = HeaderFooter::new_header(text);
                    page_def.header_footer.add_header(header);
                }
            }
        }
    }

    /// Add a footer with page number
    pub fn add_footer_with_page_number(
        &mut self,
        prefix: &str,
        format: crate::model::header_footer::PageNumberFormat,
    ) {
        use crate::model::header_footer::HeaderFooter;

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let mut footer = HeaderFooter::new_footer(prefix);
                    footer = footer.with_page_number(format);
                    page_def.header_footer.add_footer(footer);
                }
            }
        }
    }

    /// Set page layout for the document
    pub fn set_page_layout(&mut self, layout: crate::model::page_layout::PageLayout) -> Result<()> {
        use crate::model::page_def::PageDef;

        // Create page definition from layout
        let page_def = PageDef::from_layout(layout);

        // Apply to current section
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.page_def = Some(page_def);
            }
        }

        Ok(())
    }

    /// Set A4 portrait layout with default margins
    pub fn set_a4_portrait(&mut self) -> Result<()> {
        let layout = crate::model::page_layout::PageLayout::a4_portrait();
        self.set_page_layout(layout)
    }

    /// Add a paragraph with specific alignment
    pub fn add_aligned_paragraph(
        &mut self,
        text: &str,
        alignment: style::ParagraphAlignment,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::para_shape::ParaShape;

        // Create para shape with alignment
        let mut para_shape = ParaShape::new_default();
        // Set alignment in properties1 (bits 2-4)
        para_shape.properties1 = (para_shape.properties1 & !0x1C) | ((alignment as u32) << 2);
        let para_shape_id = self.add_para_shape(para_shape)?;

        let para_text = ParaText {
            content: text.to_string(),
        };

        // Create character shape
        let char_shape = style::TextStyle::new().to_char_shape(0);
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create character shape information
        let char_shapes = ParaCharShape {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        };

        // Create paragraph with alignment
        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id,
            style_id: 0,
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 1,
            instance_id: 0,
            char_shapes: Some(char_shapes),
            line_segments: None,
            list_header: None,
            ctrl_header: None,
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks: Vec::new(),
        };

        // Add the paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a paragraph with custom spacing
    pub fn add_paragraph_with_spacing(
        &mut self,
        text: &str,
        line_spacing_percent: u32,
        before_spacing_mm: f32,
        after_spacing_mm: f32,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::para_shape::ParaShape;

        // Create para shape with spacing
        let mut para_shape = ParaShape::new_default();
        para_shape.line_space = (line_spacing_percent * 100) as i32; // Convert percent to internal units
        para_shape.top_para_space = (before_spacing_mm * 283.465) as i32; // Convert mm to HWP units
        para_shape.bottom_para_space = (after_spacing_mm * 283.465) as i32;
        let para_shape_id = self.add_para_shape(para_shape)?;

        let para_text = ParaText {
            content: text.to_string(),
        };

        // Create character shape
        let char_shape = style::TextStyle::new().to_char_shape(0);
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create character shape information
        let char_shapes = ParaCharShape {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        };

        // Create paragraph with spacing
        let paragraph = Paragraph {
            text: Some(para_text),
            control_mask: 0,
            para_shape_id,
            style_id: 0,
            column_type: 0,
            char_shape_count: 1,
            range_tag_count: 0,
            line_align_count: 1,
            instance_id: 0,
            char_shapes: Some(char_shapes),
            line_segments: None,
            list_header: None,
            ctrl_header: None,
            table_data: None,
            picture_data: None,
            text_box_data: None,
            hyperlinks: Vec::new(),
        };

        // Add the paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Set A4 landscape layout with default margins
    pub fn set_a4_landscape(&mut self) -> Result<()> {
        let layout = crate::model::page_layout::PageLayout::a4_landscape();
        self.set_page_layout(layout)
    }

    /// Set Letter portrait layout with default margins
    pub fn set_letter_portrait(&mut self) -> Result<()> {
        let layout = crate::model::page_layout::PageLayout::letter_portrait();
        self.set_page_layout(layout)
    }

    /// Set Letter landscape layout with default margins
    pub fn set_letter_landscape(&mut self) -> Result<()> {
        let layout = crate::model::page_layout::PageLayout::letter_landscape();
        self.set_page_layout(layout)
    }

    /// Set custom page size in millimeters
    pub fn set_custom_page_size(
        &mut self,
        width_mm: f32,
        height_mm: f32,
        orientation: crate::model::page_layout::PageOrientation,
    ) -> Result<()> {
        let layout =
            crate::model::page_layout::PageLayout::custom_mm(width_mm, height_mm, orientation);
        self.set_page_layout(layout)
    }

    /// Set page margins in millimeters
    pub fn set_page_margins_mm(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
    ) -> Result<()> {
        let margins = crate::model::page_layout::PageMargins::new_mm(left, right, top, bottom);
        let layout = crate::model::page_layout::PageLayout {
            margins,
            ..Default::default()
        };
        self.set_page_layout(layout)
    }

    /// Set narrow margins (Office style)
    pub fn set_narrow_margins(&mut self) -> Result<()> {
        let margins = crate::model::page_layout::PageMargins::narrow();
        let layout = crate::model::page_layout::PageLayout {
            margins,
            ..Default::default()
        };
        self.set_page_layout(layout)
    }

    /// Set normal margins (Office style)
    pub fn set_normal_margins(&mut self) -> Result<()> {
        let margins = crate::model::page_layout::PageMargins::normal();
        let layout = crate::model::page_layout::PageLayout {
            margins,
            ..Default::default()
        };
        self.set_page_layout(layout)
    }

    /// Set wide margins (Office style)
    pub fn set_wide_margins(&mut self) -> Result<()> {
        let margins = crate::model::page_layout::PageMargins::wide();
        let layout = crate::model::page_layout::PageLayout {
            margins,
            ..Default::default()
        };
        self.set_page_layout(layout)
    }

    /// Set multiple columns
    pub fn set_columns(&mut self, columns: u16, spacing_mm: f32) -> Result<()> {
        let mut layout = crate::model::page_layout::PageLayout::default();
        layout = layout.with_columns(columns, spacing_mm);
        self.set_page_layout(layout)
    }

    /// Set page background color
    pub fn set_page_background_color(&mut self, color: u32) -> Result<()> {
        let mut layout = crate::model::page_layout::PageLayout::default();
        layout = layout.with_background_color(color);
        self.set_page_layout(layout)
    }

    /// Set page numbering
    pub fn set_page_numbering(
        &mut self,
        start: u16,
        format: crate::model::header_footer::PageNumberFormat,
    ) -> Result<()> {
        let mut layout = crate::model::page_layout::PageLayout::default();
        layout = layout.with_page_numbering(start, format);
        self.set_page_layout(layout)
    }

    /// Convert the document to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serializer::serialize_document(&self.document)
    }

    /// Save to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes).map_err(HwpError::Io)?;
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
            face_names: vec![FaceName::new_default("맑은 고딕".to_string())],
            char_shapes: vec![
                CharShape::new_default(), // Default 12pt font
            ],
            para_shapes: vec![
                ParaShape::new_default(), // Default left-aligned paragraph
            ],
            styles: vec![Style::new_default()],
            border_fills: vec![BorderFill::new_default()],
            tab_defs: vec![TabDef::new_default()],
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

    /// Generate and return next unique instance ID
    pub fn next_instance_id(&mut self) -> u32 {
        let id = self.next_instance_id;
        self.next_instance_id += 1;
        id
    }

    /// Ensure a font exists in the document and return its ID
    pub fn ensure_font(&mut self, font_name: &str) -> Result<u16> {
        // Check if font already exists
        for (i, face_name) in self.document.doc_info.face_names.iter().enumerate() {
            if face_name.font_name == font_name {
                return Ok(i as u16);
            }
        }

        // Add new font
        let face_name = FaceName::new_default(font_name.to_string());
        self.document.doc_info.face_names.push(face_name);
        Ok((self.document.doc_info.face_names.len() - 1) as u16)
    }

    /// Add a character shape to the document and return its ID
    pub fn add_char_shape(&mut self, char_shape: CharShape) -> Result<u16> {
        self.document.doc_info.char_shapes.push(char_shape);
        Ok((self.document.doc_info.char_shapes.len() - 1) as u16)
    }

    /// Add a paragraph shape to the document and return its ID
    pub fn add_para_shape(
        &mut self,
        para_shape: crate::model::para_shape::ParaShape,
    ) -> Result<u16> {
        self.document.doc_info.para_shapes.push(para_shape);
        Ok((self.document.doc_info.para_shapes.len() - 1) as u16)
    }
}

impl HwpWriter {
    /// Create a writer from an existing HwpDocument
    pub fn from_document(document: HwpDocument) -> Self {
        Self {
            document,
            current_section_idx: 0,
            next_instance_id: 1,
            current_list_type: None,
            current_list_level: 0,
            current_list_index: 0,
            list_stack: Vec::new(),
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
