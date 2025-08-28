use crate::error::{HwpError, Result};
use crate::parser::record::Record;

/// 하이퍼링크 유형
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HyperlinkType {
    /// URL 링크 (http, https, ftp 등)
    Url = 0,
    /// 이메일 주소
    Email = 1,
    /// 파일 경로
    File = 2,
    /// 문서 내 책갈피
    Bookmark = 3,
    /// 다른 문서의 책갈피
    ExternalBookmark = 4,
}

/// 하이퍼링크 표시 방식
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HyperlinkDisplay {
    /// 텍스트만 표시
    TextOnly = 0,
    /// URL만 표시
    UrlOnly = 1,
    /// 텍스트와 URL 모두 표시
    Both = 2,
}

/// 하이퍼링크 정보
#[derive(Debug, Clone)]
pub struct Hyperlink {
    /// 하이퍼링크 유형
    pub hyperlink_type: HyperlinkType,
    /// 표시할 텍스트
    pub display_text: String,
    /// 실제 링크 URL 또는 경로
    pub target_url: String,
    /// 툴팁 텍스트
    pub tooltip: Option<String>,
    /// 표시 방식
    pub display_mode: HyperlinkDisplay,
    /// 텍스트 색상 (RGB)
    pub text_color: u32,
    /// 방문한 링크 색상 (RGB)
    pub visited_color: u32,
    /// 밑줄 표시 여부
    pub underline: bool,
    /// 방문 여부
    pub visited: bool,
    /// 새 창에서 열기 여부
    pub open_in_new_window: bool,
    /// 문자 범위 시작 위치
    pub start_position: u32,
    /// 문자 범위 길이
    pub length: u32,
}

impl Default for Hyperlink {
    fn default() -> Self {
        Self {
            hyperlink_type: HyperlinkType::Url,
            display_text: String::new(),
            target_url: String::new(),
            tooltip: None,
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x0000FF, // Blue
            visited_color: 0x800080, // Purple
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: 0,
        }
    }
}

impl Hyperlink {
    /// 새로운 URL 하이퍼링크 생성
    pub fn new_url(display_text: &str, url: &str) -> Self {
        Self {
            hyperlink_type: HyperlinkType::Url,
            display_text: display_text.to_string(),
            target_url: url.to_string(),
            length: display_text.chars().count() as u32,
            ..Default::default()
        }
    }

    /// 새로운 이메일 하이퍼링크 생성
    pub fn new_email(display_text: &str, email: &str) -> Self {
        let mailto_url = if email.starts_with("mailto:") {
            email.to_string()
        } else {
            format!("mailto:{}", email)
        };
        
        Self {
            hyperlink_type: HyperlinkType::Email,
            display_text: display_text.to_string(),
            target_url: mailto_url,
            length: display_text.chars().count() as u32,
            ..Default::default()
        }
    }

    /// 새로운 파일 하이퍼링크 생성
    pub fn new_file(display_text: &str, file_path: &str) -> Self {
        Self {
            hyperlink_type: HyperlinkType::File,
            display_text: display_text.to_string(),
            target_url: file_path.to_string(),
            length: display_text.chars().count() as u32,
            ..Default::default()
        }
    }

    /// 새로운 북마크 하이퍼링크 생성
    pub fn new_bookmark(display_text: &str, bookmark_name: &str) -> Self {
        Self {
            hyperlink_type: HyperlinkType::Bookmark,
            display_text: display_text.to_string(),
            target_url: format!("#{}", bookmark_name),
            length: display_text.chars().count() as u32,
            ..Default::default()
        }
    }

    /// 시작 위치 설정
    pub fn with_position(mut self, start_position: u32) -> Self {
        self.start_position = start_position;
        self
    }

    /// 길이 설정
    pub fn with_length(mut self, length: u32) -> Self {
        self.length = length;
        self
    }

    /// 툴팁 설정
    pub fn with_tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    /// 표시 방식 설정
    pub fn with_display_mode(mut self, mode: HyperlinkDisplay) -> Self {
        self.display_mode = mode;
        self
    }

    /// 텍스트 색상 설정
    pub fn with_text_color(mut self, color: u32) -> Self {
        self.text_color = color;
        self
    }

    /// 방문한 링크 색상 설정
    pub fn with_visited_color(mut self, color: u32) -> Self {
        self.visited_color = color;
        self
    }

    /// 밑줄 표시 설정
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// 새 창에서 열기 설정
    pub fn with_new_window(mut self, new_window: bool) -> Self {
        self.open_in_new_window = new_window;
        self
    }

    /// HWP 형식으로 직렬화
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use crate::utils::encoding::string_to_utf16le;
        use std::io::{Cursor, Write};

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        // 하이퍼링크 속성
        writer.write_u8(self.hyperlink_type as u8).unwrap();
        writer.write_u8(self.display_mode as u8).unwrap();
        writer.write_u32::<LittleEndian>(self.text_color).unwrap();
        writer.write_u32::<LittleEndian>(self.visited_color).unwrap();

        // 플래그 비트 (underline, visited, new_window)
        let mut flags = 0u8;
        if self.underline { flags |= 0x01; }
        if self.visited { flags |= 0x02; }
        if self.open_in_new_window { flags |= 0x04; }
        writer.write_u8(flags).unwrap();

        // 위치 정보
        writer.write_u32::<LittleEndian>(self.start_position).unwrap();
        writer.write_u32::<LittleEndian>(self.length).unwrap();

        // 표시 텍스트
        let display_text_utf16 = string_to_utf16le(&self.display_text);
        writer.write_u16::<LittleEndian>(display_text_utf16.len() as u16 / 2).unwrap();
        writer.write_all(&display_text_utf16).unwrap();

        // 대상 URL
        let target_url_utf16 = string_to_utf16le(&self.target_url);
        writer.write_u16::<LittleEndian>(target_url_utf16.len() as u16 / 2).unwrap();
        writer.write_all(&target_url_utf16).unwrap();

        // 툴팁 (선택사항)
        if let Some(tooltip) = &self.tooltip {
            let tooltip_utf16 = string_to_utf16le(tooltip);
            writer.write_u16::<LittleEndian>(tooltip_utf16.len() as u16 / 2).unwrap();
            writer.write_all(&tooltip_utf16).unwrap();
        } else {
            writer.write_u16::<LittleEndian>(0).unwrap();
        }

        data
    }

    /// HWP 레코드에서 파싱
    pub fn from_record(record: &Record) -> Result<Self> {
        let data = &record.data;
        
        if data.len() < 70 {
            return Err(HwpError::InvalidFormat("Record too small for hyperlink".to_string()));
        }
        
        // Based on reverse engineering analysis:
        // 0x00-0x03: Control ID ('gsh ')  
        // 0x04-0x1B: Control header (24 bytes)
        // 0x1C-0x2F: Hyperlink properties (20 bytes)
        // 0x30+: Length-prefixed strings
        
        // Skip control ID + control header
        
        // Skip hyperlink properties for now and look for strings
        // From analysis, strings start around offset 0x32-0x38
        let mut offset = 48; // Jump to where strings typically start
        
        if offset + 2 >= data.len() {
            return Err(HwpError::InvalidFormat("Not enough data for hyperlink strings".to_string()));
        }
        
        // Read display text length
        let display_text_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        
        // Read display text (UTF-16)
        let mut display_text = String::new();
        if display_text_len > 0 && offset + display_text_len * 2 <= data.len() {
            let mut utf16_chars = Vec::new();
            for i in 0..display_text_len {
                let char_offset = offset + i * 2;
                if char_offset + 1 < data.len() {
                    let char_val = u16::from_le_bytes([data[char_offset], data[char_offset + 1]]);
                    if char_val == 0 {
                        break;
                    }
                    utf16_chars.push(char_val);
                }
            }
            display_text = String::from_utf16(&utf16_chars)
                .unwrap_or_default();
            offset += display_text_len * 2;
        }
        
        // Read target URL length  
        if offset + 2 >= data.len() {
            return Err(HwpError::InvalidFormat("Not enough data for URL length".to_string()));
        }
        
        let target_url_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        
        // Read target URL (UTF-16)
        let mut target_url = String::new();
        if target_url_len > 0 && offset + target_url_len * 2 <= data.len() {
            let mut utf16_chars = Vec::new();
            for i in 0..target_url_len {
                let char_offset = offset + i * 2;
                if char_offset + 1 < data.len() {
                    let char_val = u16::from_le_bytes([data[char_offset], data[char_offset + 1]]);
                    if char_val == 0 {
                        break;
                    }
                    utf16_chars.push(char_val);
                }
            }
            target_url = String::from_utf16(&utf16_chars)
                .unwrap_or_default();
            offset += target_url_len * 2;
        }

        // Determine hyperlink type from URL
        let hyperlink_type = if target_url.starts_with("mailto:") {
            HyperlinkType::Email
        } else if target_url.starts_with("http://") || target_url.starts_with("https://") {
            HyperlinkType::Url
        } else if target_url.starts_with("#") {
            HyperlinkType::Bookmark
        } else if target_url.contains("\\") || target_url.starts_with("./") || target_url.starts_with("C:") {
            HyperlinkType::File
        } else {
            HyperlinkType::Url
        };

        // Try to read tooltip if there's remaining data
        let tooltip = if offset + 2 < data.len() {
            let tooltip_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            if tooltip_len > 0 && offset + 2 + tooltip_len * 2 <= data.len() {
                let mut utf16_chars = Vec::new();
                for i in 0..tooltip_len {
                    let char_offset = offset + 2 + i * 2;
                    if char_offset + 1 < data.len() {
                        let char_val = u16::from_le_bytes([data[char_offset], data[char_offset + 1]]);
                        if char_val == 0 {
                            break;
                        }
                        utf16_chars.push(char_val);
                    }
                }
                String::from_utf16(&utf16_chars).ok()
            } else {
                None
            }
        } else {
            None
        };

        let text_length = display_text.chars().count() as u32;
        
        Ok(Self {
            hyperlink_type,
            display_text,
            target_url,
            tooltip,
            display_mode: HyperlinkDisplay::TextOnly,
            text_color: 0x0000FF, // Default blue
            visited_color: 0x800080, // Default purple
            underline: true,
            visited: false,
            open_in_new_window: false,
            start_position: 0,
            length: text_length,
        })
    }
}

/// 미리 정의된 하이퍼링크 스타일들
impl Hyperlink {
    /// 기본 웹 링크
    pub fn web_link(text: &str, url: &str) -> Self {
        Self::new_url(text, url)
            .with_text_color(0x0000FF) // Blue
            .with_underline(true)
    }

    /// 이메일 링크
    pub fn email_link(text: &str, email: &str) -> Self {
        Self::new_email(text, email)
            .with_text_color(0x0000FF) // Blue
            .with_underline(true)
    }

    /// 파일 링크
    pub fn file_link(text: &str, file_path: &str) -> Self {
        Self::new_file(text, file_path)
            .with_text_color(0x008000) // Green
            .with_underline(true)
    }

    /// 문서 내 링크
    pub fn internal_link(text: &str, bookmark: &str) -> Self {
        Self::new_bookmark(text, bookmark)
            .with_text_color(0x800080) // Purple
            .with_underline(true)
    }

    /// 밑줄 없는 링크
    pub fn plain_link(text: &str, url: &str) -> Self {
        Self::new_url(text, url)
            .with_text_color(0x0000FF) // Blue
            .with_underline(false)
    }

    /// 새 창에서 열리는 링크
    pub fn external_link(text: &str, url: &str) -> Self {
        Self::new_url(text, url)
            .with_text_color(0x0000FF) // Blue
            .with_underline(true)
            .with_new_window(true)
    }
}