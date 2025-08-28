use crate::model::char_shape::CharShape;

/// Text style configuration for paragraphs
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_name: Option<String>,
    pub font_size: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: u32,
    pub background_color: Option<u32>,
}

#[allow(clippy::derivable_impls)]
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_name: None,
            font_size: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            color: 0x000000, // Black color by default
            background_color: None,
        }
    }
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set font name
    pub fn font(mut self, font_name: &str) -> Self {
        self.font_name = Some(font_name.to_string());
        self
    }

    /// Set font size in points
    pub fn size(mut self, size_pt: u32) -> Self {
        self.font_size = Some(size_pt);
        self
    }

    /// Set bold
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set italic
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set underline
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Set strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Set text color (RGB format: 0xRRGGBB)
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Set background color (RGB format: 0xRRGGBB)
    pub fn background(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Convert to CharShape for internal use
    pub(crate) fn to_char_shape(&self, face_name_id: u16) -> CharShape {
        let mut properties = 0u32;
        
        if self.bold {
            properties |= 1 << 0; // Bit 0: Bold
        }
        if self.italic {
            properties |= 1 << 1; // Bit 1: Italic
        }
        if self.underline {
            properties |= 1 << 2; // Bit 2: Underline
        }
        if self.strikethrough {
            properties |= 1 << 3; // Bit 3: Strikethrough
        }

        let base_size = self.font_size.unwrap_or(12) as i32 * 100; // Convert pt to hwp units

        CharShape {
            face_name_ids: [face_name_id; 7], // Use the same font for all languages
            ratios: [100; 7],
            char_spaces: [0; 7],
            relative_sizes: [100; 7],
            char_offsets: [0; 7],
            base_size,
            properties,
            shadow_gap_x: 0,
            shadow_gap_y: 0,
            text_color: self.color,
            underline_color: self.color,
            shade_color: self.background_color.unwrap_or(0xFFFFFF),
            shadow_color: 0x808080,
            border_fill_id: 0,
        }
    }
}

/// Heading style configuration
#[derive(Debug, Clone)]
pub struct HeadingStyle {
    pub numbering: bool,
    pub alignment: TextAlign,
    pub spacing_before: i32,
    pub spacing_after: i32,
    pub text_style: TextStyle,
}

impl Default for HeadingStyle {
    fn default() -> Self {
        Self {
            numbering: false,
            alignment: TextAlign::Left,
            spacing_before: 300,  // 3pt before
            spacing_after: 200,   // 2pt after
            text_style: TextStyle::default(),
        }
    }
}

impl HeadingStyle {
    /// Create default heading style for a given level
    pub fn for_level(level: u8) -> Self {
        let (size, spacing_before, spacing_after) = match level {
            1 => (24, 500, 300), // 24pt, 5pt before, 3pt after
            2 => (18, 400, 200), // 18pt, 4pt before, 2pt after
            3 => (14, 300, 150), // 14pt, 3pt before, 1.5pt after
            4 => (12, 200, 100), // 12pt, 2pt before, 1pt after
            _ => (11, 150, 100), // 11pt, 1.5pt before, 1pt after
        };

        Self {
            numbering: false,
            alignment: TextAlign::Left,
            spacing_before,
            spacing_after,
            text_style: TextStyle::new().size(size).bold(),
        }
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
    Distribute,
}

impl TextAlign {
    /// Convert to HWP alignment value
    pub fn to_hwp_value(&self) -> u32 {
        match self {
            TextAlign::Left => 0,
            TextAlign::Right => 1,
            TextAlign::Center => 2,
            TextAlign::Justify => 3,
            TextAlign::Distribute => 4,
        }
    }
}

/// Paragraph alignment types
#[derive(Debug, Clone, Copy)]
pub enum ParagraphAlignment {
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
    Distribute = 4,
}

/// List type options
#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    /// Bulleted list (•, ◦, ▪, etc.)
    Bullet,
    /// Numbered list (1., 2., 3., etc.)
    Numbered,
    /// Alphabetic list (a., b., c., etc.)
    Alphabetic,
    /// Roman numeral list (i., ii., iii., etc.)
    Roman,
    /// Korean numbering (가., 나., 다., etc.)
    Korean,
    /// Custom bullet or numbering format
    Custom(String),
}

impl ListType {
    /// Get the default format string for this list type
    pub fn get_format(&self, level: u8) -> String {
        match self {
            ListType::Bullet => match level {
                0 => "•".to_string(),
                1 => "◦".to_string(),
                2 => "▪".to_string(),
                _ => "·".to_string(),
            },
            ListType::Numbered => "%d.".to_string(),
            ListType::Alphabetic => "%a)".to_string(),
            ListType::Roman => "%i.".to_string(),
            ListType::Korean => "%가.".to_string(),
            ListType::Custom(format) => format.clone(),
        }
    }
}

/// List style configuration
#[derive(Debug, Clone)]
pub struct ListStyle {
    pub list_type: ListType,
    pub text_style: TextStyle,
    pub indent: i32,
    pub spacing: i32,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            list_type: ListType::Bullet,
            text_style: TextStyle::default(),
            indent: 1000, // 10mm indent
            spacing: 0,
        }
    }
}

/// Table style configuration
#[derive(Debug, Clone)]
pub struct TableStyle {
    pub border_width: u8,
    pub border_color: u32,
    pub background_color: Option<u32>,
    pub padding: i32,
    pub header_style: Option<TextStyle>,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            border_width: 1,
            border_color: 0x000000,
            background_color: None,
            padding: 100, // 1mm padding
            header_style: Some(TextStyle::new().bold()),
        }
    }
}

/// Cell alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellAlign {
    Left,
    Center,
    Right,
}

/// Border style for individual table cells
#[derive(Debug, Clone)]
pub struct CellBorderStyle {
    pub left: BorderLineStyle,
    pub right: BorderLineStyle,
    pub top: BorderLineStyle,
    pub bottom: BorderLineStyle,
}

/// Style for individual border lines
#[derive(Debug, Clone)]
pub struct BorderLineStyle {
    pub line_type: BorderLineType,
    pub thickness: u8,
    pub color: u32,
}

/// Types of border lines
#[derive(Debug, Clone, Copy)]
pub enum BorderLineType {
    None = 0,
    Solid = 1,
    Dashed = 2,
    Dotted = 3,
    Double = 4,
    Thick = 5,
}

impl Default for CellBorderStyle {
    fn default() -> Self {
        Self {
            left: BorderLineStyle::default(),
            right: BorderLineStyle::default(),
            top: BorderLineStyle::default(),
            bottom: BorderLineStyle::default(),
        }
    }
}

impl Default for BorderLineStyle {
    fn default() -> Self {
        Self {
            line_type: BorderLineType::Solid,
            thickness: 1,
            color: 0x000000, // Black
        }
    }
}

impl BorderLineStyle {
    pub fn new(line_type: BorderLineType, thickness: u8, color: u32) -> Self {
        Self {
            line_type,
            thickness,
            color,
        }
    }
    
    pub fn none() -> Self {
        Self {
            line_type: BorderLineType::None,
            thickness: 0,
            color: 0,
        }
    }
    
    pub fn solid(thickness: u8) -> Self {
        Self {
            line_type: BorderLineType::Solid,
            thickness,
            color: 0x000000,
        }
    }
    
    pub fn dashed(thickness: u8) -> Self {
        Self {
            line_type: BorderLineType::Dashed,
            thickness,
            color: 0x000000,
        }
    }
    
    pub fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Convert to HWP BorderLine format
    pub fn to_border_line(&self) -> crate::model::border_fill::BorderLine {
        crate::model::border_fill::BorderLine {
            line_type: self.line_type as u8,
            thickness: self.thickness,
            color: self.color,
        }
    }
}

impl CellBorderStyle {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn all_borders(style: BorderLineStyle) -> Self {
        Self {
            left: style.clone(),
            right: style.clone(),
            top: style.clone(),
            bottom: style,
        }
    }
    
    pub fn no_borders() -> Self {
        let none_style = BorderLineStyle::none();
        Self {
            left: none_style.clone(),
            right: none_style.clone(),
            top: none_style.clone(),
            bottom: none_style,
        }
    }
    
    pub fn outer_borders() -> Self {
        Self {
            left: BorderLineStyle::solid(1),
            right: BorderLineStyle::solid(1),
            top: BorderLineStyle::solid(1),
            bottom: BorderLineStyle::solid(1),
        }
    }
    
    pub fn set_left(mut self, style: BorderLineStyle) -> Self {
        self.left = style;
        self
    }
    
    pub fn set_right(mut self, style: BorderLineStyle) -> Self {
        self.right = style;
        self
    }
    
    pub fn set_top(mut self, style: BorderLineStyle) -> Self {
        self.top = style;
        self
    }
    
    pub fn set_bottom(mut self, style: BorderLineStyle) -> Self {
        self.bottom = style;
        self
    }

    /// Convert to HWP BorderFill format
    pub fn to_border_fill(&self) -> crate::model::border_fill::BorderFill {
        use crate::model::border_fill::{BorderFill, FillInfo};
        
        BorderFill {
            properties: 0,
            left: self.left.to_border_line(),
            right: self.right.to_border_line(),
            top: self.top.to_border_line(),
            bottom: self.bottom.to_border_line(),
            diagonal: crate::model::border_fill::BorderLine {
                line_type: 0,
                thickness: 0,
                color: 0,
            },
            fill_info: FillInfo {
                fill_type: 0, // 0 = no fill
                back_color: 0xFFFFFF,
                pattern_color: 0x000000,
                pattern_type: 0,
                image_info: None,
                gradient_info: None,
            },
        }
    }
}

/// Table builder for creating tables with advanced features
pub struct TableBuilder<'a> {
    writer: &'a mut crate::HwpWriter,
    #[allow(dead_code)]
    rows: u32,
    cols: u32,
    cells: Vec<Vec<String>>,
    has_header: bool,
    #[allow(dead_code)]
    style: TableStyle,
    /// Cell merge information: (row, col) -> (row_span, col_span)
    merged_cells: std::collections::HashMap<(u32, u32), (u16, u16)>,
    /// Cell border styles: (row, col) -> BorderStyle
    cell_borders: std::collections::HashMap<(u32, u32), CellBorderStyle>,
}

impl<'a> TableBuilder<'a> {
    pub fn new(writer: &'a mut crate::HwpWriter, rows: u32, cols: u32) -> Self {
        Self {
            writer,
            rows,
            cols,
            cells: vec![vec![String::new(); cols as usize]; rows as usize],
            has_header: false,
            style: TableStyle::default(),
            merged_cells: std::collections::HashMap::new(),
            cell_borders: std::collections::HashMap::new(),
        }
    }

    /// Set whether the first row is a header
    pub fn set_header_row(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Set a cell's content
    pub fn set_cell(mut self, row: u32, col: u32, text: &str) -> Self {
        if (row as usize) < self.cells.len() && (col as usize) < self.cells[0].len() {
            self.cells[row as usize][col as usize] = text.to_string();
        }
        self
    }

    /// Set table style
    pub fn set_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Merge cells horizontally or vertically
    pub fn merge_cells(mut self, start_row: u32, start_col: u32, row_span: u16, col_span: u16) -> Self {
        self.merged_cells.insert((start_row, start_col), (row_span, col_span));
        self
    }
    
    /// Set border style for a specific cell
    pub fn set_cell_border(mut self, row: u32, col: u32, border_style: CellBorderStyle) -> Self {
        self.cell_borders.insert((row, col), border_style);
        self
    }
    
    /// Set border style for a range of cells
    pub fn set_range_border(mut self, start_row: u32, start_col: u32, end_row: u32, end_col: u32, border_style: CellBorderStyle) -> Self {
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                self.cell_borders.insert((row, col), border_style.clone());
            }
        }
        self
    }
    
    /// Set outer borders for the entire table
    pub fn set_outer_borders(mut self, border_style: BorderLineStyle) -> Self {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut cell_border = self.cell_borders.get(&(row, col)).cloned().unwrap_or_default();
                
                // Set outer borders
                if row == 0 {
                    cell_border.top = border_style.clone();
                }
                if row == self.rows - 1 {
                    cell_border.bottom = border_style.clone();
                }
                if col == 0 {
                    cell_border.left = border_style.clone();
                }
                if col == self.cols - 1 {
                    cell_border.right = border_style.clone();
                }
                
                self.cell_borders.insert((row, col), cell_border);
            }
        }
        self
    }
    
    /// Set inner borders for the table (between cells)
    pub fn set_inner_borders(mut self, border_style: BorderLineStyle) -> Self {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut cell_border = self.cell_borders.get(&(row, col)).cloned().unwrap_or_default();
                
                // Set inner borders (except outer edges)
                if row > 0 {
                    cell_border.top = border_style.clone();
                }
                if row < self.rows - 1 {
                    cell_border.bottom = border_style.clone();
                }
                if col > 0 {
                    cell_border.left = border_style.clone();
                }
                if col < self.cols - 1 {
                    cell_border.right = border_style.clone();
                }
                
                self.cell_borders.insert((row, col), cell_border);
            }
        }
        self
    }
    
    /// Set all borders (both inner and outer)
    pub fn set_all_borders(self, border_style: BorderLineStyle) -> Self {
        self.set_outer_borders(border_style.clone())
            .set_inner_borders(border_style)
    }
    
    /// Remove all borders from the table
    pub fn no_borders(mut self) -> Self {
        let no_border = CellBorderStyle::no_borders();
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.cell_borders.insert((row, col), no_border.clone());
            }
        }
        self
    }

    /// Finish building the table and add it to the document
    pub fn finish(self) -> crate::error::Result<()> {
        use crate::model::{
            control::Table,
            ctrl_header::{CtrlHeader, ControlType},
            paragraph::{Paragraph, ParaText},
            para_char_shape::{ParaCharShape, CharPositionShape},
        };
        
        // Calculate dimensions (using HWP units: 1mm = ~100 units)
        let col_width = 5000u32; // 50mm per column
        let row_height = 1000u32; // 10mm per row
        
        // Create the table structure first
        let mut table = Table::new_default(self.rows as u16, self.cols as u16);
        
        // Create border fills for each unique cell border style
        let mut border_fill_map = std::collections::HashMap::new();
        let mut next_border_fill_id = 1u16;
        
        // Create cell paragraphs and link them to table cells
        let mut cell_paragraphs = Vec::new();
        let mut paragraph_list_counter = 0u32;
        
        for (row_idx, row) in self.cells.iter().enumerate() {
            for (col_idx, cell_text) in row.iter().enumerate() {
                let row = row_idx as u16;
                let col = col_idx as u16;
                let cell_key = (row_idx as u32, col_idx as u32);
                
                // Check if this cell is part of a merged cell
                let mut is_merged_target = false;
                let (row_span, col_span) = if let Some(&(rs, cs)) = self.merged_cells.get(&cell_key) {
                    (rs, cs)
                } else {
                    // Check if this cell is covered by another merged cell
                    for (&(merge_row, merge_col), &(rs, cs)) in &self.merged_cells {
                        if (row_idx as u32 >= merge_row) && ((row_idx as u32) < merge_row + rs as u32) &&
                           (col_idx as u32 >= merge_col) && ((col_idx as u32) < merge_col + cs as u32) &&
                           (row_idx as u32 != merge_row || col_idx as u32 != merge_col) {
                            is_merged_target = true;
                            break;
                        }
                    }
                    (1, 1) // Default: no merge
                };
                
                // Skip cells that are covered by merged cells
                if is_merged_target {
                    continue;
                }
                
                // Get or create border fill for this cell
                let border_fill_id = if let Some(cell_border) = self.cell_borders.get(&cell_key) {
                    let border_fill = cell_border.to_border_fill();
                    let border_key = format!("{:?}", border_fill);
                    
                    if let Some(&existing_id) = border_fill_map.get(&border_key) {
                        existing_id
                    } else {
                        let new_id = next_border_fill_id;
                        next_border_fill_id += 1;
                        
                        // Add the border fill to the document
                        self.writer.document.doc_info.border_fills.push(border_fill);
                        border_fill_map.insert(border_key, new_id);
                        new_id
                    }
                } else {
                    0 // Default border fill
                };
                
                // Create and add table cell with proper addressing and merge info
                let cell = table.create_cell(row, col, col_width * col_span as u32, row_height * row_span as u32);
                cell.list_header_id = paragraph_list_counter;
                cell.paragraph_list_id = Some(paragraph_list_counter);
                cell.row_span = row_span;
                cell.col_span = col_span;
                cell.border_fill_id = border_fill_id;
                paragraph_list_counter += 1;
                
                // Create paragraph for cell content
                let para_text = ParaText {
                    content: cell_text.clone(),
                };
                
                // Use header style for first row if header is enabled
                let char_shape_id = if self.has_header && row_idx == 0 {
                    // Get or create bold char shape for header
                    if let Some(header_style) = &self.style.header_style {
                        let face_name_id = if let Some(font_name) = &header_style.font_name {
                            self.writer.ensure_font(font_name)?
                        } else {
                            0
                        };
                        let char_shape = header_style.to_char_shape(face_name_id);
                        self.writer.add_char_shape(char_shape)?
                    } else {
                        0
                    }
                } else {
                    0 // Default char shape
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
                    instance_id: cell.list_header_id,
                    char_shapes: if char_shape_id > 0 {
                        Some(ParaCharShape {
                            char_positions: vec![CharPositionShape {
                                position: 0,
                                char_shape_id,
                            }],
                        })
                    } else {
                        None
                    },
                    line_segments: None,
                    list_header: None,
                    ctrl_header: None,
                    table_data: None,
                    picture_data: None,
                    text_box_data: None,
                    hyperlinks: Vec::new(),
                };
                cell_paragraphs.push(paragraph);
            }
        }
        
        // Create control header for table
        let ctrl_header = CtrlHeader {
            ctrl_id: ControlType::Table as u32,
            properties: 0,
            instance_id: self.writer.next_instance_id(),
        };
        
        // Create a paragraph with table control AND actual table data
        let table_paragraph = Paragraph {
            text: None,
            control_mask: 1, // Indicates control is present
            para_shape_id: 0,
            style_id: 0,
            column_type: 0,
            char_shape_count: 0,
            range_tag_count: 0,
            line_align_count: 0,
            instance_id: self.writer.next_instance_id(),
            char_shapes: None,
            line_segments: None,
            list_header: None,
            ctrl_header: Some(ctrl_header),
            table_data: Some(table), // Store actual table data with proper cell linking
            picture_data: None,
            text_box_data: None,
            hyperlinks: Vec::new(),
        };
        
        // Add the table paragraph to the document
        if let Some(body_text) = self.writer.document.body_texts.get_mut(self.writer.current_section_idx) {
            if let Some(section) = body_text.sections.get_mut(0) {
                section.paragraphs.push(table_paragraph);
                // Add cell paragraphs (these are now properly linked via paragraph_list_id)
                section.paragraphs.extend(cell_paragraphs);
            }
        }
        
        Ok(())
    }
}

/// Image format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Bmp,
    Gif,
}

impl ImageFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Gif => "gif",
        }
    }
    
    /// Detect format from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        
        // Check magic bytes
        match &data[0..4] {
            [0xFF, 0xD8, 0xFF, _] => Some(ImageFormat::Jpeg),
            [0x89, 0x50, 0x4E, 0x47] => Some(ImageFormat::Png),
            [0x42, 0x4D, _, _] => Some(ImageFormat::Bmp),
            [0x47, 0x49, 0x46, _] => Some(ImageFormat::Gif),
            _ => None,
        }
    }
}

/// Image alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageAlign {
    Left,
    Center,
    Right,
    InlineWithText,
}

/// Image insertion options
#[derive(Debug, Clone)]
pub struct ImageOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alignment: ImageAlign,
    pub wrap_text: bool,
    pub caption: Option<String>,
}

/// Text range with specific styling
#[derive(Debug, Clone)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
    pub style: TextStyle,
}

impl TextRange {
    pub fn new(start: usize, end: usize, style: TextStyle) -> Self {
        Self { start, end, style }
    }
    
    /// Create a range that spans the entire text
    pub fn entire_text(text_len: usize, style: TextStyle) -> Self {
        Self {
            start: 0,
            end: text_len,
            style,
        }
    }
}

/// Helper for building text with multiple styles
#[derive(Debug, Clone)]
pub struct StyledText {
    pub text: String,
    pub ranges: Vec<TextRange>,
}

impl StyledText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            ranges: Vec::new(),
        }
    }
    
    /// Add a styled range to the text
    pub fn add_range(mut self, start: usize, end: usize, style: TextStyle) -> Self {
        self.ranges.push(TextRange::new(start, end, style));
        self
    }
    
    /// Apply style to a substring by finding its position
    pub fn style_substring(mut self, substring: &str, style: TextStyle) -> Self {
        if let Some(start) = self.text.find(substring) {
            let end = start + substring.len();
            self.ranges.push(TextRange::new(start, end, style));
        }
        self
    }
    
    /// Apply style to all occurrences of a substring
    pub fn style_all_occurrences(mut self, substring: &str, style: TextStyle) -> Self {
        let mut start_pos = 0;
        while let Some(found) = self.text[start_pos..].find(substring) {
            let absolute_start = start_pos + found;
            let absolute_end = absolute_start + substring.len();
            self.ranges.push(TextRange::new(absolute_start, absolute_end, style.clone()));
            start_pos = absolute_end;
        }
        self
    }
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            alignment: ImageAlign::InlineWithText,
            wrap_text: false,
            caption: None,
        }
    }
}

impl ImageOptions {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set image width in millimeters
    pub fn width(mut self, width_mm: u32) -> Self {
        self.width = Some(width_mm);
        self
    }
    
    /// Set image height in millimeters
    pub fn height(mut self, height_mm: u32) -> Self {
        self.height = Some(height_mm);
        self
    }
    
    /// Set image alignment
    pub fn align(mut self, alignment: ImageAlign) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Enable text wrapping around image
    pub fn wrap_text(mut self, wrap: bool) -> Self {
        self.wrap_text = wrap;
        self
    }
    
    /// Add caption to image
    pub fn caption(mut self, text: &str) -> Self {
        self.caption = Some(text.to_string());
        self
    }
}