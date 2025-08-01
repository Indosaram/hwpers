use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct Numbering {
    pub levels: Vec<NumberingLevel>,
}

#[derive(Debug, Clone)]
pub struct NumberingLevel {
    pub para_shape_id: u16,
    pub number_format: u8,
    pub number_type: u8,
    pub prefix_text: String,
    pub suffix_text: String,
    pub auto_indent: u8,
    pub text_offset_type: u8,
    pub width_adjust_type: u8,
    pub text_offset: i16,
    pub number_width: u16,
    pub char_shape_id: u16,
}

#[derive(Debug, Clone)]
pub struct Bullet {
    pub para_shape_id: u16,
    pub bullet_char: String,
    pub char_shape_id: u16,
    pub use_image: bool,
    pub image_bullet: Option<ImageBullet>,
}

#[derive(Debug, Clone)]
pub struct ImageBullet {
    pub image_width: u16,
    pub image_height: u16,
    pub bin_data_id: u16,
}

impl Numbering {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        let mut levels = Vec::new();

        // Read numbering levels - typically 7 levels maximum
        for _ in 0..7 {
            if reader.remaining() < 20 {
                break; // Not enough data for a complete level
            }

            let para_shape_id = reader.read_u16()?;
            let number_format = reader.read_u8()?;
            let number_type = reader.read_u8()?;

            // Read prefix text
            let prefix_len = reader.read_u16()? as usize;
            let prefix_text = if reader.remaining() >= prefix_len * 2 {
                reader.read_string(prefix_len * 2)?
            } else {
                String::new()
            };

            // Read suffix text
            let suffix_len = reader.read_u16()? as usize;
            let suffix_text = if reader.remaining() >= suffix_len * 2 {
                reader.read_string(suffix_len * 2)?
            } else {
                String::new()
            };

            if reader.remaining() >= 10 {
                let level = NumberingLevel {
                    para_shape_id,
                    number_format,
                    number_type,
                    prefix_text,
                    suffix_text,
                    auto_indent: reader.read_u8()?,
                    text_offset_type: reader.read_u8()?,
                    width_adjust_type: reader.read_u8()?,
                    text_offset: reader.read_u16()? as i16,
                    number_width: reader.read_u16()?,
                    char_shape_id: reader.read_u16()?,
                };
                levels.push(level);
            }
        }

        Ok(Self { levels })
    }
}

impl Bullet {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 6 {
            return Err(crate::error::HwpError::ParseError(format!(
                "Bullet record too small: {} bytes",
                reader.remaining()
            )));
        }

        let para_shape_id = reader.read_u16()?;

        // Read bullet character
        let bullet_char_len = reader.read_u16()? as usize;
        let bullet_char = if reader.remaining() >= bullet_char_len * 2 {
            reader.read_string(bullet_char_len * 2)?
        } else {
            String::new()
        };

        let char_shape_id = reader.read_u16()?;

        // Check for image bullet
        let use_image = reader.remaining() >= 6;
        let image_bullet = if use_image {
            Some(ImageBullet {
                image_width: reader.read_u16()?,
                image_height: reader.read_u16()?,
                bin_data_id: reader.read_u16()?,
            })
        } else {
            None
        };

        Ok(Self {
            para_shape_id,
            bullet_char,
            char_shape_id,
            use_image,
            image_bullet,
        })
    }
}

impl NumberingLevel {
    pub fn is_decimal(&self) -> bool {
        self.number_type == 0
    }

    pub fn is_circle_num(&self) -> bool {
        self.number_type == 1
    }

    pub fn is_lower_roman(&self) -> bool {
        self.number_type == 2
    }

    pub fn is_upper_roman(&self) -> bool {
        self.number_type == 3
    }

    pub fn is_lower_alpha(&self) -> bool {
        self.number_type == 4
    }

    pub fn is_upper_alpha(&self) -> bool {
        self.number_type == 5
    }
}
