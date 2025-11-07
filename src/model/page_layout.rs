/// 페이지 방향
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageOrientation {
    /// 세로 (Portrait)
    Portrait = 0,
    /// 가로 (Landscape)
    Landscape = 1,
}

/// 표준 용지 크기
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperSize {
    /// A4 (210 × 297 mm)
    A4,
    /// A3 (297 × 420 mm)
    A3,
    /// A5 (148 × 210 mm)
    A5,
    /// Letter (8.5 × 11 inch)
    Letter,
    /// Legal (8.5 × 14 inch)
    Legal,
    /// Tabloid (11 × 17 inch)
    Tabloid,
    /// B4 (250 × 353 mm)
    B4,
    /// B5 (176 × 250 mm)
    B5,
    /// 사용자 정의
    Custom,
}

/// 여백 단위
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginUnit {
    /// 밀리미터
    Millimeters,
    /// 인치
    Inches,
    /// 포인트 (1 point = 1/72 inch)
    Points,
    /// HWP 내부 단위 (1 unit = 1/7200 inch)
    HwpUnits,
}

/// 페이지 여백 설정
#[derive(Debug, Clone)]
pub struct PageMargins {
    /// 왼쪽 여백 (HWP 단위)
    pub left: u32,
    /// 오른쪽 여백 (HWP 단위)
    pub right: u32,
    /// 위쪽 여백 (HWP 단위)
    pub top: u32,
    /// 아래쪽 여백 (HWP 단위)
    pub bottom: u32,
    /// 머리글 여백 (HWP 단위)
    pub header: u32,
    /// 바닥글 여백 (HWP 단위)
    pub footer: u32,
    /// 제본 여백 (HWP 단위)
    pub gutter: u32,
    /// 미러 여백 (홀수/짝수 페이지 여백 교체)
    pub mirror_margins: bool,
}

/// 페이지 레이아웃 설정
#[derive(Debug, Clone)]
pub struct PageLayout {
    /// 용지 크기
    pub paper_size: PaperSize,
    /// 페이지 방향
    pub orientation: PageOrientation,
    /// 페이지 너비 (HWP 단위)
    pub width: u32,
    /// 페이지 높이 (HWP 단위)
    pub height: u32,
    /// 여백 설정
    pub margins: PageMargins,
    /// 다단 설정
    pub columns: u16,
    /// 단 사이 간격 (HWP 단위)
    pub column_spacing: u32,
    /// 단 구분선 표시
    pub column_line: bool,
    /// 페이지 테두리 사용
    pub page_border: bool,
    /// 페이지 배경색 (RGB)
    pub background_color: Option<u32>,
    /// 시작 페이지 번호
    pub start_page_number: u16,
    /// 페이지 번호 형식
    pub page_number_format: crate::model::header_footer::PageNumberFormat,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            left: 8504,   // 30mm
            right: 8504,  // 30mm
            top: 5669,    // 20mm
            bottom: 4252, // 15mm
            header: 4252, // 15mm
            footer: 4252, // 15mm
            gutter: 0,
            mirror_margins: false,
        }
    }
}

impl Default for PageLayout {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::A4,
            orientation: PageOrientation::Portrait,
            width: 59528,  // 210mm in HWP units
            height: 84188, // 297mm in HWP units
            margins: PageMargins::default(),
            columns: 1,
            column_spacing: 1417, // 5mm
            column_line: false,
            page_border: false,
            background_color: None,
            start_page_number: 1,
            page_number_format: crate::model::header_footer::PageNumberFormat::Numeric,
        }
    }
}

impl PaperSize {
    /// 용지 크기의 기본 치수를 HWP 단위로 반환 (세로 방향 기준)
    pub fn dimensions_hwp_units(&self) -> (u32, u32) {
        match self {
            PaperSize::A4 => (59528, 84188),       // 210 × 297 mm
            PaperSize::A3 => (84188, 119055),      // 297 × 420 mm
            PaperSize::A5 => (41929, 59528),       // 148 × 210 mm
            PaperSize::Letter => (61200, 79200),   // 8.5 × 11 inch
            PaperSize::Legal => (61200, 100800),   // 8.5 × 14 inch
            PaperSize::Tabloid => (79200, 122400), // 11 × 17 inch
            PaperSize::B4 => (70866, 100063),      // 250 × 353 mm
            PaperSize::B5 => (49606, 70866),       // 176 × 250 mm
            PaperSize::Custom => (59528, 84188),   // Default to A4
        }
    }

    /// 용지 크기 이름 반환
    pub fn name(&self) -> &'static str {
        match self {
            PaperSize::A4 => "A4",
            PaperSize::A3 => "A3",
            PaperSize::A5 => "A5",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
            PaperSize::Tabloid => "Tabloid",
            PaperSize::B4 => "B4",
            PaperSize::B5 => "B5",
            PaperSize::Custom => "Custom",
        }
    }
}

impl PageMargins {
    /// 새로운 여백 설정 생성 (밀리미터 단위)
    pub fn new_mm(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left: mm_to_hwp_units(left),
            right: mm_to_hwp_units(right),
            top: mm_to_hwp_units(top),
            bottom: mm_to_hwp_units(bottom),
            header: mm_to_hwp_units(15.0), // Default 15mm
            footer: mm_to_hwp_units(15.0), // Default 15mm
            gutter: 0,
            mirror_margins: false,
        }
    }

    /// 새로운 여백 설정 생성 (인치 단위)
    pub fn new_inches(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left: inches_to_hwp_units(left),
            right: inches_to_hwp_units(right),
            top: inches_to_hwp_units(top),
            bottom: inches_to_hwp_units(bottom),
            header: inches_to_hwp_units(0.6), // Default 0.6 inch
            footer: inches_to_hwp_units(0.6), // Default 0.6 inch
            gutter: 0,
            mirror_margins: false,
        }
    }

    /// 머리글/바닥글 여백 설정 (밀리미터 단위)
    pub fn with_header_footer_mm(mut self, header: f32, footer: f32) -> Self {
        self.header = mm_to_hwp_units(header);
        self.footer = mm_to_hwp_units(footer);
        self
    }

    /// 제본 여백 설정 (밀리미터 단위)
    pub fn with_gutter_mm(mut self, gutter: f32) -> Self {
        self.gutter = mm_to_hwp_units(gutter);
        self
    }

    /// 미러 여백 설정
    pub fn with_mirror_margins(mut self, mirror: bool) -> Self {
        self.mirror_margins = mirror;
        self
    }

    /// 좁은 여백 (Office 스타일)
    pub fn narrow() -> Self {
        Self::new_mm(12.7, 12.7, 12.7, 12.7)
    }

    /// 보통 여백 (Office 스타일)
    pub fn normal() -> Self {
        Self::new_mm(25.4, 25.4, 25.4, 25.4)
    }

    /// 넓은 여백 (Office 스타일)
    pub fn wide() -> Self {
        Self::new_mm(50.8, 50.8, 25.4, 25.4)
    }
}

impl PageLayout {
    /// 새로운 페이지 레이아웃 생성
    pub fn new(paper_size: PaperSize, orientation: PageOrientation) -> Self {
        let (width, height) = paper_size.dimensions_hwp_units();
        let (final_width, final_height) = match orientation {
            PageOrientation::Portrait => (width, height),
            PageOrientation::Landscape => (height, width),
        };

        Self {
            paper_size,
            orientation,
            width: final_width,
            height: final_height,
            ..Default::default()
        }
    }

    /// A4 세로 방향
    pub fn a4_portrait() -> Self {
        Self::new(PaperSize::A4, PageOrientation::Portrait)
    }

    /// A4 가로 방향
    pub fn a4_landscape() -> Self {
        Self::new(PaperSize::A4, PageOrientation::Landscape)
    }

    /// Letter 세로 방향
    pub fn letter_portrait() -> Self {
        Self::new(PaperSize::Letter, PageOrientation::Portrait)
    }

    /// Letter 가로 방향
    pub fn letter_landscape() -> Self {
        Self::new(PaperSize::Letter, PageOrientation::Landscape)
    }

    /// 사용자 정의 크기 (밀리미터 단위)
    pub fn custom_mm(width_mm: f32, height_mm: f32, orientation: PageOrientation) -> Self {
        let width = mm_to_hwp_units(width_mm);
        let height = mm_to_hwp_units(height_mm);
        let (final_width, final_height) = match orientation {
            PageOrientation::Portrait => (width, height),
            PageOrientation::Landscape => (height, width),
        };

        Self {
            paper_size: PaperSize::Custom,
            orientation,
            width: final_width,
            height: final_height,
            ..Default::default()
        }
    }

    /// 여백 설정
    pub fn with_margins(mut self, margins: PageMargins) -> Self {
        self.margins = margins;
        self
    }

    /// 다단 설정
    pub fn with_columns(mut self, columns: u16, spacing_mm: f32) -> Self {
        self.columns = columns;
        self.column_spacing = mm_to_hwp_units(spacing_mm);
        self
    }

    /// 단 구분선 표시
    pub fn with_column_line(mut self, show_line: bool) -> Self {
        self.column_line = show_line;
        self
    }

    /// 페이지 배경색 설정
    pub fn with_background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// 페이지 번호 설정
    pub fn with_page_numbering(
        mut self,
        start: u16,
        format: crate::model::header_footer::PageNumberFormat,
    ) -> Self {
        self.start_page_number = start;
        self.page_number_format = format;
        self
    }

    /// 유효 너비 계산 (여백 제외)
    pub fn effective_width(&self) -> u32 {
        let total_margin = self.margins.left + self.margins.right + self.margins.gutter;
        if self.width > total_margin {
            self.width - total_margin
        } else {
            1000 // Minimum width
        }
    }

    /// 유효 높이 계산 (여백 제외)
    pub fn effective_height(&self) -> u32 {
        let total_margin = self.margins.top + self.margins.bottom;
        if self.height > total_margin {
            self.height - total_margin
        } else {
            1000 // Minimum height
        }
    }

    /// 단 너비 계산
    pub fn column_width(&self) -> u32 {
        if self.columns <= 1 {
            self.effective_width()
        } else {
            let total_spacing = (self.columns - 1) as u32 * self.column_spacing;
            let available_width = if self.effective_width() > total_spacing {
                self.effective_width() - total_spacing
            } else {
                1000
            };
            available_width / self.columns as u32
        }
    }

    /// HWP 형식으로 직렬화
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        // 기본 페이지 정보
        writer.write_u32::<LittleEndian>(self.width).unwrap();
        writer.write_u32::<LittleEndian>(self.height).unwrap();

        // 여백 정보
        writer.write_u32::<LittleEndian>(self.margins.left).unwrap();
        writer
            .write_u32::<LittleEndian>(self.margins.right)
            .unwrap();
        writer.write_u32::<LittleEndian>(self.margins.top).unwrap();
        writer
            .write_u32::<LittleEndian>(self.margins.bottom)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.margins.header)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.margins.footer)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.margins.gutter)
            .unwrap();

        // 다단 설정
        writer.write_u16::<LittleEndian>(self.columns).unwrap();
        writer
            .write_u32::<LittleEndian>(self.column_spacing)
            .unwrap();

        // 페이지 속성
        let mut properties = 0u32;
        if self.orientation == PageOrientation::Landscape {
            properties |= 0x01;
        }
        if self.margins.mirror_margins {
            properties |= 0x02;
        }
        if self.column_line {
            properties |= 0x04;
        }
        if self.page_border {
            properties |= 0x08;
        }
        writer.write_u32::<LittleEndian>(properties).unwrap();

        // 페이지 번호 설정
        writer
            .write_u16::<LittleEndian>(self.start_page_number)
            .unwrap();
        writer.write_u8(self.page_number_format as u8).unwrap();

        // 배경색 (선택사항)
        if let Some(color) = self.background_color {
            writer.write_u8(1).unwrap(); // Has background color
            writer.write_u32::<LittleEndian>(color).unwrap();
        } else {
            writer.write_u8(0).unwrap(); // No background color
        }

        data
    }
}

/// 밀리미터를 HWP 단위로 변환
pub fn mm_to_hwp_units(mm: f32) -> u32 {
    (mm * 283.465).round() as u32 // 1mm = 283.465 HWP units
}

/// 인치를 HWP 단위로 변환
pub fn inches_to_hwp_units(inches: f32) -> u32 {
    (inches * 7200.0).round() as u32 // 1 inch = 7200 HWP units
}

/// HWP 단위를 밀리미터로 변환
pub fn hwp_units_to_mm(units: u32) -> f32 {
    units as f32 / 283.465
}

/// HWP 단위를 인치로 변환
pub fn hwp_units_to_inches(units: u32) -> f32 {
    units as f32 / 7200.0
}
