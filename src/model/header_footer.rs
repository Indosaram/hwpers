use crate::error::{HwpError, Result};
use crate::parser::record::Record;

/// Header/Footer 유형
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderFooterType {
    /// 머리글
    Header = 0,
    /// 바닥글
    Footer = 1,
}

/// Header/Footer 적용 페이지 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageApplyType {
    /// 모든 페이지
    All = 0,
    /// 첫 페이지만
    FirstPage = 1,
    /// 짝수 페이지
    EvenPages = 2,
    /// 홀수 페이지
    OddPages = 3,
}

/// Header 또는 Footer 정보
#[derive(Debug, Clone)]
pub struct HeaderFooter {
    /// Header 또는 Footer 타입
    pub header_footer_type: HeaderFooterType,
    /// 적용할 페이지 타입
    pub apply_type: PageApplyType,
    /// 텍스트 내용
    pub text: String,
    /// 텍스트 스타일 ID
    pub char_shape_id: u16,
    /// 단락 스타일 ID
    pub para_shape_id: u16,
    /// 정렬 방식 (0=왼쪽, 1=가운데, 2=오른쪽)
    pub alignment: u8,
    /// 페이지 번호 포함 여부
    pub include_page_number: bool,
    /// 페이지 번호 형식 (1=숫자, 2=로마자 소문자, 3=로마자 대문자, 4=영문 소문자, 5=영문 대문자)
    pub page_number_format: u8,
    /// 높이 (HWP 단위)
    pub height: u32,
    /// 여백 (HWP 단위)
    pub margin: u32,
}

impl Default for HeaderFooter {
    fn default() -> Self {
        Self {
            header_footer_type: HeaderFooterType::Header,
            apply_type: PageApplyType::All,
            text: String::new(),
            char_shape_id: 0,
            para_shape_id: 0,
            alignment: 0, // 왼쪽 정렬
            include_page_number: false,
            page_number_format: 1, // 숫자
            height: 1000, // 10mm
            margin: 500,  // 5mm
        }
    }
}

impl HeaderFooter {
    /// 새로운 Header 생성
    pub fn new_header(text: &str) -> Self {
        Self {
            header_footer_type: HeaderFooterType::Header,
            text: text.to_string(),
            ..Default::default()
        }
    }
    
    /// 새로운 Footer 생성
    pub fn new_footer(text: &str) -> Self {
        Self {
            header_footer_type: HeaderFooterType::Footer,
            text: text.to_string(),
            ..Default::default()
        }
    }
    
    /// 페이지 번호가 포함된 Header/Footer 생성
    pub fn with_page_number(mut self, format: PageNumberFormat) -> Self {
        self.include_page_number = true;
        self.page_number_format = format as u8;
        self
    }
    
    /// 정렬 방식 설정
    pub fn with_alignment(mut self, alignment: HeaderFooterAlignment) -> Self {
        self.alignment = alignment as u8;
        self
    }
    
    /// 적용 페이지 타입 설정
    pub fn with_apply_type(mut self, apply_type: PageApplyType) -> Self {
        self.apply_type = apply_type;
        self
    }
    
    /// 높이 설정 (mm 단위)
    pub fn with_height_mm(mut self, height_mm: u32) -> Self {
        self.height = height_mm * 100; // mm to HWP units
        self
    }
    
    /// 여백 설정 (mm 단위)
    pub fn with_margin_mm(mut self, margin_mm: u32) -> Self {
        self.margin = margin_mm * 100; // mm to HWP units
        self
    }
    
    /// HWP 형식으로 직렬화
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use crate::utils::encoding::string_to_utf16le;
        use std::io::{Cursor, Write};
        
        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        
        // Header/Footer 타입과 속성
        writer.write_u8(self.header_footer_type as u8).unwrap();
        writer.write_u8(self.apply_type as u8).unwrap();
        writer.write_u8(self.alignment).unwrap();
        writer.write_u8(if self.include_page_number { 1 } else { 0 }).unwrap();
        writer.write_u8(self.page_number_format).unwrap();
        
        // 높이와 여백
        writer.write_u32::<LittleEndian>(self.height).unwrap();
        writer.write_u32::<LittleEndian>(self.margin).unwrap();
        
        // 스타일 ID들
        writer.write_u16::<LittleEndian>(self.char_shape_id).unwrap();
        writer.write_u16::<LittleEndian>(self.para_shape_id).unwrap();
        
        // 텍스트 내용
        let text_utf16 = string_to_utf16le(&self.text);
        writer.write_u16::<LittleEndian>(text_utf16.len() as u16 / 2).unwrap();
        writer.write_all(&text_utf16).unwrap();
        
        data
    }
    
    /// HWP 레코드에서 파싱
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        // Based on actual file analysis, HeaderFooter is a 40-byte structure with 10 u32 fields
        if record.data.len() < 40 {
            return Err(HwpError::InvalidFormat("HeaderFooter record too short".to_string()));
        }
        
        // Read 10 u32 fields
        let _field1 = reader.read_u32()?; // 0x0000E888
        let _field2 = reader.read_u32()?; // 0x000148DA
        let height = reader.read_u32()?;  // Height (appears twice)
        let _height2 = reader.read_u32()?; // Same as height
        let left_margin = reader.read_u32()?; // Left margin
        let _top_margin = reader.read_u32()?;  // Top margin
        let _right_margin = reader.read_u32()?; // Right margin
        let _bottom_margin = reader.read_u32()?; // Bottom margin
        let _field9 = reader.read_u32()?;  // Reserved (0)
        let _field10 = reader.read_u32()?; // Reserved (0)
        
        // Default values since the actual structure doesn't contain text or type info
        // The text content is stored separately in the document
        Ok(Self {
            header_footer_type: HeaderFooterType::Header,
            apply_type: PageApplyType::All,
            text: String::new(), // Text is stored elsewhere
            char_shape_id: 0,
            para_shape_id: 0,
            alignment: 0,
            include_page_number: false,
            page_number_format: 1,
            height,
            margin: left_margin,
        })
    }
}

/// 페이지 번호 형식
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageNumberFormat {
    /// 숫자 (1, 2, 3, ...)
    Numeric = 1,
    /// 로마자 소문자 (i, ii, iii, ...)
    RomanLower = 2,
    /// 로마자 대문자 (I, II, III, ...)
    RomanUpper = 3,
    /// 영문 소문자 (a, b, c, ...)
    AlphaLower = 4,
    /// 영문 대문자 (A, B, C, ...)
    AlphaUpper = 5,
}

/// Header/Footer 정렬 방식
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderFooterAlignment {
    /// 왼쪽 정렬
    Left = 0,
    /// 가운데 정렬
    Center = 1,
    /// 오른쪽 정렬
    Right = 2,
}

/// Header/Footer 컬렉션을 관리하는 구조체
#[derive(Debug, Clone, Default)]
pub struct HeaderFooterCollection {
    /// Header/Footer 목록
    pub items: Vec<HeaderFooter>,
}

impl HeaderFooterCollection {
    /// 새로운 컬렉션 생성
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
    
    /// Header 추가
    pub fn add_header(&mut self, header: HeaderFooter) {
        self.items.push(header);
    }
    
    /// Footer 추가
    pub fn add_footer(&mut self, footer: HeaderFooter) {
        self.items.push(footer);
    }
    
    /// 특정 타입의 Header/Footer 찾기
    pub fn find_by_type(&self, header_footer_type: HeaderFooterType, apply_type: PageApplyType) -> Option<&HeaderFooter> {
        self.items.iter().find(|item| {
            item.header_footer_type == header_footer_type && item.apply_type == apply_type
        })
    }
    
    /// 모든 Header들 가져오기
    pub fn headers(&self) -> Vec<&HeaderFooter> {
        self.items.iter().filter(|item| item.header_footer_type == HeaderFooterType::Header).collect()
    }
    
    /// 모든 Footer들 가져오기
    pub fn footers(&self) -> Vec<&HeaderFooter> {
        self.items.iter().filter(|item| item.header_footer_type == HeaderFooterType::Footer).collect()
    }
}