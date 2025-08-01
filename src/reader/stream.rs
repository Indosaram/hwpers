use crate::error::{HwpError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub struct StreamReader {
    cursor: Cursor<Vec<u8>>,
}

impl StreamReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.cursor.read_u8()?)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(self.cursor.read_u16::<LittleEndian>()?)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(self.cursor.read_u32::<LittleEndian>()?)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.cursor.read_i32::<LittleEndian>()?)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        self.cursor.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_string(&mut self, len: usize) -> Result<String> {
        let bytes = self.read_bytes(len)?;
        let (cow, _, had_errors) = encoding_rs::UTF_16LE.decode(&bytes);
        if had_errors {
            return Err(HwpError::EncodingError(
                "Invalid UTF-16LE string".to_string(),
            ));
        }
        Ok(cow.into_owned().trim_end_matches('\0').to_string())
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    pub fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        let len = self.cursor.get_ref().len();
        len.saturating_sub(pos)
    }
}
