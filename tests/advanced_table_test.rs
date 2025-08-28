use hwpers::{HwpWriter, writer::style::{BorderLineStyle, CellBorderStyle}};

#[test]
fn test_table_with_cell_merge() {
    let mut writer = HwpWriter::new();
    
    // Create a 3x3 table with cell merging
    let mut table_builder = writer.add_table(3, 3).unwrap();
    
    // Set content
    table_builder = table_builder
        .set_cell(0, 0, "Merged Header")  // This will span 2 columns
        .set_cell(0, 2, "Header 3")
        .set_cell(1, 0, "Row 1 Col 1")
        .set_cell(1, 1, "Row 1 Col 2")
        .set_cell(1, 2, "Row 1 Col 3")
        .set_cell(2, 0, "Row 2 Col 1")
        .set_cell(2, 1, "Row 2 Col 2")
        .set_cell(2, 2, "Row 2 Col 3");
    
    // Merge cells horizontally (header spans 2 columns)
    table_builder = table_builder.merge_cells(0, 0, 1, 2); // row=0, col=0, row_span=1, col_span=2
    
    // Build the table
    table_builder.finish().unwrap();
    
    let document = writer.document();
    
    // Check that table was created
    assert!(!document.body_texts[0].sections[0].paragraphs.is_empty());
    
    // Find the table paragraph
    let table_para = document.body_texts[0].sections[0].paragraphs
        .iter()
        .find(|p| p.table_data.is_some())
        .expect("Table paragraph should exist");
    
    let table = table_para.table_data.as_ref().unwrap();
    
    // Check table dimensions
    assert_eq!(table.rows, 3);
    assert_eq!(table.cols, 3);
    
    // Check that merged cell has correct span
    let merged_cell = table.get_cell(0, 0).unwrap();
    assert_eq!(merged_cell.col_span, 2);
    assert_eq!(merged_cell.row_span, 1);
}

#[test]
fn test_table_with_borders() {
    let mut writer = HwpWriter::new();
    
    // Create a 2x2 table with custom borders
    let mut table_builder = writer.add_table(2, 2).unwrap();
    
    // Set content
    table_builder = table_builder
        .set_cell(0, 0, "A1")
        .set_cell(0, 1, "B1") 
        .set_cell(1, 0, "A2")
        .set_cell(1, 1, "B2");
    
    // Set different border styles
    let thick_border = BorderLineStyle::solid(3);
    let dashed_border = BorderLineStyle::dashed(2).with_color(0xFF0000); // Red dashed
    
    // Set outer borders to thick
    table_builder = table_builder.set_outer_borders(thick_border);
    
    // Set inner borders to dashed red
    table_builder = table_builder.set_inner_borders(dashed_border);
    
    // Build the table
    table_builder.finish().unwrap();
    
    let document = writer.document();
    
    // Check that border fills were created
    assert!(!document.doc_info.border_fills.is_empty());
    
    // Find the table paragraph
    let table_para = document.body_texts[0].sections[0].paragraphs
        .iter()
        .find(|p| p.table_data.is_some())
        .expect("Table paragraph should exist");
    
    let table = table_para.table_data.as_ref().unwrap();
    
    // Check that cells have border fill IDs assigned
    for cell in &table.cells {
        if cell.border_fill_id > 0 {
            // Check that the border fill exists
            assert!(document.doc_info.border_fills.len() >= cell.border_fill_id as usize);
        }
    }
}

#[test]
fn test_complex_table_with_merge_and_borders() {
    let mut writer = HwpWriter::new();
    
    // Create a 4x4 table with complex merging and borders
    let mut table_builder = writer.add_table(4, 4).unwrap();
    
    // Set content
    table_builder = table_builder
        .set_cell(0, 0, "Title")           // Will span 4 columns
        .set_cell(1, 0, "Subtitle A")      // Will span 2 columns
        .set_cell(1, 2, "Subtitle B")      // Will span 2 columns
        .set_cell(2, 0, "Data 1")
        .set_cell(2, 1, "Data 2")
        .set_cell(2, 2, "Data 3")
        .set_cell(2, 3, "Data 4")
        .set_cell(3, 0, "Footer")          // Will span 4 columns
        .set_cell(3, 1, "")
        .set_cell(3, 2, "")
        .set_cell(3, 3, "");
    
    // Merge cells
    table_builder = table_builder
        .merge_cells(0, 0, 1, 4)  // Title row spans all columns
        .merge_cells(1, 0, 1, 2)  // Subtitle A spans 2 columns
        .merge_cells(1, 2, 1, 2)  // Subtitle B spans 2 columns
        .merge_cells(3, 0, 1, 4); // Footer spans all columns
    
    // Set borders
    let thick_black = BorderLineStyle::solid(2);
    let thin_gray = BorderLineStyle::solid(1).with_color(0x808080);
    
    // Different border styles for different areas
    table_builder = table_builder
        .set_outer_borders(thick_black.clone())
        .set_range_border(0, 0, 0, 3, CellBorderStyle::all_borders(thick_black.clone())) // Title row
        .set_range_border(1, 0, 1, 3, CellBorderStyle::all_borders(thin_gray.clone()))   // Subtitle row
        .set_range_border(2, 0, 2, 3, CellBorderStyle::all_borders(thin_gray.clone()))   // Data row
        .set_range_border(3, 0, 3, 3, CellBorderStyle::all_borders(thick_black));        // Footer row
    
    // Build the table
    table_builder.finish().unwrap();
    
    let document = writer.document();
    
    // Check that table was created with correct structure
    let table_para = document.body_texts[0].sections[0].paragraphs
        .iter()
        .find(|p| p.table_data.is_some())
        .expect("Table paragraph should exist");
    
    let table = table_para.table_data.as_ref().unwrap();
    
    // Verify table dimensions
    assert_eq!(table.rows, 4);
    assert_eq!(table.cols, 4);
    
    // Check merges - note: only cells that are not covered by merges should exist
    assert!(table.get_cell(0, 0).is_some()); // Title cell should exist
    assert!(table.get_cell(1, 0).is_some()); // Subtitle A should exist
    assert!(table.get_cell(1, 2).is_some()); // Subtitle B should exist
    assert!(table.get_cell(3, 0).is_some()); // Footer should exist
}

#[test] 
fn test_table_no_borders() {
    let mut writer = HwpWriter::new();
    
    // Create a simple table with no borders
    let table_builder = writer.add_table(2, 2).unwrap()
        .set_cell(0, 0, "A")
        .set_cell(0, 1, "B")
        .set_cell(1, 0, "C")
        .set_cell(1, 1, "D")
        .no_borders(); // Remove all borders
    
    table_builder.finish().unwrap();
    
    let document = writer.document();
    
    // Find the table
    let table_para = document.body_texts[0].sections[0].paragraphs
        .iter()
        .find(|p| p.table_data.is_some())
        .expect("Table paragraph should exist");
    
    let table = table_para.table_data.as_ref().unwrap();
    
    // Check that table exists without borders
    assert_eq!(table.rows, 2);
    assert_eq!(table.cols, 2);
}

#[test]
fn test_vertical_cell_merge() {
    let mut writer = HwpWriter::new();
    
    // Create a 3x2 table with vertical merge
    let table_builder = writer.add_table(3, 2).unwrap()
        .set_cell(0, 0, "Merged Vertical")  // This will span 3 rows
        .set_cell(0, 1, "Header")
        .set_cell(1, 1, "Row 1")
        .set_cell(2, 1, "Row 2")
        .merge_cells(0, 0, 3, 1); // row=0, col=0, row_span=3, col_span=1
    
    table_builder.finish().unwrap();
    
    let document = writer.document();
    
    // Find the table
    let table_para = document.body_texts[0].sections[0].paragraphs
        .iter()
        .find(|p| p.table_data.is_some())
        .expect("Table paragraph should exist");
    
    let table = table_para.table_data.as_ref().unwrap();
    
    // Check that merged cell has correct span
    let merged_cell = table.get_cell(0, 0).unwrap();
    assert_eq!(merged_cell.row_span, 3);
    assert_eq!(merged_cell.col_span, 1);
}