use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128;

use crate::error::{HwpError, Result};

#[derive(Debug, Clone)]
pub struct DistributionDecryptor {
    key: [u8; 16],
}

impl DistributionDecryptor {
    pub fn from_record_data(record_data: &[u8]) -> Result<Self> {
        if record_data.len() < 260 {
            return Err(HwpError::ParseError(format!(
                "Distribution record data too short: expected at least 260, got {}",
                record_data.len()
            )));
        }

        let body = &record_data[4..260];
        let mut decoded = [0u8; 256];
        decoded.copy_from_slice(body);

        obfuscation_transform(&mut decoded);

        let offset = 4 + (decoded[0] & 0x0F) as usize;
        if offset + 16 > 256 {
            return Err(HwpError::ParseError(
                "Invalid distribution data offset".to_string(),
            ));
        }

        let mut key = [0u8; 16];
        key.copy_from_slice(&decoded[offset..offset + 16]);

        Ok(Self { key })
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let cipher = Aes128::new_from_slice(&self.key)
            .map_err(|e| HwpError::ParseError(format!("AES key error: {}", e)))?;

        let mut result = data.to_vec();

        let padding_needed = (16 - (result.len() % 16)) % 16;
        result.extend(vec![0u8; padding_needed]);

        for chunk in result.chunks_exact_mut(16) {
            let block = aes::Block::from_mut_slice(chunk);
            cipher.decrypt_block(block);
        }

        if padding_needed > 0 {
            result.truncate(data.len());
        }

        Ok(result)
    }
}

fn obfuscation_transform(data: &mut [u8; 256]) {
    let seed = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let mut rng = MsvcRng::new(seed);

    let mut value: u8 = 0;
    let mut count: u32 = 0;

    // Index-based loop required: first 4 bytes are seed and must not be XORed
    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        if count == 0 {
            value = rng.next_value();
            count = rng.next_count();
        }

        if i >= 4 {
            data[i] ^= value;
        }

        count -= 1;
    }
}

struct MsvcRng {
    seed: u32,
}

impl MsvcRng {
    fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        (self.seed >> 16) & 0x7FFF
    }

    fn next_value(&mut self) -> u8 {
        (self.rand() & 0xFF) as u8
    }

    fn next_count(&mut self) -> u32 {
        (self.rand() & 0x0F) + 1
    }
}

pub fn decrypt_distribution_stream(data: &[u8], record_data: &[u8]) -> Result<Vec<u8>> {
    let decryptor = DistributionDecryptor::from_record_data(record_data)?;
    decryptor.decrypt(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msvc_rng_deterministic() {
        let mut rng1 = MsvcRng::new(12345);
        let mut rng2 = MsvcRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.rand(), rng2.rand());
        }
    }

    #[test]
    fn test_obfuscation_transform() {
        let mut data = [0u8; 256];
        data[0] = 0x01;
        data[1] = 0x02;
        data[2] = 0x03;
        data[3] = 0x04;

        let original = data;
        obfuscation_transform(&mut data);

        assert_eq!(data[0..4], original[0..4]);
        assert_ne!(data[4..], original[4..]);
    }

    #[test]
    fn test_record_data_too_short() {
        let short_data = [0u8; 100];
        let result = DistributionDecryptor::from_record_data(&short_data);
        assert!(result.is_err());
    }
}
