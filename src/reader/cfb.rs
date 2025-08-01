use crate::error::{HwpError, Result};
use cfb::CompoundFile;
use std::io::{Read, Seek};
use std::path::Path;

pub struct CfbReader<F> {
    cfb: CompoundFile<F>,
}

impl CfbReader<std::fs::File> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let cfb = CompoundFile::open(file)
            .map_err(|e| HwpError::Cfb(format!("Failed to open CFB: {}", e)))?;
        Ok(Self { cfb })
    }
}

impl<F: Read + Seek> CfbReader<F> {
    pub fn new(reader: F) -> Result<Self> {
        let cfb = CompoundFile::open(reader)
            .map_err(|e| HwpError::Cfb(format!("Failed to open CFB: {}", e)))?;
        Ok(Self { cfb })
    }

    pub fn read_stream(&mut self, path: &str) -> Result<Vec<u8>> {
        let mut stream = self
            .cfb
            .open_stream(path)
            .map_err(|e| HwpError::NotFound(format!("Stream '{}' not found: {}", path, e)))?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn stream_exists(&self, path: &str) -> bool {
        self.cfb.exists(path)
    }

    pub fn list_streams(&self) -> Vec<String> {
        let mut streams = Vec::new();
        for entry in self.cfb.walk() {
            if entry.is_stream() {
                streams.push(entry.path().display().to_string());
            }
        }
        streams
    }
}
