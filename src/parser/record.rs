use crate::error::Result;
use crate::reader::StreamReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordHeader {
    pub tag_id: u16,
    pub level: u8,
    pub size: u32,
}

impl RecordHeader {
    pub fn parse(reader: &mut StreamReader) -> Result<Self> {
        // Read single 32-bit header
        let header = reader.read_u32()?;

        // Extract fields from packed header
        // Bits 0-9: tag_id (10 bits)
        // Bits 10-19: level (10 bits)
        // Bits 20-31: size (12 bits)
        let tag_id = (header & 0x3FF) as u16;
        let level = ((header >> 10) & 0x3FF) as u8;
        let mut size = header >> 20;

        // If size is 0xFFF (4095), read extended size
        if size == 0xFFF {
            size = reader.read_u32()?;
        }

        Ok(Self {
            tag_id,
            level,
            size,
        })
    }
}

#[derive(Debug)]
pub struct Record {
    pub header: RecordHeader,
    pub data: Vec<u8>,
}

impl Record {
    pub fn parse(reader: &mut StreamReader) -> Result<Self> {
        if reader.remaining() < 4 {
            return Err(crate::error::HwpError::ParseError(
                "Not enough data for record header".to_string(),
            ));
        }

        let header = RecordHeader::parse(reader)?;

        // Size field contains the data size (without header)
        let data_size = header.size as usize;

        if data_size > reader.remaining() {
            return Err(crate::error::HwpError::ParseError(format!(
                "Record size {} exceeds remaining data {}",
                data_size,
                reader.remaining()
            )));
        }

        // Read data bytes
        let data = reader.read_bytes(data_size)?;

        Ok(Self { header, data })
    }

    pub fn tag_id(&self) -> u16 {
        self.header.tag_id
    }

    pub fn data_reader(&self) -> StreamReader {
        StreamReader::new(self.data.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum HwpTag {
    // Document Info
    DocumentProperties = 0x10,
    IdMappings = 0x11,
    BinData = 0x12,
    FaceName = 0x13,
    BorderFill = 0x14,
    CharShape = 0x15,
    TabDef = 0x16,
    Numbering = 0x17,
    Bullet = 0x18,
    ParaShape = 0x19,
    Style = 0x1A,
    DocData = 0x1B,
    DistributeDocData = 0x1C,

    // BodyText - Section Definition
    SectionDefine = 0x42,
    ColumnDefine = 0x43,
    TableControl = 0x44,
    SheetControl = 0x45,
    LineInfo = 0x47,
    HiddenComment = 0x48,
    HeaderFooter = 0x49,
    Footnote = 0x4A,
    AutoNumber = 0x4B,
    NewNumber = 0x4C,
    PageHide = 0x4D,
    PageOddEvenAdjust = 0x4E,
    PageNumberPosition = 0x4F,
    ParaHeader = 0x50,
    ParaText = 0x51,
    ParaCharShape = 0x52,
    ParaLineSeg = 0x53,
    ParaRangeTag = 0x54,
    CtrlHeader = 0x55,
    ListHeader = 0x56,
    PageDef = 0x57,
    FootnoteShape = 0x58,
    PageBorderFill = 0x59,
    ShapeObject = 0x5A,
    Table = 0x5B,
    ShapeComponent = 0x5C,
    ShapeComponentLine = 0x5D,
    ShapeComponentRectangle = 0x5E,
    ShapeComponentEllipse = 0x5F,
    ShapeComponentArc = 0x60,
    ShapeComponentPolygon = 0x61,
    ShapeComponentCurve = 0x62,
    ShapeComponentOle = 0x63,
    ShapeComponentPicture = 0x64,
    ShapeComponentContainer = 0x65,

    // Embedded controls
    EqEdit = 0x70,
    Reserved1 = 0x71,
    Reserved2 = 0x72,
    FormObject = 0x73,
    MemoShape = 0x74,
    MemoList = 0x75,
    ChartData = 0x76,
}

impl HwpTag {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x10 => Some(Self::DocumentProperties),
            0x11 => Some(Self::IdMappings),
            0x12 => Some(Self::BinData),
            0x13 => Some(Self::FaceName),
            0x14 => Some(Self::BorderFill),
            0x15 => Some(Self::CharShape),
            0x16 => Some(Self::TabDef),
            0x17 => Some(Self::Numbering),
            0x18 => Some(Self::Bullet),
            0x19 => Some(Self::ParaShape),
            0x1A => Some(Self::Style),
            0x1B => Some(Self::DocData),
            0x1C => Some(Self::DistributeDocData),
            0x42 => Some(Self::SectionDefine),
            0x43 => Some(Self::ColumnDefine),
            0x44 => Some(Self::TableControl),
            0x45 => Some(Self::SheetControl),
            0x47 => Some(Self::LineInfo),
            0x48 => Some(Self::HiddenComment),
            0x49 => Some(Self::HeaderFooter),
            0x4A => Some(Self::Footnote),
            0x4B => Some(Self::AutoNumber),
            0x4C => Some(Self::NewNumber),
            0x4D => Some(Self::PageHide),
            0x4E => Some(Self::PageOddEvenAdjust),
            0x4F => Some(Self::PageNumberPosition),
            0x50 => Some(Self::ParaHeader),
            0x51 => Some(Self::ParaText),
            0x52 => Some(Self::ParaCharShape),
            0x53 => Some(Self::ParaLineSeg),
            0x54 => Some(Self::ParaRangeTag),
            0x55 => Some(Self::CtrlHeader),
            0x56 => Some(Self::ListHeader),
            0x57 => Some(Self::PageDef),
            0x58 => Some(Self::FootnoteShape),
            0x59 => Some(Self::PageBorderFill),
            0x5A => Some(Self::ShapeObject),
            0x5B => Some(Self::Table),
            0x5C => Some(Self::ShapeComponent),
            0x5D => Some(Self::ShapeComponentLine),
            0x5E => Some(Self::ShapeComponentRectangle),
            0x5F => Some(Self::ShapeComponentEllipse),
            0x60 => Some(Self::ShapeComponentArc),
            0x61 => Some(Self::ShapeComponentPolygon),
            0x62 => Some(Self::ShapeComponentCurve),
            0x63 => Some(Self::ShapeComponentOle),
            0x64 => Some(Self::ShapeComponentPicture),
            0x65 => Some(Self::ShapeComponentContainer),
            0x70 => Some(Self::EqEdit),
            0x71 => Some(Self::Reserved1),
            0x72 => Some(Self::Reserved2),
            0x73 => Some(Self::FormObject),
            0x74 => Some(Self::MemoShape),
            0x75 => Some(Self::MemoList),
            0x76 => Some(Self::ChartData),
            _ => None,
        }
    }
}
