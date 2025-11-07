use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct ParaCharShape {
    pub char_positions: Vec<CharPositionShape>,
}

#[derive(Debug, Clone)]
pub struct CharPositionShape {
    pub position: u32,
    pub char_shape_id: u16,
}

impl ParaCharShape {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        let mut char_positions = Vec::new();

        // Read position-shape pairs
        while reader.remaining() >= 8 {
            let position = reader.read_u32()?;
            let char_shape_id = reader.read_u32()? as u16; // Only lower 16 bits used

            char_positions.push(CharPositionShape {
                position,
                char_shape_id,
            });
        }

        Ok(Self { char_positions })
    }

    /// Get the character shape ID at a specific character position
    pub fn get_shape_at_position(&self, char_pos: u32) -> Option<u16> {
        // Find the shape that applies to this position
        let mut current_shape = None;

        for pos_shape in &self.char_positions {
            if pos_shape.position <= char_pos {
                current_shape = Some(pos_shape.char_shape_id);
            } else {
                break;
            }
        }

        current_shape
    }

    /// Create a new ParaCharShape with a single character shape for entire text
    pub fn new_single_shape(char_shape_id: u16) -> Self {
        Self {
            char_positions: vec![CharPositionShape {
                position: 0,
                char_shape_id,
            }],
        }
    }

    /// Create a new ParaCharShape with multiple character shapes for text ranges
    pub fn new_with_ranges(ranges: Vec<(u32, u16)>) -> Self {
        let mut char_positions = Vec::new();

        for (position, char_shape_id) in ranges {
            char_positions.push(CharPositionShape {
                position,
                char_shape_id,
            });
        }

        // Sort by position to ensure correct ordering
        char_positions.sort_by_key(|p| p.position);

        Self { char_positions }
    }

    /// Add a character shape change at a specific position
    pub fn add_shape_at_position(&mut self, position: u32, char_shape_id: u16) {
        // Remove any existing shape at this exact position
        self.char_positions.retain(|p| p.position != position);

        // Add new shape
        self.char_positions.push(CharPositionShape {
            position,
            char_shape_id,
        });

        // Re-sort by position
        self.char_positions.sort_by_key(|p| p.position);
    }

    /// Apply a character shape to a range of characters
    pub fn apply_shape_to_range(&mut self, start_pos: u32, end_pos: u32, char_shape_id: u16) {
        // Remove any shapes that are completely within this range
        self.char_positions
            .retain(|p| !(p.position > start_pos && p.position < end_pos));

        // Add shape at start position
        self.add_shape_at_position(start_pos, char_shape_id);

        // If there was a different shape before start_pos, restore it at end_pos
        if let Some(prev_shape) = self.get_shape_before_position(start_pos) {
            if prev_shape != char_shape_id {
                self.add_shape_at_position(end_pos, prev_shape);
            }
        }
    }

    /// Get the character shape ID that was active before a specific position
    fn get_shape_before_position(&self, position: u32) -> Option<u16> {
        let mut prev_shape = None;

        for pos_shape in &self.char_positions {
            if pos_shape.position < position {
                prev_shape = Some(pos_shape.char_shape_id);
            } else {
                break;
            }
        }

        prev_shape
    }

    /// Convert to bytes for serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        // Write number of position shapes
        writer
            .write_u32::<LittleEndian>(self.char_positions.len() as u32)
            .unwrap();

        // Write each position-shape pair
        for char_pos in &self.char_positions {
            writer.write_u32::<LittleEndian>(char_pos.position).unwrap();
            writer
                .write_u32::<LittleEndian>(char_pos.char_shape_id as u32)
                .unwrap();
        }

        data
    }
}
