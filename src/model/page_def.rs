use crate::error::Result;
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
}

impl PageDef {
    pub fn from_record(record: &Record) -> Result<Self> {
        // PageDef in HWP is stored as a variable attribute list
        // This is a simplified parser that handles the most common case
        let data = &record.data;
        
        // Default A4 values in HWP units (1 unit = 1/7200 inch)
        let mut width = 59528;  // 210mm
        let mut height = 84188; // 297mm  
        let mut left_margin = 8504;  // 30mm
        let mut right_margin = 8504; // 30mm
        let mut top_margin = 5669;   // 20mm
        let mut bottom_margin = 4252; // 15mm
        let footer_margin = 4252; // 15mm
        let gutter_margin = 0;
        let properties = 0;
        
        // Try to parse the first few values as simple u32s
        if data.len() >= 8 {
            let mut reader = record.data_reader();
            
            // First two values appear to be width and height
            if let Ok(w) = reader.read_u32() {
                if w > 0 && w < 0x100000 { // Reasonable width
                    width = w;
                }
            }
            if let Ok(h) = reader.read_u32() {
                if h > 0 && h < 0x100000 { // Reasonable height
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
                if m1 < width/2 { left_margin = m1; }
                if m2 < width/2 { right_margin = m2; }
                if m3 < height/2 { top_margin = m3; }
                if m4 < height/2 { bottom_margin = m4; }
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
}