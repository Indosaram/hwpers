use crate::error::Result;
use crate::model::header_footer::HeaderFooterCollection;
use crate::model::page_layout::PageLayout;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct PageDef {
    pub width: u32,
    pub height: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub top_margin: u32,
    pub bottom_margin: u32,
    pub header_margin: u32,
    pub footer_margin: u32,
    pub gutter_margin: u32,
    pub properties: u32,
    pub footnote_shape_id: u16,
    pub page_border_fill_id: u16,
    /// Header/Footer 컬렉션
    pub header_footer: HeaderFooterCollection,
    /// 고급 페이지 레이아웃 설정
    pub layout: Option<PageLayout>,
}

impl PageDef {
    pub fn from_record(record: &Record) -> Result<Self> {
        // PageDef in HWP is stored as a variable attribute list
        // This is a simplified parser that handles the most common case
        let data = &record.data;

        // Default A4 values in HWP units (1 unit = 1/7200 inch)
        let mut width = 59528; // 210mm
        let mut height = 84188; // 297mm
        let mut left_margin = 8504; // 30mm
        let mut right_margin = 8504; // 30mm
        let mut top_margin = 5669; // 20mm
        let mut bottom_margin = 4252; // 15mm
        let footer_margin = 4252; // 15mm
        let gutter_margin = 0;
        let properties = 0;

        // Try to parse the first few values as simple u32s
        if data.len() >= 8 {
            let mut reader = record.data_reader();

            // First two values appear to be width and height
            if let Ok(w) = reader.read_u32() {
                if w > 0 && w < 0x100000 {
                    // Reasonable width
                    width = w;
                }
            }
            if let Ok(h) = reader.read_u32() {
                if h > 0 && h < 0x100000 {
                    // Reasonable height
                    height = h;
                }
            }

            // Try to read margins - they might be at different positions
            if reader.remaining() >= 24 {
                let m1 = reader.read_u32().unwrap_or(left_margin);
                let m2 = reader.read_u32().unwrap_or(right_margin);
                let m3 = reader.read_u32().unwrap_or(top_margin);
                let m4 = reader.read_u32().unwrap_or(bottom_margin);

                // Only use if they seem reasonable
                if m1 < width / 2 {
                    left_margin = m1;
                }
                if m2 < width / 2 {
                    right_margin = m2;
                }
                if m3 < height / 2 {
                    top_margin = m3;
                }
                if m4 < height / 2 {
                    bottom_margin = m4;
                }
            }
        }

        Ok(Self {
            width,
            height,
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            header_margin: footer_margin,
            footer_margin,
            gutter_margin,
            properties,
            footnote_shape_id: 0,
            page_border_fill_id: 0,
            header_footer: HeaderFooterCollection::new(),
            layout: None,
        })
    }

    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    pub fn effective_width(&self) -> u32 {
        self.width - self.left_margin - self.right_margin - self.gutter_margin
    }

    pub fn effective_height(&self) -> u32 {
        self.height - self.top_margin - self.bottom_margin
    }

    /// Create a new default PageDef for writing (A4 size)
    pub fn new_default() -> Self {
        Self {
            width: 59528,        // 210mm in HWP units (1 unit = 1/7200 inch)
            height: 84188,       // 297mm
            left_margin: 8504,   // 30mm
            right_margin: 8504,  // 30mm
            top_margin: 5669,    // 20mm
            bottom_margin: 4252, // 15mm
            header_margin: 4252, // 15mm
            footer_margin: 4252, // 15mm
            gutter_margin: 0,
            properties: 0,
            footnote_shape_id: 0,
            page_border_fill_id: 0,
            header_footer: HeaderFooterCollection::new(),
            layout: None,
        }
    }

    /// Create PageDef from PageLayout
    pub fn from_layout(layout: PageLayout) -> Self {
        Self {
            width: layout.width,
            height: layout.height,
            left_margin: layout.margins.left,
            right_margin: layout.margins.right,
            top_margin: layout.margins.top,
            bottom_margin: layout.margins.bottom,
            header_margin: layout.margins.header,
            footer_margin: layout.margins.footer,
            gutter_margin: layout.margins.gutter,
            properties: 0, // Will be set based on layout properties
            footnote_shape_id: 0,
            page_border_fill_id: 0,
            header_footer: HeaderFooterCollection::new(),
            layout: Some(layout),
        }
    }

    /// Update margins from layout
    pub fn update_from_layout(&mut self, layout: PageLayout) {
        self.width = layout.width;
        self.height = layout.height;
        self.left_margin = layout.margins.left;
        self.right_margin = layout.margins.right;
        self.top_margin = layout.margins.top;
        self.bottom_margin = layout.margins.bottom;
        self.header_margin = layout.margins.header;
        self.footer_margin = layout.margins.footer;
        self.gutter_margin = layout.margins.gutter;
        self.layout = Some(layout);
    }

    /// Get current layout or create default
    #[allow(clippy::field_reassign_with_default)]
    pub fn get_layout(&self) -> PageLayout {
        if let Some(layout) = &self.layout {
            layout.clone()
        } else {
            // Create layout from current PageDef values
            let mut layout = PageLayout::default();
            layout.width = self.width;
            layout.height = self.height;
            layout.margins.left = self.left_margin;
            layout.margins.right = self.right_margin;
            layout.margins.top = self.top_margin;
            layout.margins.bottom = self.bottom_margin;
            layout.margins.header = self.header_margin;
            layout.margins.footer = self.footer_margin;
            layout.margins.gutter = self.gutter_margin;
            layout
        }
    }

    /// Serialize to bytes for HWP file
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Cursor;

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);

        writer.write_u32::<LittleEndian>(self.width).unwrap();
        writer.write_u32::<LittleEndian>(self.height).unwrap();
        writer.write_u32::<LittleEndian>(self.left_margin).unwrap();
        writer.write_u32::<LittleEndian>(self.right_margin).unwrap();
        writer.write_u32::<LittleEndian>(self.top_margin).unwrap();
        writer
            .write_u32::<LittleEndian>(self.bottom_margin)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.header_margin)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.footer_margin)
            .unwrap();
        writer
            .write_u32::<LittleEndian>(self.gutter_margin)
            .unwrap();
        writer.write_u32::<LittleEndian>(self.properties).unwrap();
        writer
            .write_u16::<LittleEndian>(self.footnote_shape_id)
            .unwrap();
        writer
            .write_u16::<LittleEndian>(self.page_border_fill_id)
            .unwrap();

        data
    }
}
