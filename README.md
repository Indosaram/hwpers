# hwpers

[![Crates.io](https://img.shields.io/crates/v/hwpers.svg)](https://crates.io/crates/hwpers)
[![Documentation](https://docs.rs/hwpers/badge.svg)](https://docs.rs/hwpers)
[![CI](https://github.com/Indosaram/hwpers/workflows/CI/badge.svg)](https://github.com/Indosaram/hwpers/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A Rust library for parsing Korean Hangul Word Processor (HWP) files with full layout rendering support.

## Features

### Parser (Reading HWP files)
- **Complete HWP 5.0 Format Support**: Parse all document components including text, formatting, tables, and embedded objects
- **Visual Layout Rendering**: Reconstruct documents with pixel-perfect accuracy when layout data is available
- **Font and Style Preservation**: Extract and apply original fonts, sizes, colors, and text formatting
- **Advanced Layout Engine**: Support for multi-column layouts, line-by-line positioning, and character-level formatting
- **SVG Export**: Render documents to scalable vector graphics
- **Zero-copy Parsing**: Efficient parsing with minimal memory allocation
- **Safe Rust**: Memory-safe implementation with comprehensive error handling

### Writer (Creating HWP files) - v0.2.0+
- **Basic Document Creation**: Create simple HWP documents with text content
- **Paragraph Formatting**: Text alignment, line spacing, paragraph spacing
- **Page Layout**: Custom page sizes, margins, orientation
- **Hyperlinks**: URL, email, file, and bookmark links (partial support)
- **Header/Footer**: Basic header and footer support (limited functionality)

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
hwpers = "0.1"
```

### Basic Usage

```rust
use hwpers::HwpReader;

// Parse an HWP file
let document = HwpReader::from_file("document.hwp")?;

// Extract text content
let text = document.extract_text();
println!("{}", text);

// Access document properties
if let Some(props) = document.get_properties() {
    println!("Pages: {}", props.total_page_count);
}

// Iterate through sections and paragraphs
for (i, section) in document.sections().enumerate() {
    println!("Section {}: {} paragraphs", i, section.paragraphs.len());
    
    for paragraph in &section.paragraphs {
        if let Some(text) = &paragraph.text {
            println!("  {}", text.content);
        }
    }
}
```

### Visual Layout Rendering

```rust
use hwpers::{HwpReader, render::{HwpRenderer, RenderOptions}};

let document = HwpReader::from_file("document.hwp")?;

// Create renderer with custom options
let options = RenderOptions {
    dpi: 96,
    scale: 1.0,
    show_margins: false,
    show_baselines: false,
};

let renderer = HwpRenderer::new(&document, options);
let result = renderer.render();

// Export first page to SVG
if let Some(svg) = result.to_svg(0) {
    std::fs::write("page1.svg", svg)?;
}

println!("Rendered {} pages", result.pages.len());
```

### Advanced Formatting Access

```rust
// Access character and paragraph formatting
for section in document.sections() {
    for paragraph in &section.paragraphs {
        // Get paragraph formatting
        if let Some(para_shape) = document.get_para_shape(paragraph.para_shape_id as usize) {
            println!("Indent: {}, Alignment: {}", 
                para_shape.indent, 
                para_shape.get_alignment()
            );
        }
        
        // Get character formatting runs
        if let Some(char_shapes) = &paragraph.char_shapes {
            for pos_shape in &char_shapes.char_positions {
                if let Some(char_shape) = document.get_char_shape(pos_shape.char_shape_id as usize) {
                    println!("Position {}: Size {}, Bold: {}", 
                        pos_shape.position,
                        char_shape.base_size / 100,
                        char_shape.is_bold()
                    );
                }
            }
        }
    }
}
```

## Supported Features

### Document Structure
- ✅ File header and version detection
- ✅ Document properties and metadata
- ✅ Section definitions and page layout
- ✅ Paragraph and character formatting
- ✅ Font definitions (FaceName)
- ✅ Styles and templates

### Content Types
- ✅ Text content with full Unicode support
- ✅ Tables and structured data
- ✅ Control objects (images, OLE objects)
- ✅ Numbering and bullet lists
- ✅ Tab stops and alignment

### Layout and Rendering
- ✅ Page dimensions and margins
- ✅ Multi-column layouts
- ✅ Line-by-line positioning (when available)
- ✅ Character-level positioning (when available)
- ✅ Borders and fill patterns
- ✅ SVG export with accurate positioning

### Advanced Features
- ✅ Compressed document support
- ✅ CFB (Compound File Binary) format handling
- ✅ Multiple encoding support (UTF-16LE)
- ✅ Error recovery and partial parsing

## Command Line Tool

The library includes a command-line tool for inspecting HWP files:

```bash
# Install the tool
cargo install hwpers

# Inspect an HWP file
hwp_info document.hwp
```

## Format Support

This library supports HWP 5.0 format files. For older HWP formats, consider using format conversion tools first.

## Writer Limitations (v0.2.0)

The HWP writer functionality is currently in early development with several limitations:

### ⚠️ Partially Implemented
- **Hyperlinks**: Basic structure implemented, but position tracking within paragraphs needs refinement
- **Header/Footer**: 40-byte structure implemented, but text content storage mechanism incomplete
- **Page Layout**: Basic settings work, but multi-column layouts not supported
- **Styles**: Currently uses hardcoded style IDs; proper style management system needed

### ❌ Not Yet Implemented
- **Images**: Control structure exists but BinData stream integration missing - images won't display
- **Tables**: Table creation and formatting not implemented
- **Lists/Numbering**: Bullet points and numbered lists not supported
- **Text Boxes**: Text box controls not implemented
- **Shapes/Drawing**: Shape and drawing objects not supported
- **Advanced Formatting**: Character styles, fonts, colors need proper style manager
- **Document Properties**: Metadata and document properties not fully implemented

### 🔧 Known Issues
- Generated files may not open correctly in some versions of Hanword
- Style IDs are hardcoded and may conflict with document defaults
- Position tracking for hyperlinks within paragraphs is imprecise
- No compression support for writer (reader supports both compressed and uncompressed)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- HWP file format specification by Hancom Inc.
- Korean text processing community
- Rust parsing and document processing ecosystem