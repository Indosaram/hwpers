use crate::error::Result;
use crate::parser::record::Record;
use crate::reader::StreamReader;

#[derive(Debug, Clone)]
pub struct BorderFill {
    pub properties: u16,
    pub left: BorderLine,
    pub right: BorderLine,
    pub top: BorderLine,
    pub bottom: BorderLine,
    pub diagonal: BorderLine,
    pub fill_info: FillInfo,
}

#[derive(Debug, Clone)]
pub struct BorderLine {
    pub line_type: u8,
    pub thickness: u8,
    pub color: u32,
}

#[derive(Debug, Clone)]
pub struct FillInfo {
    pub fill_type: u32,
    pub back_color: u32,
    pub pattern_color: u32,
    pub pattern_type: u32,
    pub image_info: Option<ImageInfo>,
    pub gradient_info: Option<GradientInfo>,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub brightness: u8,
    pub contrast: u8,
    pub effect: u8,
    pub bin_data_id: u16,
}

#[derive(Debug, Clone)]
pub struct GradientInfo {
    pub gradient_type: u8,
    pub start_color: u32,
    pub end_color: u32,
    pub angle: u16,
    pub center_x: u16,
    pub center_y: u16,
    pub blur_degree: u16,
}

impl BorderFill {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();

        if reader.remaining() < 2 {
            return Err(crate::error::HwpError::ParseError(format!(
                "BorderFill record too small: {} bytes",
                reader.remaining()
            )));
        }

        let properties = reader.read_u16()?;

        // Initialize with defaults
        let mut left = BorderLine {
            line_type: 0,
            thickness: 0,
            color: 0,
        };
        let mut right = BorderLine {
            line_type: 0,
            thickness: 0,
            color: 0,
        };
        let mut top = BorderLine {
            line_type: 0,
            thickness: 0,
            color: 0,
        };
        let mut bottom = BorderLine {
            line_type: 0,
            thickness: 0,
            color: 0,
        };
        let mut diagonal = BorderLine {
            line_type: 0,
            thickness: 0,
            color: 0,
        };

        // Read border lines if available
        if reader.remaining() >= 6 {
            left = BorderLine::read(&mut reader)?;
        }
        if reader.remaining() >= 6 {
            right = BorderLine::read(&mut reader)?;
        }
        if reader.remaining() >= 6 {
            top = BorderLine::read(&mut reader)?;
        }
        if reader.remaining() >= 6 {
            bottom = BorderLine::read(&mut reader)?;
        }
        if reader.remaining() >= 6 {
            diagonal = BorderLine::read(&mut reader)?;
        }

        // Read fill info if available
        let fill_info = if reader.remaining() >= 16 {
            FillInfo::read(&mut reader)?
        } else {
            FillInfo {
                fill_type: 0,
                back_color: 0xFFFFFFFF,
                pattern_color: 0,
                pattern_type: 0,
                image_info: None,
                gradient_info: None,
            }
        };

        Ok(Self {
            properties,
            left,
            right,
            top,
            bottom,
            diagonal,
            fill_info,
        })
    }
}
impl BorderFill {
    /// Create a new default BorderFill for writing
    pub fn new_default() -> Self {
        let default_border = BorderLine {
            line_type: 0,     // No border
            thickness: 0,
            color: 0x000000,  // Black
        };

        Self {
            properties: 0,
            left: default_border.clone(),
            right: default_border.clone(),
            top: default_border.clone(),
            bottom: default_border.clone(),
            diagonal: default_border,
            fill_info: FillInfo {
                fill_type: 0,           // No fill
                back_color: 0xFFFFFFFF, // White background
                pattern_color: 0,
                pattern_type: 0,
                image_info: None,
                gradient_info: None,
            },
        }
    }
}

impl BorderLine {
    fn read(reader: &mut StreamReader) -> Result<Self> {
        if reader.remaining() < 6 {
            return Err(crate::error::HwpError::ParseError(
                "Insufficient data for BorderLine".to_string(),
            ));
        }

        Ok(Self {
            line_type: reader.read_u8()?,
            thickness: reader.read_u8()?,
            color: reader.read_u32()?,
        })
    }
}

impl FillInfo {
    fn read(reader: &mut StreamReader) -> Result<Self> {
        if reader.remaining() < 16 {
            return Err(crate::error::HwpError::ParseError(
                "Insufficient data for FillInfo".to_string(),
            ));
        }

        let fill_type = reader.read_u32()?;
        let back_color = reader.read_u32()?;
        let pattern_color = reader.read_u32()?;
        let pattern_type = reader.read_u32()?;

        // Read optional image info
        let image_info = if (fill_type & 0x04) != 0 && reader.remaining() >= 6 {
            Some(ImageInfo {
                brightness: reader.read_u8()?,
                contrast: reader.read_u8()?,
                effect: reader.read_u8()?,
                bin_data_id: reader.read_u16()?,
            })
        } else {
            None
        };

        // Read optional gradient info
        let gradient_info = if (fill_type & 0x08) != 0 && reader.remaining() >= 16 {
            Some(GradientInfo {
                gradient_type: reader.read_u8()?,
                start_color: reader.read_u32()?,
                end_color: reader.read_u32()?,
                angle: reader.read_u16()?,
                center_x: reader.read_u16()?,
                center_y: reader.read_u16()?,
                blur_degree: reader.read_u16()?,
            })
        } else {
            None
        };

        Ok(Self {
            fill_type,
            back_color,
            pattern_color,
            pattern_type,
            image_info,
            gradient_info,
        })
    }
}
