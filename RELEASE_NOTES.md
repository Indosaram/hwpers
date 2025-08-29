# Release Notes - v0.3.0

## 🎉 Introducing HWP Writer

We're excited to announce the release of hwpers v0.3.0, which introduces the ability to **create HWP documents** programmatically! While still in early development, this release marks a significant milestone in making hwpers a complete HWP file manipulation library.

## ✨ What's New

### Document Creation
For the first time, you can now create HWP documents from scratch using Rust:

```rust
use hwpers::writer::HwpWriter;

let mut writer = HwpWriter::new();
writer.add_paragraph("Hello, HWP!")?;
writer.save_to_file("document.hwp")?;
```

### Key Features

#### 📝 Text Formatting
- **Paragraph alignment**: left, center, right, justify
- **Line spacing**: adjustable percentage (50% - 200%)
- **Paragraph spacing**: configurable before/after spacing

#### 🔗 Hyperlinks
- URL links to websites
- Email links with mailto:
- File links to local documents
- Internal bookmarks for navigation

#### 📄 Page Layout
- Custom page sizes (A4, Letter, Legal, or custom dimensions)
- Adjustable margins
- Portrait/landscape orientation

#### 📑 Headers & Footers
- Add headers and footers to documents
- Page numbering with various formats (numeric, roman, alphabetic)

## 📚 Examples

Check out the new examples in the `examples/` directory:
- `complete_feature_demo.rs` - Comprehensive demonstration of all features
- `hyperlink_document.rs` - Working with hyperlinks
- `header_footer_document.rs` - Headers and footers usage
- `alignment_example.rs` - Text alignment options
- `spacing_example.rs` - Line and paragraph spacing

## ⚠️ Known Limitations

This is an early release of the writer functionality. Please be aware of these limitations:

- **Images**: Structure implemented but BinData integration incomplete
- **Tables**: Not yet supported
- **Lists**: Bullet points and numbering not implemented
- **Styles**: Uses hardcoded style IDs (proper style management coming soon)
- **Compression**: Writer creates uncompressed files only

## 🔄 Migration Guide

If you're upgrading from v0.2.0:
- The parser functionality remains unchanged
- New writer features are in the `hwpers::writer` module
- All new types are in respective model modules (`hyperlink`, `header_footer`, `page_layout`)

## 🙏 Acknowledgments

This release was made possible through careful analysis of the HWP 5.0 specification and reverse engineering of actual HWP files. Special thanks to the Rust community for their support.

## 📦 Installation

```toml
[dependencies]
hwpers = "0.3"
```

## 🐛 Bug Reports

If you encounter any issues, please report them at:
https://github.com/Indosaram/hwpers/issues

## 🚀 What's Next

For v0.4.0, we're planning to focus on:
- Complete image support with BinData integration
- Table creation and formatting
- Lists and numbering
- Improved style management system
- Text boxes and shapes

---

Thank you for using hwpers! We're committed to making it the best HWP manipulation library for Rust.