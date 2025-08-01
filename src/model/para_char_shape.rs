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
}
