# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-01-29

### Added
- **HWP Writer Module**: Initial implementation of HWP document creation capabilities
  - Basic document structure creation with sections and paragraphs
  - Text content writing with UTF-16LE encoding support
  - Compound File Binary (CFB) format generation

- **Hyperlink Support**
  - URL, email, file, and bookmark link types
  - `Hyperlink` model with various constructors
  - Serialization with correct control ID ('gsh ')
  - Multiple hyperlinks per paragraph support

- **Header/Footer Functionality**
  - Basic header and footer support
  - Page number insertion with various formats (numeric, roman, alphabetic)
  - 40-byte structure implementation based on file analysis

- **Page Layout Configuration**
  - Custom page sizes (width/height in mm)
  - Page margins (top/bottom/left/right)
  - Page orientation (portrait/landscape)
  - Standard page sizes (A4, Letter, Legal)

- **Paragraph Formatting**
  - Text alignment (left, center, right, justify)
  - Line spacing (percentage-based)
  - Paragraph spacing (before/after in mm)
  - Character and paragraph shape management

- **New Model Structures**
  - `hyperlink::Hyperlink` - Hyperlink data model
  - `header_footer::HeaderFooter` - Header/footer configuration
  - `page_layout::PageLayout` - Page layout settings
  - `text_box::TextBox` - Text box structure (partial)

- **Writer API Methods**
  - `HwpWriter::new()` - Create new document writer
  - `add_paragraph()` - Add text paragraph
  - `add_paragraph_with_hyperlinks()` - Add paragraph with links
  - `add_aligned_paragraph()` - Add paragraph with alignment
  - `add_paragraph_with_spacing()` - Add paragraph with custom spacing
  - `add_header()` / `add_footer_with_page_number()` - Header/footer management
  - `set_custom_page_size()` / `set_page_margins_mm()` - Page layout
  - `save_to_file()` - Write document to file

- **Examples**
  - `complete_feature_demo.rs` - Comprehensive feature demonstration
  - `hyperlink_document.rs` - Hyperlink examples
  - `header_footer_document.rs` - Header/footer usage
  - Various analysis tools for understanding HWP structure

### Changed
- Updated `Document` model to support writer functionality
- Enhanced `Section` and `Paragraph` models for serialization
- Improved error handling with writer-specific error types
- Extended parser to better understand control structures

### Fixed
- Hyperlink control ID corrected to 'gsh ' (0x20687367)
- Header/footer structure corrected to 40-byte format
- Picture control ID corrected to '$pic' (0x63697024)
- UTF-16LE encoding handling for Korean text

### Known Issues
- Image/BinData integration incomplete - images won't display
- Style management system uses hardcoded IDs
- Header/footer text storage mechanism incomplete
- Position tracking for hyperlinks within paragraphs needs refinement
- No compression support for writer (reader supports both)
- Tables, lists, and text boxes not yet implemented

## [0.2.0] - 2025-01-28

### Added
- Initial public release with HWP parser functionality
- Complete HWP 5.0 format parsing support
- Visual layout rendering engine
- SVG export capabilities
- Font and style preservation
- Multi-column layout support

### Features
- Parse all document components (text, tables, embedded objects)
- Extract and preserve formatting information
- Render documents with pixel-perfect accuracy
- Export to SVG format
- Zero-copy parsing for efficiency

## [0.1.0] - 2025-01-20

### Added
- Initial project structure
- Basic HWP file reading capability
- CFB (Compound File Binary) format support
- Document tree parsing
- Text extraction functionality

---

[0.3.0]: https://github.com/Indosaram/hwpers/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Indosaram/hwpers/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Indosaram/hwpers/releases/tag/v0.1.0