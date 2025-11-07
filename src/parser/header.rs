use crate::error::{HwpError, Result};
use crate::reader::StreamReader;

const HWP_SIGNATURE: &[u8] = b"HWP Document File";

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub signature: [u8; 32],
    pub version: u32,
    pub flags: u32,
    pub reserved: [u8; 216],
}

impl FileHeader {
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        if data.len() < 256 {
            return Err(HwpError::InvalidFormat(
                "FileHeader must be 256 bytes".to_string(),
            ));
        }

        let mut reader = StreamReader::new(data);

        let mut signature = [0u8; 32];
        signature.copy_from_slice(&reader.read_bytes(32)?);

        // Verify signature
        if &signature[..17] != HWP_SIGNATURE {
            return Err(HwpError::InvalidFormat("Invalid HWP signature".to_string()));
        }

        let version = reader.read_u32()?;
        let flags = reader.read_u32()?;

        let mut reserved = [0u8; 216];
        reserved.copy_from_slice(&reader.read_bytes(216)?);

        Ok(Self {
            signature,
            version,
            flags,
            reserved,
        })
    }

    pub fn is_compressed(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn is_encrypted(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn is_distribute(&self) -> bool {
        (self.flags & 0x04) != 0
    }

    pub fn is_script(&self) -> bool {
        (self.flags & 0x08) != 0
    }

    pub fn is_drm(&self) -> bool {
        (self.flags & 0x10) != 0
    }

    pub fn is_xml_template(&self) -> bool {
        (self.flags & 0x20) != 0
    }

    pub fn is_history(&self) -> bool {
        (self.flags & 0x40) != 0
    }

    pub fn is_sign(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    pub fn is_certificate_encrypt(&self) -> bool {
        (self.flags & 0x100) != 0
    }

    pub fn is_sign_spare(&self) -> bool {
        (self.flags & 0x200) != 0
    }

    pub fn is_certificate_drm(&self) -> bool {
        (self.flags & 0x400) != 0
    }

    pub fn is_ccl(&self) -> bool {
        (self.flags & 0x800) != 0
    }

    pub fn version_string(&self) -> String {
        let major = (self.version >> 24) & 0xFF;
        let minor = (self.version >> 16) & 0xFF;
        let build = (self.version >> 8) & 0xFF;
        let revision = self.version & 0xFF;

        format!("{major}.{minor}.{build}.{revision}")
    }
}
impl FileHeader {
    /// Create a new default FileHeader for writing
    pub fn new_default() -> Self {
        let mut signature = [0u8; 32];
        signature[..17].copy_from_slice(HWP_SIGNATURE);

        Self {
            signature,
            version: 0x05050114, // HWP 5.0.5.1 version
            flags: 0x01,         // Enable compression
            reserved: [0u8; 216],
        }
    }

    /// Convert FileHeader to bytes for writing
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.reserved);
        bytes
    }

    /// Set compression flag
    pub fn set_compressed(&mut self, compressed: bool) -> &mut Self {
        if compressed {
            self.flags |= 0x01;
        } else {
            self.flags &= !0x01;
        }
        self
    }

    /// Set encryption flag
    pub fn set_encrypted(&mut self, encrypted: bool) -> &mut Self {
        if encrypted {
            self.flags |= 0x02;
        } else {
            self.flags &= !0x02;
        }
        self
    }

    /// Set document version
    pub fn set_version(&mut self, major: u8, minor: u8, build: u8, revision: u8) -> &mut Self {
        self.version = ((major as u32) << 24)
            | ((minor as u32) << 16)
            | ((build as u32) << 8)
            | (revision as u32);
        self
    }
}
