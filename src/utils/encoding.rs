use crate::error::{HwpError, Result};
use encoding_rs::UTF_16LE;

pub fn utf16le_to_string(data: &[u8]) -> Result<String> {
    let (cow, _, had_errors) = UTF_16LE.decode(data);
    if had_errors {
        return Err(HwpError::EncodingError("Invalid UTF-16LE data".to_string()));
    }
    Ok(cow.into_owned())
}

pub fn string_to_utf16le(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for ch in s.encode_utf16() {
        result.extend_from_slice(&ch.to_le_bytes());
    }
    result
}
