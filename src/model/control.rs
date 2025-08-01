#[derive(Debug, Clone)]
pub enum Control {
    SectionDef,
    ColumnDef,
    Table,
    ShapeObject,
    Equation,
    Picture,
    Header,
    Footer,
    Footnote,
    Endnote,
    AutoNumber,
    NewNumber,
    PageHide,
    PageOddEvenAdjust,
    PageNumberPosition,
    IndexMark,
    BookMark,
    OverlappingLetter,
    DutmalSaero,
    HiddenComment,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub properties: u32,
    pub rows: u16,
    pub cols: u16,
    pub cell_spacing: u16,
    pub left_margin: i32,
    pub right_margin: i32,
    pub top_margin: i32,
    pub bottom_margin: i32,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub list_header_id: u32,
    pub col_span: u16,
    pub row_span: u16,
    pub width: u32,
    pub height: u32,
    pub left_margin: u16,
    pub right_margin: u16,
    pub top_margin: u16,
    pub bottom_margin: u16,
    pub border_fill_id: u16,
    pub text_width: u32,
    pub field_name: String,
}

impl Table {
    pub fn from_record(record: &crate::parser::record::Record) -> crate::error::Result<Self> {
        let mut reader = record.data_reader();
        
        if reader.remaining() < 20 {
            return Err(crate::error::HwpError::ParseError(
                format!("Table record too small: {} bytes", reader.remaining())
            ));
        }
        
        let properties = reader.read_u32()?;
        let rows = reader.read_u16()?;
        let cols = reader.read_u16()?;
        let cell_spacing = reader.read_u16()?;
        let left_margin = reader.read_i32()?;
        let right_margin = reader.read_i32()?;
        let top_margin = reader.read_i32()?;
        let bottom_margin = reader.read_i32()?;
        
        // Read cells if available
        let mut cells = Vec::new();
        let total_cells = (rows * cols) as usize;
        
        for _ in 0..total_cells {
            if reader.remaining() < 34 {
                break; // Not enough data for a complete cell
            }
            
            let cell = TableCell {
                list_header_id: reader.read_u32()?,
                col_span: reader.read_u16()?,
                row_span: reader.read_u16()?,
                width: reader.read_u32()?,
                height: reader.read_u32()?,
                left_margin: reader.read_u16()?,
                right_margin: reader.read_u16()?,
                top_margin: reader.read_u16()?,
                bottom_margin: reader.read_u16()?,
                border_fill_id: reader.read_u16()?,
                text_width: reader.read_u32()?,
                field_name: {
                    if reader.remaining() >= 2 {
                        let name_len = reader.read_u16()? as usize;
                        if reader.remaining() >= name_len * 2 {
                            reader.read_string(name_len * 2)?
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                },
            };
            cells.push(cell);
        }
        
        Ok(Self {
            properties,
            rows,
            cols,
            cell_spacing,
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            cells,
        })
    }
}