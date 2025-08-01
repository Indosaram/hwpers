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
        self.line_segments.iter()
            .map(|seg| seg.line_height)
            .sum()
    }
    
    /// Get the line segment that contains a specific text position
    pub fn get_line_at_position(&self, text_pos: u32) -> Option<&LineSegment> {
        for (i, segment) in self.line_segments.iter().enumerate() {
            let next_pos = self.line_segments.get(i + 1)
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