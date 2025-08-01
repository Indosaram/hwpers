use flate2::read::ZlibDecoder;
use std::io::Read;
use crate::error::Result;

pub fn decompress_stream(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    // HWP files use raw deflate without zlib header
    // Try raw deflate first
    use flate2::read::DeflateDecoder;
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => {
            // If raw deflate fails, try zlib
            let mut decoder = ZlibDecoder::new(data);
            let mut decompressed = Vec::new();
            
            match decoder.read_to_end(&mut decompressed) {
                Ok(_) => Ok(decompressed),
                Err(_) => {
                    // If both fail, return data as-is (might not be compressed)
                    Ok(data.to_vec())
                }
            }
        }
    }
}