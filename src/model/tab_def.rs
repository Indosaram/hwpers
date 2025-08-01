use crate::error::Result;
use crate::parser::record::Record;

#[derive(Debug, Clone)]
pub struct TabDef {
    pub properties: u32,
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub position: u32,
    pub tab_type: u8,
    pub leader_type: u8,
}

impl TabDef {
    pub fn from_record(record: &Record) -> Result<Self> {
        let mut reader = record.data_reader();
        
        if reader.remaining() < 4 {
            return Err(crate::error::HwpError::ParseError(
                format!("TabDef record too small: {} bytes", reader.remaining())
            ));
        }
        
        let properties = reader.read_u32()?;
        let mut tabs = Vec::new();
        
        // Each tab entry is 6 bytes (4 bytes position + 1 byte type + 1 byte leader)
        while reader.remaining() >= 6 {
            let tab = Tab {
                position: reader.read_u32()?,
                tab_type: reader.read_u8()?,
                leader_type: reader.read_u8()?,
            };
            tabs.push(tab);
        }
        
        Ok(Self {
            properties,
            tabs,
        })
    }
}

impl Tab {
    pub fn is_left_aligned(&self) -> bool {
        self.tab_type & 0x03 == 0
    }
    
    pub fn is_center_aligned(&self) -> bool {
        self.tab_type & 0x03 == 1
    }
    
    pub fn is_right_aligned(&self) -> bool {
        self.tab_type & 0x03 == 2
    }
    
    pub fn is_decimal_aligned(&self) -> bool {
        self.tab_type & 0x03 == 3
    }
    
    pub fn has_leader(&self) -> bool {
        self.leader_type != 0
    }
}