use pyo3::exceptions::{PyOSError, PyFileNotFoundError, PyValueError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use hwpers::hwpx::writer::{
    HeaderFooterApplyTo as RustHeaderFooterApplyTo,
    HwpxFooter as RustFooter,
    HwpxHeader as RustHeader,
    HwpxHyperlink as RustHyperlink,
    HwpxImage as RustImage,
    HwpxImageFormat as RustImageFormat,
    HwpxTable as RustTable,
    HwpxTextStyle as RustTextStyle,
    HwpxWriter as RustWriter,
    PageNumberFormat as RustPageNumberFormat,
    StyledText as RustStyledText,
};
use hwpers::hwpx::HwpxReader as RustReader;
use hwpers::error::HwpError;

// ---------------------------------------------------------------------------
// Error conversion
// ---------------------------------------------------------------------------

fn to_py_err(e: HwpError) -> PyErr {
    match e {
        HwpError::Io(io_err) => PyOSError::new_err(io_err.to_string()),
        HwpError::NotFound(msg) => PyFileNotFoundError::new_err(msg),
        HwpError::InvalidInput(msg) => PyValueError::new_err(msg),
        other => PyRuntimeError::new_err(other.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
}

impl From<ImageFormat> for RustImageFormat {
    fn from(f: ImageFormat) -> Self {
        match f {
            ImageFormat::Png => RustImageFormat::Png,
            ImageFormat::Jpeg => RustImageFormat::Jpeg,
            ImageFormat::Gif => RustImageFormat::Gif,
            ImageFormat::Bmp => RustImageFormat::Bmp,
        }
    }
}

impl From<RustImageFormat> for ImageFormat {
    fn from(f: RustImageFormat) -> Self {
        match f {
            RustImageFormat::Png => ImageFormat::Png,
            RustImageFormat::Jpeg => ImageFormat::Jpeg,
            RustImageFormat::Gif => ImageFormat::Gif,
            RustImageFormat::Bmp => ImageFormat::Bmp,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderFooterApplyTo {
    All,
    Odd,
    Even,
}

impl From<HeaderFooterApplyTo> for RustHeaderFooterApplyTo {
    fn from(a: HeaderFooterApplyTo) -> Self {
        match a {
            HeaderFooterApplyTo::All => RustHeaderFooterApplyTo::All,
            HeaderFooterApplyTo::Odd => RustHeaderFooterApplyTo::Odd,
            HeaderFooterApplyTo::Even => RustHeaderFooterApplyTo::Even,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageNumberFormat {
    Numeric,
    RomanLower,
    RomanUpper,
    AlphaLower,
    AlphaUpper,
}

impl From<PageNumberFormat> for RustPageNumberFormat {
    fn from(f: PageNumberFormat) -> Self {
        match f {
            PageNumberFormat::Numeric => RustPageNumberFormat::Numeric,
            PageNumberFormat::RomanLower => RustPageNumberFormat::RomanLower,
            PageNumberFormat::RomanUpper => RustPageNumberFormat::RomanUpper,
            PageNumberFormat::AlphaLower => RustPageNumberFormat::AlphaLower,
            PageNumberFormat::AlphaUpper => RustPageNumberFormat::AlphaUpper,
        }
    }
}

// ---------------------------------------------------------------------------
// TextStyle
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct TextStyle {
    inner: RustTextStyle,
}

#[pymethods]
impl TextStyle {
    #[new]
    #[pyo3(signature = (*, font_size=None, bold=false, italic=false, underline=false, strikethrough=false, color=0))]
    fn new(
        font_size: Option<u32>,
        bold: bool,
        italic: bool,
        underline: bool,
        strikethrough: bool,
        color: u32,
    ) -> Self {
        let mut s = RustTextStyle::new();
        if let Some(sz) = font_size {
            s.font_size = Some(sz);
        }
        s.bold = bold;
        s.italic = italic;
        s.underline = underline;
        s.strikethrough = strikethrough;
        s.color = color;
        Self { inner: s }
    }

    fn bold(&self) -> Self {
        let mut s = self.inner.clone();
        s.bold = true;
        Self { inner: s }
    }

    fn italic(&self) -> Self {
        let mut s = self.inner.clone();
        s.italic = true;
        Self { inner: s }
    }

    fn underline(&self) -> Self {
        let mut s = self.inner.clone();
        s.underline = true;
        Self { inner: s }
    }

    fn strikethrough(&self) -> Self {
        let mut s = self.inner.clone();
        s.strikethrough = true;
        Self { inner: s }
    }

    fn size(&self, size_pt: u32) -> Self {
        let mut s = self.inner.clone();
        s.font_size = Some(size_pt);
        Self { inner: s }
    }

    fn color(&self, color: u32) -> Self {
        let mut s = self.inner.clone();
        s.color = color;
        Self { inner: s }
    }

    fn __repr__(&self) -> String {
        let font_size = match self.inner.font_size {
            Some(s) => format!("{}", s),
            None => "None".to_string(),
        };
        format!(
            "TextStyle(font_size={}, bold={}, italic={}, underline={}, strikethrough={}, color=0x{:06X})",
            font_size,
            if self.inner.bold { "True" } else { "False" },
            if self.inner.italic { "True" } else { "False" },
            if self.inner.underline { "True" } else { "False" },
            if self.inner.strikethrough { "True" } else { "False" },
            self.inner.color
        )
    }
}

// ---------------------------------------------------------------------------
// StyledText
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct StyledText {
    inner: RustStyledText,
}

#[pymethods]
impl StyledText {
    #[new]
    #[pyo3(signature = (text, *, style=None))]
    fn new(text: &str, style: Option<TextStyle>) -> Self {
        let s = match style {
            Some(ts) => RustStyledText::with_style(text, ts.inner),
            None => RustStyledText::new(text),
        };
        Self { inner: s }
    }

    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    fn __repr__(&self) -> String {
        format!("StyledText(\"{}\")", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct Image {
    inner: RustImage,
}

#[pymethods]
impl Image {
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        RustImage::from_bytes(data.to_vec())
            .map(|img| Self { inner: img })
            .ok_or_else(|| PyValueError::new_err("Unsupported or invalid image format"))
    }

    fn with_size(&self, width_mm: u32, height_mm: u32) -> Self {
        Self {
            inner: self.inner.clone().with_size(width_mm, height_mm),
        }
    }

    #[getter]
    fn format(&self) -> ImageFormat {
        self.inner.format.into()
    }

    #[getter]
    fn width_mm(&self) -> Option<u32> {
        self.inner.width_mm
    }

    #[getter]
    fn height_mm(&self) -> Option<u32> {
        self.inner.height_mm
    }

    fn __repr__(&self) -> String {
        format!(
            "Image(format={:?}, width_mm={:?}, height_mm={:?})",
            ImageFormat::from(self.inner.format),
            self.inner.width_mm,
            self.inner.height_mm
        )
    }
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct Table {
    inner: RustTable,
}

#[pymethods]
impl Table {
    #[new]
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            inner: RustTable::new(rows, cols),
        }
    }

    #[staticmethod]
    fn from_data(data: Vec<Vec<String>>) -> Self {
        let str_data: Vec<Vec<&str>> = data.iter().map(|row| row.iter().map(|s| s.as_str()).collect()).collect();
        Self {
            inner: RustTable::from_data(str_data),
        }
    }

    fn set_cell(&mut self, row: usize, col: usize, value: &str) {
        self.inner.set_cell(row, col, value);
    }

    #[getter]
    fn rows(&self) -> Vec<Vec<String>> {
        self.inner.rows.clone()
    }

    fn __repr__(&self) -> String {
        let rows = self.inner.rows.len();
        let cols = self.inner.rows.first().map(|r| r.len()).unwrap_or(0);
        format!("Table(rows={}, cols={})", rows, cols)
    }
}

// ---------------------------------------------------------------------------
// Hyperlink
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct Hyperlink {
    inner: RustHyperlink,
}

#[pymethods]
impl Hyperlink {
    #[new]
    fn new(text: &str, url: &str) -> Self {
        Self {
            inner: RustHyperlink::new(text, url),
        }
    }

    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    #[getter]
    fn url(&self) -> &str {
        &self.inner.url
    }

    fn __repr__(&self) -> String {
        format!("Hyperlink(\"{}\", \"{}\")", self.inner.text, self.inner.url)
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct Header {
    inner: RustHeader,
}

#[pymethods]
impl Header {
    #[new]
    fn new(text: &str) -> Self {
        Self {
            inner: RustHeader::new(text),
        }
    }

    #[staticmethod]
    fn for_odd_pages(text: &str) -> Self {
        Self {
            inner: RustHeader::for_odd_pages(text),
        }
    }

    #[staticmethod]
    fn for_even_pages(text: &str) -> Self {
        Self {
            inner: RustHeader::for_even_pages(text),
        }
    }

    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    fn __repr__(&self) -> String {
        format!("Header(\"{}\")", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug, Clone)]
pub struct Footer {
    inner: RustFooter,
}

#[pymethods]
impl Footer {
    #[new]
    fn new(text: &str) -> Self {
        Self {
            inner: RustFooter::new(text),
        }
    }

    fn with_page_number(&self) -> Self {
        Self {
            inner: self.inner.clone().with_page_number(),
        }
    }

    fn with_page_number_format(&self, format: PageNumberFormat) -> Self {
        Self {
            inner: self.inner.clone().with_page_number_format(format.into()),
        }
    }

    fn for_odd_pages(&self) -> Self {
        Self {
            inner: self.inner.clone().for_odd_pages(),
        }
    }

    fn for_even_pages(&self) -> Self {
        Self {
            inner: self.inner.clone().for_even_pages(),
        }
    }

    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    #[getter]
    fn include_page_number(&self) -> bool {
        self.inner.include_page_number
    }

    fn __repr__(&self) -> String {
        format!(
            "Footer(\"{}\", include_page_number={})",
            self.inner.text,
            if self.inner.include_page_number { "True" } else { "False" }
        )
    }
}

// ---------------------------------------------------------------------------
// Document (read result)
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Debug)]
pub struct Document {
    inner: hwpers::HwpDocument,
}

#[pymethods]
impl Document {
    fn extract_text(&self) -> String {
        self.inner.extract_text()
    }

    #[getter]
    fn title(&self) -> Option<String> {
        self.inner.title().map(|s| s.to_string())
    }

    #[getter]
    fn author(&self) -> Option<String> {
        self.inner.author().map(|s| s.to_string())
    }

    fn __repr__(&self) -> String {
        let text = self.inner.extract_text();
        let preview = if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text
        };
        format!("Document(\"{}\")", preview)
    }
}

// ---------------------------------------------------------------------------
// Reader
// ---------------------------------------------------------------------------

#[pyclass]
pub struct Reader;

#[pymethods]
impl Reader {
    #[new]
    fn new() -> Self {
        Self
    }

    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Document> {
        RustReader::from_file(path)
            .map(|doc| Document { inner: doc })
            .map_err(to_py_err)
    }

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Document> {
        RustReader::from_bytes(data)
            .map(|doc| Document { inner: doc })
            .map_err(to_py_err)
    }
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

#[pyclass]
pub struct Writer {
    inner: RustWriter,
}

#[pymethods]
impl Writer {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustWriter::new(),
        }
    }

    fn add_paragraph(&mut self, text: &str) -> PyResult<()> {
        self.inner.add_paragraph(text).map_err(to_py_err)
    }

    fn add_styled_paragraph(&mut self, text: &str, style: &TextStyle) -> PyResult<()> {
        self.inner
            .add_styled_paragraph(text, style.inner.clone())
            .map_err(to_py_err)
    }

    fn add_mixed_styled_paragraph(&mut self, runs: Vec<StyledText>) -> PyResult<()> {
        let rust_runs: Vec<RustStyledText> = runs.into_iter().map(|r| r.inner).collect();
        self.inner
            .add_mixed_styled_paragraph(rust_runs)
            .map_err(to_py_err)
    }

    fn add_table(&mut self, table: &Table) -> PyResult<()> {
        self.inner.add_table(table.inner.clone()).map_err(to_py_err)
    }

    fn add_image(&mut self, image: &Image) -> PyResult<()> {
        self.inner.add_image(image.inner.clone()).map_err(to_py_err)
    }

    fn add_image_from_file(&mut self, path: &str) -> PyResult<()> {
        self.inner.add_image_from_file(path).map_err(to_py_err)
    }

    fn add_hyperlink(&mut self, display_text: &str, url: &str) -> PyResult<()> {
        self.inner.add_hyperlink(display_text, url).map_err(to_py_err)
    }

    fn add_paragraph_with_hyperlinks(
        &mut self,
        text: &str,
        links: Vec<Hyperlink>,
    ) -> PyResult<()> {
        let rust_links: Vec<RustHyperlink> = links.into_iter().map(|l| l.inner).collect();
        self.inner
            .add_paragraph_with_hyperlinks(text, rust_links)
            .map_err(to_py_err)
    }

    fn add_header(&mut self, text: &str) {
        self.inner.add_header(text);
    }

    fn add_header_config(&mut self, header: &Header) {
        self.inner.add_header_config(header.inner.clone());
    }

    fn add_footer(&mut self, text: &str) {
        self.inner.add_footer(text);
    }

    fn add_footer_with_page_number(&mut self, prefix: &str) {
        self.inner.add_footer_with_page_number(prefix);
    }

    fn add_footer_config(&mut self, footer: &Footer) {
        self.inner.add_footer_config(footer.inner.clone());
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.inner.to_bytes().map_err(to_py_err)?;
        Ok(PyBytes::new_bound(py, &bytes))
    }

    fn save_to_file(&self, path: &str) -> PyResult<()> {
        self.inner.save_to_file(path).map_err(to_py_err)
    }

    fn __repr__(&self) -> String {
        "Writer()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule]
fn _hwpers(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Writer>()?;
    m.add_class::<TextStyle>()?;
    m.add_class::<StyledText>()?;
    m.add_class::<Table>()?;
    m.add_class::<Image>()?;
    m.add_class::<Hyperlink>()?;
    m.add_class::<Header>()?;
    m.add_class::<Footer>()?;
    m.add_class::<Reader>()?;
    m.add_class::<Document>()?;
    m.add_class::<ImageFormat>()?;
    m.add_class::<HeaderFooterApplyTo>()?;
    m.add_class::<PageNumberFormat>()?;
    Ok(())
}
