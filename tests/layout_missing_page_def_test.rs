use hwpers::render::layout::LayoutEngine;
use hwpers::HwpReader;
use std::path::PathBuf;

/// A section without a PageDef record must not panic during layout.
/// `styled_document.hwp` (shipped in the repo) parses into a section whose
/// `page_def` is `None`, which previously triggered a panic in `layout_section`.
#[test]
fn test_layout_section_without_page_def_does_not_panic() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("styled_document.hwp");
    let bytes = std::fs::read(&path).expect("failed to read styled_document.hwp");
    let doc = HwpReader::from_bytes(&bytes).expect("failed to parse styled_document.hwp");

    assert!(
        doc.sections().any(|s| s.page_def.is_none()),
        "expected at least one section without a page definition"
    );

    let engine = LayoutEngine::new(&doc);
    let result = engine.calculate_layout();
    assert!(result.pages.is_empty());
}
