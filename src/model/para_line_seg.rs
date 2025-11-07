use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct ParaLineSeg {
    pub line_segments: Vec<LineSegment>,
}

#[derive(Debug, Clone)]
pub struct LineSegment {
    pub text_start_position: u32,
    pub line_vertical_position: i32,
    pub line_height: i32,
    pub text_part_height: i32,
    pub distance_baseline_to_line_vertical_position: i32,
    pub line_space: i32,
    pub segment_width: i32,
    pub properties: u32,
}

impl ParaLineSeg {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        let mut line_segments = Vec::new();

        // Each line segment is 32 bytes
        while reader.remaining() >= 32 {
            let segment = LineSegment {
                text_start_position: reader.read_u32()?,
                line_vertical_position: reader.read_i32()?,
                line_height: reader.read_i32()?,
                text_part_height: reader.read_i32()?,
                distance_baseline_to_line_vertical_position: reader.read_i32()?,
                line_space: reader.read_i32()?,
                segment_width: reader.read_i32()?,
                properties: reader.read_u32()?,
            };

            line_segments.push(segment);
        }

        Ok(Self { line_segments })
    }

    /// Get the total height of all lines
    pub fn total_height(&self) -> i32 {
        self.line_segments.iter().map(|seg| seg.line_height).sum()
    }

    /// Get the line segment that contains a specific text position
    pub fn get_line_at_position(&self, text_pos: u32) -> Option<&LineSegment> {
        for (i, segment) in self.line_segments.iter().enumerate() {
            let next_pos = self
                .line_segments
                .get(i + 1)
                .map(|s| s.text_start_position)
                .unwrap_or(u32::MAX);

            if text_pos >= segment.text_start_position && text_pos < next_pos {
                return Some(segment);
            }
        }
        None
    }
}
impl LineSegment {
    /// Create a new line segment
    pub fn new(
        text_start_position: u32,
        line_vertical_position: i32,
        line_height: i32,
        segment_width: i32,
    ) -> Self {
        Self {
            text_start_position,
            line_vertical_position,
            line_height,
            text_part_height: line_height - (line_height / 6), // Typical text height is ~5/6 of line height
            distance_baseline_to_line_vertical_position: line_height / 6, // Baseline offset
            line_space: 0,                                     // No extra line spacing by default
            segment_width,
            properties: 0, // No special properties
        }
    }

    /// Create a line segment with custom text and baseline heights
    pub fn new_with_heights(
        text_start_position: u32,
        line_vertical_position: i32,
        line_height: i32,
        text_part_height: i32,
        baseline_distance: i32,
        segment_width: i32,
    ) -> Self {
        Self {
            text_start_position,
            line_vertical_position,
            line_height,
            text_part_height,
            distance_baseline_to_line_vertical_position: baseline_distance,
            line_space: 0,
            segment_width,
            properties: 0,
        }
    }

    /// Set line spacing
    pub fn with_line_space(mut self, line_space: i32) -> Self {
        self.line_space = line_space;
        self
    }

    /// Set properties (alignment, etc.)
    pub fn with_properties(mut self, properties: u32) -> Self {
        self.properties = properties;
        self
    }
}

impl ParaLineSeg {
    /// Create a new empty line segments collection
    pub fn new() -> Self {
        Self {
            line_segments: Vec::new(),
        }
    }

    /// Create line segments for single-line text
    pub fn new_single_line(_text_length: u32, line_height: i32, width: i32) -> Self {
        let segment = LineSegment::new(0, 0, line_height, width);
        Self {
            line_segments: vec![segment],
        }
    }

    /// Create line segments for multi-line text with automatic wrapping
    pub fn new_multi_line(
        text: &str,
        line_height: i32,
        max_width: i32,
        average_char_width: i32,
    ) -> Self {
        let mut line_segments = Vec::new();
        let chars_per_line = (max_width / average_char_width.max(1)) as usize;

        if chars_per_line == 0 {
            // If width is too small, create single line with text
            let segment = LineSegment::new(0, 0, line_height, max_width);
            return Self {
                line_segments: vec![segment],
            };
        }

        let mut current_position = 0u32;
        let mut line_y = 0i32;

        let text_chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < text_chars.len() {
            let line_start = i;
            let mut line_end = (i + chars_per_line).min(text_chars.len());

            // Try to break at word boundaries
            if line_end < text_chars.len() {
                // Look back for a space to break on
                let mut break_point = line_end;
                for j in (line_start..line_end).rev() {
                    if text_chars[j].is_whitespace() {
                        break_point = j + 1;
                        break;
                    }
                }

                // If we found a good break point, use it
                if break_point > line_start {
                    line_end = break_point;
                }
            }

            let line_chars = line_end - line_start;
            let line_width = (line_chars as i32 * average_char_width).min(max_width);

            let segment = LineSegment::new(current_position, line_y, line_height, line_width);

            line_segments.push(segment);

            current_position += line_chars as u32;
            line_y += line_height;
            i = line_end;
        }

        Self { line_segments }
    }

    /// Add a line segment
    pub fn add_segment(&mut self, segment: LineSegment) {
        self.line_segments.push(segment);
    }

    /// Calculate total paragraph width (widest line)
    pub fn max_width(&self) -> i32 {
        self.line_segments
            .iter()
            .map(|seg| seg.segment_width)
            .max()
            .unwrap_or(0)
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.line_segments.len()
    }

    /// Get line at index
    pub fn get_line(&self, index: usize) -> Option<&LineSegment> {
        self.line_segments.get(index)
    }

    /// Serialize to bytes for HWP format
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        for segment in &self.line_segments {
            writer
                .write_u32::<LittleEndian>(segment.text_start_position)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.line_vertical_position)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.line_height)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.text_part_height)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.distance_baseline_to_line_vertical_position)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.line_space)
                .unwrap();
            writer
                .write_i32::<LittleEndian>(segment.segment_width)
                .unwrap();
            writer
                .write_u32::<LittleEndian>(segment.properties)
                .unwrap();
        }

        data
    }
}

impl LineSegment {
    pub fn is_first_line(&self) -> bool {
        (self.properties & 0x01) != 0
    }

    pub fn is_last_line(&self) -> bool {
        (self.properties & 0x02) != 0
    }

    pub fn is_empty_line(&self) -> bool {
        (self.properties & 0x04) != 0
    }

    pub fn has_line_control(&self) -> bool {
        (self.properties & 0x08) != 0
    }
}
