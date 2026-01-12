#[derive(Debug, Clone)]
pub struct PreviewImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Gif,
    Bmp,
    Unknown,
}

impl PreviewImage {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let format = Self::detect_format(&data);
        Self { data, format }
    }

    fn detect_format(data: &[u8]) -> ImageFormat {
        if data.len() < 8 {
            return ImageFormat::Unknown;
        }

        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            ImageFormat::Png
        } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            ImageFormat::Gif
        } else if data.starts_with(&[0x42, 0x4D]) {
            ImageFormat::Bmp
        } else {
            ImageFormat::Unknown
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn extension(&self) -> &'static str {
        match self.format {
            ImageFormat::Png => "png",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Unknown => "bin",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        let img = PreviewImage::from_bytes(png_header);
        assert_eq!(img.format, ImageFormat::Png);
        assert_eq!(img.extension(), "png");
    }

    #[test]
    fn test_detect_gif() {
        let gif_header = b"GIF89a\x00\x00".to_vec();
        let img = PreviewImage::from_bytes(gif_header);
        assert_eq!(img.format, ImageFormat::Gif);
    }

    #[test]
    fn test_detect_bmp() {
        let bmp_header = vec![0x42, 0x4D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let img = PreviewImage::from_bytes(bmp_header);
        assert_eq!(img.format, ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_unknown() {
        let unknown = vec![0x00, 0x01, 0x02, 0x03];
        let img = PreviewImage::from_bytes(unknown);
        assert_eq!(img.format, ImageFormat::Unknown);
    }
}
