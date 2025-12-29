# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-12-29

### Fixed - HWP Writer Compatibility

#### Critical Bug Fix
- **HWP files now open correctly in Hangul word processor**
  - Fixed FileHeader: version 5.0.3.4, compression disabled, reserved[4]=0x04
  - Fixed Scripts streams: uncompressed raw data matching hwplib format
  - Fixed BodyText structure: added required section/column definition paragraph
    - Section define (`secd`) and column define (`cold`) control characters
    - PAGE_DEF, FOOTNOTE_SHAPE, PAGE_BORDER_FILL records
    - Proper `lastInList` flag handling for paragraph linking
  - Added PARA_LINE_SEG for text layout

#### Technical Changes
- Rewrote `serialize_body_text()` with proper HWP structure
- Added `write_section_definition()` for section control paragraph
- Added `write_content_paragraph()` for text paragraphs
- Updated default values for ParaShape and TabDef

## [0.3.1] - 2025-01-29

### Added - Complete HWP Writer Implementation

#### Core Features
- **HWP Writer Module**: Full HWP document creation capabilities
  - Document structure creation with sections and paragraphs
  - Text content writing with UTF-16LE encoding support
  - Compound File Binary (CFB) format generation
  - Complete serialization for all supported features

#### Rich Text & Formatting
- **Styled Text**: Rich text formatting support
  - Bold, italic, underline, strikethrough
  - Custom fonts and font sizes
  - Text colors and background colors
  - Multiple styles within single paragraph
  - `StyledText` API with range-based styling

- **Paragraph Formatting**
  - Text alignment (left, center, right, justify)
  - Line spacing (percentage-based)
  - Paragraph spacing (before/after in mm)
  - Heading levels 1-6 with automatic sizing
  - Character and paragraph shape management

#### Tables
- **Full Table Support**
  - Table creation with custom rows/columns
  - Cell content and formatting
  - Cell borders with customizable styles
  - Horizontal and vertical cell merging
  - `TableBuilder` API for fluent table construction

#### Lists
- **Complete List Implementation**
  - Bullet lists with different symbols per level (•, ◦, ▪)
  - Numbered lists (1., 2., 3., ...)
  - Alphabetic lists (a), b), c), ...)
  - Roman numeral lists (i., ii., iii., ...)
  - Korean lists (가., 나., 다., ...)
  - Nested lists with proper indentation
  - Paragraph shape-based indentation system

#### Images
- **Image Insertion**
  - PNG, JPEG, BMP, GIF format support
  - Custom dimensions and positioning
  - Image captions with automatic formatting
  - Proper BinData integration
  - 1-based bin_id system

#### Text Boxes
- **Text Box Support**
  - Basic text boxes with positioning
  - Predefined styles (basic, highlight, warning, info, bubble, transparent)
  - Custom styling (borders, backgrounds, colors, alignment)
  - Floating text boxes with rotation and transparency
  - Size and position control in millimeters

#### Hyperlinks
- **Complete Hyperlink Support**
  - URL links with proper serialization
  - Email links (mailto: format)
  - File links
  - Internal bookmarks
  - External document bookmarks
  - Custom styling (colors, underline, visited state)
  - Multiple hyperlinks per paragraph
  - Predefined link styles

#### Page Layout
- **Comprehensive Page Layout Control**
  - Custom page sizes (width/height in mm)
  - Standard sizes (A4, A3, A5, Letter, Legal, Tabloid, B4, B5)
  - Portrait/landscape orientation
  - Custom margins (left, right, top, bottom)
  - Predefined margin presets (narrow, normal, wide)
  - Multi-column layouts with adjustable spacing
  - Page background colors
  - Page numbering with multiple formats

#### Headers & Footers
- **Full Header/Footer Implementation**
  - Custom header and footer text
  - Page numbering (numeric, roman upper/lower, alphabetic)
  - Multiple headers/footers per document
  - Proper serialization and storage

#### Document Properties
- **Metadata Support**
  - Document title, author, subject, keywords
  - Automatic statistics calculation
  - Character count (Hangul, Hanja, English, digits, spaces)
  - Word count and paragraph count
  - Page count estimation
  - Section count tracking

### Changed
- Page layout methods changed from `Result<()>` to `void` for direct state mutation
- Image serialization improved with separate caption paragraphs
- List indentation system switched from text-based to paragraph shape-based
- Heading implementation enhanced with per-level paragraph shapes
- Empty table handling changed from error to no-op

### Fixed
- **Page Layout Bugs**
  - Fixed margin methods overwriting entire page layout
  - All page layout setters now modify state directly

- **Hyperlink Serialization**
  - Completely rewrote `from_record()` to match `to_bytes()` format
  - Fixed deserialization starting at offset 0 instead of offset 48
  - Proper UTF-16LE string parsing with bounds checking

- **List Formatting**
  - Korean list marker format: "가." instead of "가)."
  - Implemented unique para_shape_id for each indentation level
  - Nested list indentation now works correctly

- **Image Features**
  - Caption now added as separate paragraph ("그림: caption")
  - Fixed bin_id to be 1-based (1, 2, 3, ...) instead of 0-based
  - Fixed abs_name to "image1.png", "image2.png" format
  - Picture control paragraph text set to None
  - control_mask set to 2 (0x02)

- **Heading Styles**
  - Each heading level now creates unique paragraph shape
  - Proper spacing_before and spacing_after from HeadingStyle

- **Empty Table Handling**
  - Empty tables no longer throw error
  - Returns Ok(()) without adding paragraphs

### Tests
- 104 tests all passing
- Comprehensive test coverage for all features
- Round-trip serialization tests
- Integration tests for complex documents

### Examples
- `complete_feature_demo.rs` - All features demonstration
- `page_layout_document.rs` - Page layout examples
- `shape_document.rs.disabled` - Future shape drawing reference

### Documentation
- Complete README update with all implemented features
- TODO comments for unimplemented features (shapes, charts, etc.)
- Inline documentation for all public APIs

### Not Yet Implemented
- Geometric shapes (rectangles, circles, lines, arrows, polygons)
- Charts and graphs
- Mathematical equations (MathML)
- Forms and input fields
- Comments and annotations
- Track changes/revision history
- Mail merge fields

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

[0.3.1]: https://github.com/Indosaram/hwpers/compare/v0.2.0...v0.3.1
[0.2.0]: https://github.com/Indosaram/hwpers/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Indosaram/hwpers/releases/tag/v0.1.0