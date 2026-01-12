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
    /// Current page layout
    page_layout: crate::model::page_layout::PageLayout,
}

/// Options for custom hyperlink styling
pub struct HyperlinkStyleOptions {
    pub text_color: u32,
    pub underline: bool,
    pub new_window: bool,
}

/// Options for custom text box styling
pub struct CustomTextBoxStyle {
    pub alignment: crate::model::text_box::TextBoxAlignment,
    pub border_style: crate::model::text_box::TextBoxBorderStyle,
    pub border_color: u32,
    pub background_color: u32,
}

/// Options for floating text box styling
pub struct FloatingTextBoxStyle {
    pub opacity: u8,
    pub rotation: i16,
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
                preview_text: None,
                preview_image: None,
                summary_info: None,
            },
            current_section_idx: 0,
            next_instance_id: 1,
            current_list_type: None,
            current_list_level: 0,
            current_list_index: 0,
            list_stack: Vec::new(),
            page_layout: crate::model::page_layout::PageLayout::default(),
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
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::para_shape::ParaShape;

        let heading_style = style::HeadingStyle::for_level(level);

        // Get or create font for the heading style
        let face_name_id = if let Some(font_name) = &heading_style.text_style.font_name {
            self.ensure_font(font_name)?
        } else {
            0 // Use default font
        };

        // Create character shape from heading text style
        let char_shape = heading_style.text_style.to_char_shape(face_name_id);
        let char_shape_id = self.add_char_shape(char_shape)?;

        // Create paragraph shape with heading-specific spacing
        let mut para_shape = ParaShape::new_default();
        para_shape.top_para_space = heading_style.spacing_before;
        para_shape.bottom_para_space = heading_style.spacing_after;
        let para_shape_id = self.add_para_shape(para_shape)?;

        // Create paragraph text
        let para_text = ParaText {
            content: text.to_string(),
        };

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
            para_shape_id,
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

        // Add paragraph to current section
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add a simple table from 2D string array
    pub fn add_simple_table(&mut self, data: &[Vec<&str>]) -> Result<()> {
        if data.is_empty() {
            // Empty table - do nothing
            return Ok(());
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

            // Create paragraph shape with appropriate indentation for this level
            let indent_per_level = 1000; // ~3.5mm per level in HWP units
            let left_margin = 567 + (indent_per_level * self.current_list_level as i32);

            let mut para_shape = crate::model::para_shape::ParaShape::new_default();
            para_shape.left_margin = left_margin;
            para_shape.indent = 0; // No first line indent for list items
            let para_shape_id = self.add_para_shape(para_shape)?;

            // Create the paragraph with proper para_shape_id
            use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};

            let para_text = ParaText { content: full_text };

            let char_shapes = ParaCharShape {
                char_positions: vec![CharPositionShape {
                    position: 0,
                    char_shape_id: 0, // Use default character shape
                }],
            };

            let paragraph = Paragraph {
                text: Some(para_text),
                control_mask: 0,
                para_shape_id,
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

            // Add paragraph to current section
            if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
                if let Some(section) = body_text.sections.get_mut(0) {
                    section.paragraphs.push(paragraph);
                }
            }
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
        // No text-based indent - we use paragraph shapes for indentation
        match list_type {
            style::ListType::Bullet => {
                let symbol = match level {
                    0 => "•",
                    1 => "◦",
                    _ => "▪",
                };
                symbol.to_string()
            }
            style::ListType::Numbered => format!("{}.", index),
            style::ListType::Alphabetic => {
                let letter = ((index - 1) % 26) as u8 + b'a';
                format!("{})", letter as char)
            }
            style::ListType::Roman => {
                let roman = self.to_roman(index);
                format!("{}.", roman)
            }
            style::ListType::Korean => {
                let korean_nums = ["가", "나", "다", "라", "마", "바", "사", "아", "자", "차"];
                let korean = korean_nums
                    .get((index - 1) as usize % korean_nums.len())
                    .unwrap_or(&"가");
                format!("{}.", korean)
            }
            style::ListType::Custom(format) => format.clone(),
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

        // Calculate bin_id (1-based index)
        let bin_id = (self.document.doc_info.bin_data.len() + 1) as u16;

        // Create binary data entry
        let bin_data = BinData {
            properties: 0,
            abs_name: format!("image{}.{}", bin_id, format.extension()),
            rel_name: format!("image_{}.{}", self.next_instance_id(), format.extension()),
            bin_id,
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

        // Create paragraph containing the image (no text - picture control paragraph)
        let paragraph = Paragraph {
            text: None,      // Picture control paragraphs don't contain text
            control_mask: 2, // Control header present (0x02)
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

        // Add the picture control paragraph to the document
        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        // If caption is provided, add it as a separate paragraph
        if let Some(caption) = &options.caption {
            let caption_text = format!("그림: {}", caption);
            self.add_paragraph(&caption_text)?;
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
    pub fn add_custom_hyperlink(
        &mut self,
        display_text: &str,
        hyperlink_type: crate::model::hyperlink::HyperlinkType,
        target_url: &str,
        display_mode: crate::model::hyperlink::HyperlinkDisplay,
        style_options: HyperlinkStyleOptions,
    ) -> Result<()> {
        use crate::model::hyperlink::Hyperlink;

        let hyperlink = Hyperlink {
            hyperlink_type,
            display_text: display_text.to_string(),
            target_url: target_url.to_string(),
            tooltip: None,
            display_mode,
            text_color: style_options.text_color,
            visited_color: 0x800080,
            underline: style_options.underline,
            visited: false,
            open_in_new_window: style_options.new_window,
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
    pub fn set_page_margins_mm(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
        use crate::model::page_layout::mm_to_hwp_units;
        self.page_layout.margins.left = mm_to_hwp_units(left);
        self.page_layout.margins.right = mm_to_hwp_units(right);
        self.page_layout.margins.top = mm_to_hwp_units(top);
        self.page_layout.margins.bottom = mm_to_hwp_units(bottom);
    }

    /// Set narrow margins (Office style)
    pub fn set_narrow_margins(&mut self) {
        self.page_layout.margins = crate::model::page_layout::PageMargins::narrow();
    }

    /// Set normal margins (Office style)
    pub fn set_normal_margins(&mut self) {
        self.page_layout.margins = crate::model::page_layout::PageMargins::normal();
    }

    /// Set wide margins (Office style)
    pub fn set_wide_margins(&mut self) {
        self.page_layout.margins = crate::model::page_layout::PageMargins::wide();
    }

    /// Set multiple columns
    pub fn set_columns(&mut self, columns: u16, spacing_mm: f32) {
        use crate::model::page_layout::mm_to_hwp_units;
        self.page_layout.columns = columns;
        self.page_layout.column_spacing = mm_to_hwp_units(spacing_mm);
    }

    /// Set page background color
    pub fn set_page_background_color(&mut self, color: u32) {
        self.page_layout.background_color = Some(color);
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
            page_layout: crate::model::page_layout::PageLayout::default(),
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

// Page Layout Methods
impl HwpWriter {
    /// Get current page layout
    pub fn get_page_layout(&self) -> crate::model::page_layout::PageLayout {
        self.page_layout.clone()
    }

    /// Set paper size
    pub fn set_paper_size(&mut self, paper_size: crate::model::page_layout::PaperSize) {
        let (width, height) = paper_size.dimensions_hwp_units();
        let (final_width, final_height) = match self.page_layout.orientation {
            crate::model::page_layout::PageOrientation::Portrait => (width, height),
            crate::model::page_layout::PageOrientation::Landscape => (height, width),
        };
        self.page_layout.paper_size = paper_size;
        self.page_layout.width = final_width;
        self.page_layout.height = final_height;
    }

    /// Set page orientation
    pub fn set_page_orientation(
        &mut self,
        orientation: crate::model::page_layout::PageOrientation,
    ) {
        if self.page_layout.orientation != orientation {
            // Swap width and height when changing orientation
            std::mem::swap(&mut self.page_layout.width, &mut self.page_layout.height);
            self.page_layout.orientation = orientation;
        }
    }

    /// Set page margins in inches
    pub fn set_page_margins_inches(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
        use crate::model::page_layout::inches_to_hwp_units;
        self.page_layout.margins.left = inches_to_hwp_units(left);
        self.page_layout.margins.right = inches_to_hwp_units(right);
        self.page_layout.margins.top = inches_to_hwp_units(top);
        self.page_layout.margins.bottom = inches_to_hwp_units(bottom);
    }

    /// Set custom page size in millimeters
    pub fn set_custom_page_size_mm(&mut self, width_mm: f32, height_mm: f32) {
        use crate::model::page_layout::mm_to_hwp_units;
        self.page_layout.paper_size = crate::model::page_layout::PaperSize::Custom;
        let width = mm_to_hwp_units(width_mm);
        let height = mm_to_hwp_units(height_mm);
        let (final_width, final_height) = match self.page_layout.orientation {
            crate::model::page_layout::PageOrientation::Portrait => (width, height),
            crate::model::page_layout::PageOrientation::Landscape => (height, width),
        };
        self.page_layout.width = final_width;
        self.page_layout.height = final_height;
    }
}

// Styled Text Methods
impl HwpWriter {
    /// Helper to convert TextStyle to CharShape
    fn text_style_to_char_shape(&self, style: &style::TextStyle) -> CharShape {
        let mut char_shape = CharShape::new_default();

        // Set properties
        let mut properties = 0u32;
        if style.bold {
            properties |= 0x1; // Bit 0 for bold
        }
        if style.italic {
            properties |= 0x2; // Bit 1 for italic
        }
        if style.underline {
            properties |= 0x1 << 2; // Bits 2-4 for underline (type 1)
        }
        if style.strikethrough {
            properties |= 0x1 << 5; // Bits 5-7 for strikethrough (type 1)
        }
        char_shape.properties = properties;

        // Set colors
        char_shape.text_color = style.color;
        if let Some(bg_color) = style.background_color {
            char_shape.shade_color = bg_color;
        }

        // Set font size if specified
        if let Some(size) = style.font_size {
            char_shape.base_size = (size * 100) as i32; // Convert pt to HWP units
        }

        char_shape
    }

    /// Add a styled paragraph
    pub fn add_styled_paragraph(&mut self, styled_text: &style::StyledText) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::paragraph::{ParaText, Paragraph};

        let text = &styled_text.text;
        let ranges = &styled_text.ranges;

        // Create CharShape for each unique style and build position-shape pairs
        let mut char_positions = Vec::new();
        let default_char_shape_id = 0u16; // Use default shape for unstyled text

        for range in ranges {
            let char_shape = self.text_style_to_char_shape(&range.style);
            let char_shape_id = self.add_char_shape(char_shape)?;

            char_positions.push(CharPositionShape {
                position: range.start as u32,
                char_shape_id,
            });
        }

        // Sort by position
        char_positions.sort_by_key(|p| p.position);

        let para_char_shape = if char_positions.is_empty() {
            ParaCharShape::new_single_shape(default_char_shape_id)
        } else {
            ParaCharShape { char_positions }
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: text.clone(),
            }),
            char_shapes: Some(para_char_shape.clone()),
            char_shape_count: para_char_shape.char_positions.len() as u16,
            ..Default::default()
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add paragraph with bold ranges
    pub fn add_paragraph_with_bold(
        &mut self,
        text: &str,
        bold_ranges: Vec<(usize, usize)>,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::paragraph::{ParaText, Paragraph};

        let mut char_positions = Vec::new();

        for (start, _end) in bold_ranges {
            let mut char_shape = CharShape::new_default();
            char_shape.properties = 0x1; // Bold
            let char_shape_id = self.add_char_shape(char_shape)?;

            char_positions.push(CharPositionShape {
                position: start as u32,
                char_shape_id,
            });
        }

        char_positions.sort_by_key(|p| p.position);

        let para_char_shape = ParaCharShape { char_positions };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: text.to_string(),
            }),
            char_shapes: Some(para_char_shape.clone()),
            char_shape_count: para_char_shape.char_positions.len() as u16,
            ..Default::default()
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add paragraph with colors
    pub fn add_paragraph_with_colors(
        &mut self,
        text: &str,
        color_ranges: Vec<(usize, usize, u32)>,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::paragraph::{ParaText, Paragraph};

        let mut char_positions = Vec::new();

        for (start, _end, color) in color_ranges {
            let mut char_shape = CharShape::new_default();
            char_shape.text_color = color;
            let char_shape_id = self.add_char_shape(char_shape)?;

            char_positions.push(CharPositionShape {
                position: start as u32,
                char_shape_id,
            });
        }

        char_positions.sort_by_key(|p| p.position);

        let para_char_shape = ParaCharShape { char_positions };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: text.to_string(),
            }),
            char_shapes: Some(para_char_shape.clone()),
            char_shape_count: para_char_shape.char_positions.len() as u16,
            ..Default::default()
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add paragraph with highlight
    pub fn add_paragraph_with_highlight(
        &mut self,
        text: &str,
        highlight_ranges: Vec<(usize, usize, u32)>,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::paragraph::{ParaText, Paragraph};

        let mut char_positions = Vec::new();

        for (start, _end, color) in highlight_ranges {
            let mut char_shape = CharShape::new_default();
            char_shape.shade_color = color; // Use shade_color for background/highlight
            let char_shape_id = self.add_char_shape(char_shape)?;

            char_positions.push(CharPositionShape {
                position: start as u32,
                char_shape_id,
            });
        }

        char_positions.sort_by_key(|p| p.position);

        let para_char_shape = ParaCharShape { char_positions };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: text.to_string(),
            }),
            char_shapes: Some(para_char_shape.clone()),
            char_shape_count: para_char_shape.char_positions.len() as u16,
            ..Default::default()
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add mixed text with multiple styles
    pub fn add_mixed_text(
        &mut self,
        text: &str,
        style_ranges: Vec<(usize, usize, style::TextStyle)>,
    ) -> Result<()> {
        use crate::model::para_char_shape::{CharPositionShape, ParaCharShape};
        use crate::model::paragraph::{ParaText, Paragraph};

        let mut char_positions = Vec::new();

        for (start, _end, text_style) in style_ranges {
            let char_shape = self.text_style_to_char_shape(&text_style);
            let char_shape_id = self.add_char_shape(char_shape)?;

            char_positions.push(CharPositionShape {
                position: start as u32,
                char_shape_id,
            });
        }

        char_positions.sort_by_key(|p| p.position);

        let para_char_shape = ParaCharShape { char_positions };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: text.to_string(),
            }),
            char_shapes: Some(para_char_shape.clone()),
            char_shape_count: para_char_shape.char_positions.len() as u16,
            ..Default::default()
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }
}

// Text Box Methods
impl HwpWriter {
    /// Add a simple text box
    pub fn add_text_box(&mut self, text: &str) -> Result<()> {
        use crate::model::ctrl_header::CtrlHeader;
        use crate::model::paragraph::ParaText;
        use crate::model::text_box::TextBox;

        let text_box = TextBox::new(text);

        let ctrl_header = CtrlHeader {
            ctrl_id: 0x7874, // TextBox control ID
            properties: 0,
            instance_id: 0,
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: String::new(),
            }),
            control_mask: 0x02, // Control header present
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: None,
            text_box_data: Some(text_box),
            hyperlinks: Vec::new(),
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add text box at specific position
    pub fn add_text_box_at_position(
        &mut self,
        text: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        use crate::model::ctrl_header::CtrlHeader;
        use crate::model::paragraph::ParaText;
        use crate::model::text_box::TextBox;

        let text_box = TextBox::new(text)
            .with_position_mm(x as i32, y as i32)
            .with_size_mm(width, height);

        let ctrl_header = CtrlHeader {
            ctrl_id: 0x7874, // TextBox control ID
            properties: 0,
            instance_id: 0,
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: String::new(),
            }),
            control_mask: 0x02,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: None,
            text_box_data: Some(text_box),
            hyperlinks: Vec::new(),
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add custom text box
    pub fn add_custom_text_box(
        &mut self,
        text: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        style: CustomTextBoxStyle,
    ) -> Result<()> {
        use crate::model::ctrl_header::CtrlHeader;
        use crate::model::paragraph::ParaText;
        use crate::model::text_box::TextBox;

        let text_box = TextBox::new(text)
            .with_position_mm(x as i32, y as i32)
            .with_size_mm(width, height)
            .with_alignment(style.alignment)
            .with_border(style.border_style, 1, style.border_color)
            .with_background(style.background_color);

        let ctrl_header = CtrlHeader {
            ctrl_id: 0x7874, // TextBox control ID
            properties: 0,
            instance_id: 0,
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: String::new(),
            }),
            control_mask: 0x02,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: None,
            text_box_data: Some(text_box),
            hyperlinks: Vec::new(),
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add styled text box
    pub fn add_styled_text_box(&mut self, text: &str, style: &str) -> Result<()> {
        use crate::model::ctrl_header::CtrlHeader;
        use crate::model::paragraph::ParaText;
        use crate::model::text_box::TextBox;

        let text_box = match style {
            "basic" => TextBox::basic(text),
            "highlight" => TextBox::highlight(text),
            "warning" => TextBox::warning(text),
            "info" => TextBox::info(text),
            "transparent" => TextBox::transparent(text),
            "bubble" => TextBox::bubble(text),
            _ => TextBox::basic(text),
        };

        let ctrl_header = CtrlHeader {
            ctrl_id: 0x7874, // TextBox control ID
            properties: 0,
            instance_id: 0,
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: String::new(),
            }),
            control_mask: 0x02,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: None,
            text_box_data: Some(text_box),
            hyperlinks: Vec::new(),
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }

    /// Add floating text box
    pub fn add_floating_text_box(
        &mut self,
        text: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        style: FloatingTextBoxStyle,
    ) -> Result<()> {
        use crate::model::ctrl_header::CtrlHeader;
        use crate::model::paragraph::ParaText;
        use crate::model::text_box::{TextBox, TextBoxAlignment};

        let text_box = TextBox::new(text)
            .with_position_mm(x as i32, y as i32)
            .with_size_mm(width, height)
            .with_alignment(TextBoxAlignment::Absolute)
            .with_transparent_background()
            .with_opacity(style.opacity)
            .with_rotation(style.rotation);

        let ctrl_header = CtrlHeader {
            ctrl_id: 0x7874, // TextBox control ID
            properties: 0,
            instance_id: 0,
        };

        let paragraph = Paragraph {
            text: Some(ParaText {
                content: String::new(),
            }),
            control_mask: 0x02,
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: 0,
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: None,
            picture_data: None,
            text_box_data: Some(text_box),
            hyperlinks: Vec::new(),
        };

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(paragraph);
            }
        }

        Ok(())
    }
}

// Header/Footer Methods
impl HwpWriter {
    /// Add header with options
    pub fn add_header_with_options(
        &mut self,
        text: &str,
        page_type: crate::model::header_footer::PageApplyType,
        alignment: crate::model::header_footer::HeaderFooterAlignment,
    ) {
        use crate::model::header_footer::HeaderFooter;

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let header = HeaderFooter::new_header(text)
                        .with_apply_type(page_type)
                        .with_alignment(alignment);
                    page_def.header_footer.add_header(header);
                }
            }
        }
    }

    /// Add header with page number
    pub fn add_header_with_page_number(
        &mut self,
        text: &str,
        format: crate::model::header_footer::PageNumberFormat,
    ) {
        use crate::model::header_footer::{HeaderFooter, HeaderFooterAlignment};

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let header = HeaderFooter::new_header(text)
                        .with_page_number(format)
                        .with_alignment(HeaderFooterAlignment::Center); // Default center alignment for page numbers
                    page_def.header_footer.add_header(header);
                }
            }
        }
    }

    /// Add footer
    pub fn add_footer(&mut self, text: &str) {
        use crate::model::header_footer::HeaderFooter;

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let footer = HeaderFooter::new_footer(text);
                    page_def.header_footer.add_footer(footer);
                }
            }
        }
    }

    /// Add footer with options
    pub fn add_footer_with_options(
        &mut self,
        text: &str,
        page_type: crate::model::header_footer::PageApplyType,
        alignment: crate::model::header_footer::HeaderFooterAlignment,
    ) {
        use crate::model::header_footer::HeaderFooter;

        if let Some(body_text) = self.document.body_texts.get_mut(self.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                if section.page_def.is_none() {
                    section.page_def = Some(crate::model::page_def::PageDef::new_default());
                }
                if let Some(page_def) = section.page_def.as_mut() {
                    let footer = HeaderFooter::new_footer(text)
                        .with_apply_type(page_type)
                        .with_alignment(alignment);
                    page_def.header_footer.add_footer(footer);
                }
            }
        }
    }
}

// Document Metadata and Statistics Methods
impl HwpWriter {
    /// Set document title
    pub fn set_document_title(&mut self, title: &str) -> &mut Self {
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.document_title = Some(title.to_string());
        }
        self
    }

    /// Set document author
    pub fn set_document_author(&mut self, author: &str) -> &mut Self {
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.document_author = Some(author.to_string());
        }
        self
    }

    /// Set document subject
    pub fn set_document_subject(&mut self, subject: &str) -> &mut Self {
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.document_subject = Some(subject.to_string());
        }
        self
    }

    /// Set document keywords
    pub fn set_document_keywords(&mut self, keywords: &str) -> &mut Self {
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.document_keywords = Some(keywords.to_string());
        }
        self
    }

    /// Set document company
    pub fn set_document_company(&mut self, company: &str) -> &mut Self {
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.document_company = Some(company.to_string());
        }
        self
    }

    /// Update document statistics (character count, word count, etc.)
    pub fn update_document_statistics(&mut self) {
        // Extract text first (immutable borrow)
        let text = self.document.extract_text();

        // Count sections
        let section_count = self
            .document
            .body_texts
            .iter()
            .map(|bt| bt.sections.len())
            .sum::<usize>() as u16;

        // Count total characters (excluding spaces)
        let chars: Vec<char> = text.chars().collect();
        let total_character_count = chars.iter().filter(|c| !c.is_whitespace()).count() as u32;
        let space_character_count = chars.iter().filter(|c| c.is_whitespace()).count() as u32;

        // Count different character types
        let hangul_character_count = chars.iter().filter(|c| is_hangul(**c)).count() as u32;
        let english_character_count =
            chars.iter().filter(|c| c.is_ascii_alphabetic()).count() as u32;
        let hanja_character_count = chars.iter().filter(|c| is_hanja(**c)).count() as u32;
        let japanese_character_count = chars.iter().filter(|c| is_japanese(**c)).count() as u32;

        // Count symbols and other
        let symbol_character_count =
            chars.iter().filter(|c| c.is_ascii_punctuation()).count() as u32;
        let other_character_count = (total_character_count as usize
            - hangul_character_count as usize
            - english_character_count as usize
            - hanja_character_count as usize
            - japanese_character_count as usize
            - symbol_character_count as usize) as u32;

        // Count words (split by whitespace)
        let total_word_count = text.split_whitespace().count() as u32;

        // Rough line count estimation (average 10 words per line)
        let line_count = if total_word_count > 0 {
            (total_word_count as f32 / 10.0).ceil() as u32
        } else {
            0
        };

        // Page count estimation (average 500 words per page)
        let total_page_count = if total_word_count > 0 {
            (total_word_count as f32 / 500.0).ceil() as u32
        } else {
            1
        };

        // Now update properties (mutable borrow)
        if let Some(props) = self.document.doc_info.properties.as_mut() {
            props.section_count = section_count;
            props.total_character_count = total_character_count;
            props.space_character_count = space_character_count;
            props.hangul_character_count = hangul_character_count;
            props.english_character_count = english_character_count;
            props.hanja_character_count = hanja_character_count;
            props.japanese_character_count = japanese_character_count;
            props.symbol_character_count = symbol_character_count;
            props.other_character_count = other_character_count;
            props.total_word_count = total_word_count;
            props.line_count = line_count;
            props.total_page_count = total_page_count;
        }
    }

    /// Get document statistics
    pub fn get_document_statistics(&self) -> Option<&crate::model::DocumentProperties> {
        self.document.doc_info.properties.as_ref()
    }

    /// Get mutable document statistics
    pub fn get_document_statistics_mut(&mut self) -> Option<&mut crate::model::DocumentProperties> {
        self.document.doc_info.properties.as_mut()
    }
}

// Helper functions for character type detection
fn is_hangul(c: char) -> bool {
    matches!(c, '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{A960}'..='\u{A97F}')
}

fn is_hanja(c: char) -> bool {
    matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}')
}

fn is_japanese(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}')
}

// ============================================================================
// TODO: Unimplemented Features
// ============================================================================
//
// The following features are part of the HWP format specification but are not
// yet implemented in this library. Example usage can be found in:
// examples/shape_document.rs.disabled
//
// ## Shape Drawing (도형 그리기)
// - [ ] add_rectangle(x, y, width, height) - Draw rectangles
// - [ ] add_circle(x, y, radius) - Draw circles
// - [ ] add_ellipse(x, y, width, height) - Draw ellipses
// - [ ] add_line(x1, y1, x2, y2) - Draw lines
// - [ ] add_dashed_line(x1, y1, x2, y2) - Draw dashed lines
// - [ ] add_arrow(x1, y1, x2, y2) - Draw arrows
// - [ ] add_polygon(points) - Draw custom polygons
// - [ ] add_custom_shape(shape_type, ...) - Draw custom shapes with styling
// - [ ] add_shape_with_text(shape, text, alignment) - Shapes with text content
// - [ ] add_shape_group(shapes) - Group multiple shapes together
//
// ## Other Potential Features
// - [ ] Advanced table features (cell spanning, nested tables, etc.)
// - [ ] Chart/graph insertion
// - [ ] Mathematical equations (MathML)
// - [ ] Video/audio embedding
// - [ ] Forms and input fields
// - [ ] Comments and annotations
// - [ ] Track changes/revision history
// - [ ] Mail merge fields
//
// If you need any of these features, please open an issue at:
// https://github.com/yourusername/hwpers/issues
// ============================================================================
