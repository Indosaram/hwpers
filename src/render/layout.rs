use crate::model::{Section, Paragraph, CharShape, ParaShape, ParaLineSeg};
use crate::HwpDocument;

/// Represents a single rendered line of text
#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub y: i32,           // Vertical position in HWP units
    pub height: i32,      // Line height in HWP units
    pub baseline_y: i32,  // Baseline position
    pub runs: Vec<TextRun>,
}

/// Represents a run of text with consistent formatting
#[derive(Debug, Clone)]
pub struct TextRun {
    pub x: i32,               // Horizontal position in HWP units
    pub width: i32,           // Width of the run
    pub text: String,         // Text content
    pub char_shape_id: u16,   // Character shape ID
    pub font_size: i32,       // Actual font size in HWP units
}

/// Represents a rendered paragraph
#[derive(Debug, Clone)]
pub struct RenderedParagraph {
    pub x: i32,                    // Left position
    pub y: i32,                    // Top position
    pub width: i32,                // Paragraph width
    pub height: i32,               // Total height
    pub lines: Vec<RenderedLine>,  // Rendered lines
    pub para_shape_id: u16,        // Paragraph shape ID
}

/// Represents a rendered page
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub width: u32,                       // Page width
    pub height: u32,                      // Page height
    pub paragraphs: Vec<RenderedParagraph>, // Rendered paragraphs
    pub page_number: u32,                 // Page number
}

/// Layout calculation result
#[derive(Debug)]
pub struct LayoutResult {
    pub pages: Vec<RenderedPage>,
    pub total_height: i32,
}

/// Layout engine for HWP documents
pub struct LayoutEngine<'a> {
    document: &'a HwpDocument,
}

impl<'a> LayoutEngine<'a> {
    pub fn new(document: &'a HwpDocument) -> Self {
        Self { document }
    }
    
    /// Perform full layout calculation for the document
    pub fn calculate_layout(&self) -> LayoutResult {
        let mut pages = Vec::new();
        let mut current_page_num = 1;
        
        for section in self.document.sections() {
            let section_pages = self.layout_section(section, &mut current_page_num);
            pages.extend(section_pages);
        }
        
        let total_height = pages.iter()
            .map(|p| p.height as i32)
            .sum();
        
        LayoutResult { pages, total_height }
    }
    
    /// Layout a single section
    fn layout_section(&self, section: &Section, page_num: &mut u32) -> Vec<RenderedPage> {
        let mut pages = Vec::new();
        
        // Get page definition
        let page_def = section.page_def.as_ref()
            .expect("Section must have page definition");
        
        let mut current_page = RenderedPage {
            width: page_def.width,
            height: page_def.height,
            paragraphs: Vec::new(),
            page_number: *page_num,
        };
        
        // Calculate content area
        let content_x = page_def.left_margin as i32;
        let content_y = page_def.top_margin as i32;
        let content_width = page_def.effective_width() as i32;
        let content_height = page_def.effective_height() as i32;
        
        let mut current_y = content_y;
        
        // Layout each paragraph
        for paragraph in &section.paragraphs {
            if let Some(rendered_para) = self.layout_paragraph(
                paragraph, 
                content_x, 
                current_y, 
                content_width
            ) {
                // Check if paragraph fits on current page
                if current_y + rendered_para.height > content_y + content_height {
                    // Start new page
                    pages.push(current_page);
                    *page_num += 1;
                    
                    current_page = RenderedPage {
                        width: page_def.width,
                        height: page_def.height,
                        paragraphs: Vec::new(),
                        page_number: *page_num,
                    };
                    
                    current_y = content_y;
                }
                
                current_y += rendered_para.height;
                current_page.paragraphs.push(rendered_para);
            }
        }
        
        if !current_page.paragraphs.is_empty() {
            pages.push(current_page);
            *page_num += 1;
        }
        
        pages
    }
    
    /// Layout a single paragraph
    fn layout_paragraph(
        &self, 
        paragraph: &Paragraph, 
        x: i32, 
        y: i32, 
        width: i32
    ) -> Option<RenderedParagraph> {
        let text = paragraph.text.as_ref()?.content.as_str();
        if text.is_empty() {
            return None;
        }
        
        // Get paragraph shape
        let para_shape = self.document.get_para_shape(paragraph.para_shape_id as usize)?;
        
        // Calculate paragraph margins
        let para_x = x + para_shape.left_margin;
        let para_width = width - para_shape.left_margin - para_shape.right_margin;
        
        // Check if paragraph has pre-calculated line segments
        if let Some(line_segs) = &paragraph.line_segments {
            // Use pre-calculated line layout
            let lines = self.render_with_line_segments(
                text,
                paragraph,
                line_segs,
                para_x,
                y
            );
            
            let total_height = line_segs.total_height();
            
            return Some(RenderedParagraph {
                x: para_x,
                y,
                width: para_width,
                height: total_height,
                lines,
                para_shape_id: paragraph.para_shape_id,
            });
        }
        
        // Otherwise, calculate line breaks dynamically
        let lines = self.calculate_line_breaks(
            text,
            paragraph,
            para_shape,
            para_x,
            y,
            para_width
        );
        
        let total_height = lines.iter()
            .map(|l| l.height)
            .sum();
        
        Some(RenderedParagraph {
            x: para_x,
            y,
            width: para_width,
            height: total_height,
            lines,
            para_shape_id: paragraph.para_shape_id,
        })
    }
    
    /// Render using pre-calculated line segments
    fn render_with_line_segments(
        &self,
        text: &str,
        paragraph: &Paragraph,
        line_segs: &ParaLineSeg,
        x: i32,
        y: i32
    ) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        let mut current_y = y;
        
        for (idx, line_seg) in line_segs.line_segments.iter().enumerate() {
            // Get text range for this line
            let start = if idx == 0 { 0 } else { line_segs.line_segments[idx-1].text_start_position as usize };
            let end = line_seg.text_start_position as usize;
            
            if end > text.len() {
                break;
            }
            
            let line_text = &text[start..end];
            
            // Create text runs for this line
            let runs = self.create_text_runs_for_line(
                line_text,
                paragraph,
                x,
                start as u32
            );
            
            lines.push(RenderedLine {
                y: current_y + line_seg.line_vertical_position,
                height: line_seg.line_height,
                baseline_y: current_y + line_seg.line_vertical_position + line_seg.distance_baseline_to_line_vertical_position,
                runs,
            });
            
            current_y += line_seg.line_height;
        }
        
        lines
    }
    
    /// Calculate line breaks dynamically
    fn calculate_line_breaks(
        &self,
        text: &str,
        paragraph: &Paragraph,
        para_shape: &ParaShape,
        x: i32,
        y: i32,
        _width: i32
    ) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        let mut current_y = y + para_shape.top_para_space;
        
        // Get default character shape
        let default_char_shape_id = if let Some(char_shapes) = &paragraph.char_shapes {
            char_shapes.char_positions.first()
                .map(|cs| cs.char_shape_id)
                .unwrap_or(0)
        } else {
            0
        };
        
        let char_shape = self.document.get_char_shape(default_char_shape_id as usize);
        let line_height = self.calculate_line_height(para_shape, char_shape);
        
        // Simple line breaking - split by newlines for now
        for line_text in text.lines() {
            if line_text.is_empty() {
                current_y += line_height;
                continue;
            }
            
            let runs = self.create_text_runs_for_line(
                line_text,
                paragraph,
                x,
                0
            );
            
            lines.push(RenderedLine {
                y: current_y,
                height: line_height,
                baseline_y: current_y + (line_height * 4 / 5), // Approximate baseline
                runs,
            });
            
            current_y += line_height;
        }
        
        lines
    }
    
    /// Create text runs for a line of text
    fn create_text_runs_for_line(
        &self,
        line_text: &str,
        paragraph: &Paragraph,
        x: i32,
        text_offset: u32
    ) -> Vec<TextRun> {
        let mut runs = Vec::new();
        
        if let Some(char_shapes) = &paragraph.char_shapes {
            // Create runs based on character shape changes
            let mut current_x = x;
            
            for (idx, pos_shape) in char_shapes.char_positions.iter().enumerate() {
                let start = pos_shape.position as usize;
                let end = if idx + 1 < char_shapes.char_positions.len() {
                    char_shapes.char_positions[idx + 1].position as usize
                } else {
                    line_text.len() + text_offset as usize
                };
                
                if start >= text_offset as usize && (start - text_offset as usize) < line_text.len() {
                    let run_start = (start - text_offset as usize).min(line_text.len());
                    let run_end = (end - text_offset as usize).min(line_text.len());
                    
                    if run_start < run_end {
                        let run_text = &line_text[run_start..run_end];
                        let char_shape = self.document.get_char_shape(pos_shape.char_shape_id as usize);
                        
                        let width = self.calculate_text_width(run_text, char_shape);
                        
                        runs.push(TextRun {
                            x: current_x,
                            width,
                            text: run_text.to_string(),
                            char_shape_id: pos_shape.char_shape_id,
                            font_size: char_shape.map(|cs| cs.base_size).unwrap_or(1000),
                        });
                        
                        current_x += width;
                    }
                }
            }
        } else {
            // Single run for entire line
            let width = self.calculate_text_width(line_text, None);
            
            runs.push(TextRun {
                x,
                width,
                text: line_text.to_string(),
                char_shape_id: 0,
                font_size: 1000, // Default 10pt
            });
        }
        
        runs
    }
    
    /// Calculate line height based on paragraph and character shapes
    fn calculate_line_height(&self, para_shape: &ParaShape, char_shape: Option<&CharShape>) -> i32 {
        let base_size = char_shape.map(|cs| cs.base_size).unwrap_or(1000);
        
        match para_shape.line_space_type {
            0 => {
                // Percentage
                base_size * para_shape.line_space / 100
            }
            1 => {
                // Fixed value
                para_shape.line_space
            }
            2 => {
                // At least
                base_size.max(para_shape.line_space)
            }
            _ => base_size
        }
    }
    
    /// Calculate text width (simplified)
    fn calculate_text_width(&self, text: &str, char_shape: Option<&CharShape>) -> i32 {
        let font_size = char_shape.map(|cs| cs.base_size).unwrap_or(1000);
        
        // Approximate width calculation
        // In reality, this would use font metrics
        let char_count = text.chars().count() as i32;
        let avg_char_width = font_size / 2; // Rough approximation
        
        char_count * avg_char_width
    }
}