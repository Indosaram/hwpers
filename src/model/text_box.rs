use crate::error::{HwpError, Result};
use crate::parser::record::Record;

/// 텍스트 박스 정렬 방식
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBoxAlignment {
    /// 인라인 (글자처럼 취급)
    Inline = 0,
    /// 왼쪽
    Left = 1,
    /// 가운데
    Center = 2,
    /// 오른쪽
    Right = 3,
    /// 자유 위치
    Absolute = 4,
}

/// 텍스트 박스 테두리 스타일
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBoxBorderStyle {
    /// 테두리 없음
    None = 0,
    /// 실선
    Solid = 1,
    /// 점선
    Dotted = 2,
    /// 파선
    Dashed = 3,
    /// 이중선
    Double = 4,
}

/// 텍스트 박스 배경 채우기 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBoxFillType {
    /// 채우기 없음
    None = 0,
    /// 단색
    Solid = 1,
    /// 그라데이션
    Gradient = 2,
    /// 이미지
    Image = 3,
}

/// 텍스트 박스 정보
#[derive(Debug, Clone)]
pub struct TextBox {
    /// 텍스트 내용
    pub text: String,
    /// 위치 X (HWP 단위)
    pub x: i32,
    /// 위치 Y (HWP 단위)
    pub y: i32,
    /// 너비 (HWP 단위)
    pub width: u32,
    /// 높이 (HWP 단위)
    pub height: u32,
    /// 정렬 방식
    pub alignment: TextBoxAlignment,
    /// 테두리 스타일
    pub border_style: TextBoxBorderStyle,
    /// 테두리 두께
    pub border_width: u8,
    /// 테두리 색상 (RGB)
    pub border_color: u32,
    /// 배경 채우기 타입
    pub fill_type: TextBoxFillType,
    /// 배경 색상 (RGB)
    pub background_color: u32,
    /// 텍스트 여백 (내부 패딩)
    pub padding: u16,
    /// 텍스트 스타일 ID
    pub char_shape_id: u16,
    /// 단락 스타일 ID
    pub para_shape_id: u16,
    /// Z-order (쌓기 순서)
    pub z_order: u16,
    /// 투명도 (0-255, 0=투명, 255=불투명)
    pub opacity: u8,
    /// 회전 각도 (도 단위)
    pub rotation: i16,
}

impl Default for TextBox {
    fn default() -> Self {
        Self {
            text: String::new(),
            x: 0,
            y: 0,
            width: 5000,  // 50mm
            height: 2000, // 20mm
            alignment: TextBoxAlignment::Inline,
            border_style: TextBoxBorderStyle::Solid,
            border_width: 1,
            border_color: 0x000000, // Black
            fill_type: TextBoxFillType::None,
            background_color: 0xFFFFFF, // White
            padding: 100, // 1mm
            char_shape_id: 0,
            para_shape_id: 0,
            z_order: 0,
            opacity: 255, // Fully opaque
            rotation: 0,
        }
    }
}

impl TextBox {
    /// 새로운 텍스트 박스 생성
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }
    
    /// 위치 설정 (mm 단위)
    pub fn with_position_mm(mut self, x_mm: i32, y_mm: i32) -> Self {
        self.x = x_mm * 100; // mm to HWP units
        self.y = y_mm * 100;
        self
    }
    
    /// 크기 설정 (mm 단위)
    pub fn with_size_mm(mut self, width_mm: u32, height_mm: u32) -> Self {
        self.width = width_mm * 100; // mm to HWP units
        self.height = height_mm * 100;
        self
    }
    
    /// 정렬 방식 설정
    pub fn with_alignment(mut self, alignment: TextBoxAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// 테두리 설정
    pub fn with_border(mut self, style: TextBoxBorderStyle, width: u8, color: u32) -> Self {
        self.border_style = style;
        self.border_width = width;
        self.border_color = color;
        self
    }
    
    /// 배경 설정
    pub fn with_background(mut self, color: u32) -> Self {
        self.fill_type = TextBoxFillType::Solid;
        self.background_color = color;
        self
    }
    
    /// 투명 배경 설정
    pub fn with_transparent_background(mut self) -> Self {
        self.fill_type = TextBoxFillType::None;
        self
    }
    
    /// 패딩 설정 (mm 단위)
    pub fn with_padding_mm(mut self, padding_mm: u16) -> Self {
        self.padding = padding_mm * 100; // mm to HWP units
        self
    }
    
    /// 투명도 설정 (0-255)
    pub fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }
    
    /// 회전 설정 (도 단위)
    pub fn with_rotation(mut self, degrees: i16) -> Self {
        self.rotation = degrees;
        self
    }
    
    /// Z-order 설정
    pub fn with_z_order(mut self, z_order: u16) -> Self {
        self.z_order = z_order;
        self
    }
    
    /// HWP 형식으로 직렬화
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use crate::utils::encoding::string_to_utf16le;
        use std::io::{Cursor, Write};
        
        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        
        // 기본 속성
        writer.write_i32::<LittleEndian>(self.x).unwrap();
        writer.write_i32::<LittleEndian>(self.y).unwrap();
        writer.write_u32::<LittleEndian>(self.width).unwrap();
        writer.write_u32::<LittleEndian>(self.height).unwrap();
        
        // 정렬과 스타일
        writer.write_u8(self.alignment as u8).unwrap();
        writer.write_u8(self.border_style as u8).unwrap();
        writer.write_u8(self.border_width).unwrap();
        writer.write_u32::<LittleEndian>(self.border_color).unwrap();
        
        // 배경
        writer.write_u8(self.fill_type as u8).unwrap();
        writer.write_u32::<LittleEndian>(self.background_color).unwrap();
        
        // 기타 속성
        writer.write_u16::<LittleEndian>(self.padding).unwrap();
        writer.write_u16::<LittleEndian>(self.char_shape_id).unwrap();
        writer.write_u16::<LittleEndian>(self.para_shape_id).unwrap();
        writer.write_u16::<LittleEndian>(self.z_order).unwrap();
        writer.write_u8(self.opacity).unwrap();
        writer.write_i16::<LittleEndian>(self.rotation).unwrap();
        
        // 텍스트 내용
        let text_utf16 = string_to_utf16le(&self.text);
        writer.write_u16::<LittleEndian>(text_utf16.len() as u16 / 2).unwrap();
        writer.write_all(&text_utf16).unwrap();
        
        data
    }
    
    /// HWP 레코드에서 파싱
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        
        let alignment = match reader.read_u8()? {
            0 => TextBoxAlignment::Inline,
            1 => TextBoxAlignment::Left,
            2 => TextBoxAlignment::Center,
            3 => TextBoxAlignment::Right,
            4 => TextBoxAlignment::Absolute,
            _ => TextBoxAlignment::Inline,
        };
        
        let border_style = match reader.read_u8()? {
            0 => TextBoxBorderStyle::None,
            1 => TextBoxBorderStyle::Solid,
            2 => TextBoxBorderStyle::Dotted,
            3 => TextBoxBorderStyle::Dashed,
            4 => TextBoxBorderStyle::Double,
            _ => TextBoxBorderStyle::Solid,
        };
        
        let border_width = reader.read_u8()?;
        let border_color = reader.read_u32()?;
        
        let fill_type = match reader.read_u8()? {
            0 => TextBoxFillType::None,
            1 => TextBoxFillType::Solid,
            2 => TextBoxFillType::Gradient,
            3 => TextBoxFillType::Image,
            _ => TextBoxFillType::None,
        };
        
        let background_color = reader.read_u32()?;
        let padding = reader.read_u16()?;
        let char_shape_id = reader.read_u16()?;
        let para_shape_id = reader.read_u16()?;
        let z_order = reader.read_u16()?;
        let opacity = reader.read_u8()?;
        let rotation = reader.read_u16()? as i16;
        
        let text_len = reader.read_u16()? as usize;
        let text_bytes = reader.read_bytes(text_len * 2)?;
        
        // Convert UTF-16LE bytes to string
        let mut utf16_chars = Vec::new();
        for chunk in text_bytes.chunks_exact(2) {
            let char_value = u16::from_le_bytes([chunk[0], chunk[1]]);
            utf16_chars.push(char_value);
        }
        let text = String::from_utf16(&utf16_chars)
            .map_err(|_| HwpError::InvalidFormat("Invalid UTF-16 text".to_string()))?;
        
        Ok(Self {
            text,
            x,
            y,
            width,
            height,
            alignment,
            border_style,
            border_width,
            border_color,
            fill_type,
            background_color,
            padding,
            char_shape_id,
            para_shape_id,
            z_order,
            opacity,
            rotation,
        })
    }
}

/// 미리 정의된 텍스트 박스 스타일들
impl TextBox {
    /// 기본 텍스트 박스
    pub fn basic(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Solid, 1, 0x000000)
            .with_background(0xFFFFFF)
    }
    
    /// 강조 텍스트 박스 (노란 배경)
    pub fn highlight(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Solid, 2, 0x000000)
            .with_background(0xFFFF00) // Yellow
    }
    
    /// 경고 텍스트 박스 (빨간 테두리)
    pub fn warning(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Solid, 2, 0xFF0000) // Red border
            .with_background(0xFFE0E0) // Light red background
    }
    
    /// 정보 텍스트 박스 (파란 배경)
    pub fn info(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Solid, 1, 0x0000FF) // Blue border
            .with_background(0xE0E0FF) // Light blue background
    }
    
    /// 투명 텍스트 박스 (테두리만)
    pub fn transparent(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Dashed, 1, 0x808080) // Gray dashed border
            .with_transparent_background()
    }
    
    /// 말풍선 스타일
    pub fn bubble(text: &str) -> Self {
        Self::new(text)
            .with_border(TextBoxBorderStyle::Solid, 2, 0x000000)
            .with_background(0xF0F0F0) // Light gray
            .with_padding_mm(5)
    }
}