use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct CtrlHeader {
    pub ctrl_id: u32,
    pub properties: u32,
    pub instance_id: u32,
}

impl CtrlHeader {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        if reader.remaining() < 4 {
            return Err(crate::error::HwpError::ParseError(
                format!("CtrlHeader record too small: {} bytes", reader.remaining())
            ));
        }
        
        let ctrl_id = reader.read_u32()?;
        let properties = if reader.remaining() >= 4 { reader.read_u32()? } else { 0 };
        let instance_id = if reader.remaining() >= 4 { reader.read_u32()? } else { 0 };
        
        Ok(Self {
            ctrl_id,
            properties,
            instance_id,
        })
    }
    
    pub fn get_control_type(&self) -> ControlType {
        ControlType::from_ctrl_id(self.ctrl_id)
    }
    
    pub fn is_inline(&self) -> bool {
        (self.properties & 0x01) != 0
    }
    
    pub fn affects_line_pacing(&self) -> bool {
        (self.properties & 0x02) != 0
    }
    
    pub fn is_word_break_allowed(&self) -> bool {
        (self.properties & 0x04) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    Table,
    Gso,  // Drawing object
    Equation,
    SectionDefinition,
    ColumnDefinition,
    HeaderFooter,
    Footnote,
    Endnote,
    AutoNumber,
    NewNumber,
    PageHide,
    PageNumberPosition,
    IndexMark,
    BookMark,
    OverlappingLetter,
    HiddenComment,
    Field,
    Unknown,
}

impl ControlType {
    pub fn from_ctrl_id(ctrl_id: u32) -> Self {
        match ctrl_id {
            0x5442 => Self::Table,          // 'TB'
            0x6F73 => Self::Gso,             // 'so'
            0x7165 => Self::Equation,        // 'eq'
            0x636573 => Self::SectionDefinition, // 'sec'
            0x6C6F63 => Self::ColumnDefinition,  // 'col'
            0x646E65 => Self::Endnote,       // 'end'
            0x746F66 => Self::Footnote,      // 'fot'
            0x676170 => Self::PageNumberPosition, // 'pag'
            0x6B6D62 => Self::BookMark,      // 'bmk'
            0x6F6961 => Self::AutoNumber,    // 'aio'
            0x6E756E => Self::NewNumber,     // 'nun'
            0x65646968 => Self::PageHide,   // 'hide'
            0x74636573 => Self::OverlappingLetter, // 'tcmt'
            0x6B6469 => Self::IndexMark,     // 'idx'
            0x646C66 => Self::Field,         // 'fld'
            _ => Self::Unknown,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Table => "Table",
            Self::Gso => "DrawingObject",
            Self::Equation => "Equation",
            Self::SectionDefinition => "SectionDefinition",
            Self::ColumnDefinition => "ColumnDefinition",
            Self::HeaderFooter => "HeaderFooter",
            Self::Footnote => "Footnote",
            Self::Endnote => "Endnote",
            Self::AutoNumber => "AutoNumber",
            Self::NewNumber => "NewNumber",
            Self::PageHide => "PageHide",
            Self::PageNumberPosition => "PageNumberPosition",
            Self::IndexMark => "IndexMark",
            Self::BookMark => "BookMark",
            Self::OverlappingLetter => "OverlappingLetter",
            Self::HiddenComment => "HiddenComment",
            Self::Field => "Field",
            Self::Unknown => "Unknown",
        }
    }
}