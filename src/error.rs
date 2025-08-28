use thiserror::Error;

#[derive(Error, Debug)]
pub enum HwpError {
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CFB error: {0}")]
    Cfb(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, HwpError>;
