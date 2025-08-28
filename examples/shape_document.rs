use hwpers::{
    HwpWriter, 
    model::shape::*
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = HwpWriter::new();

    // Set document metadata
    writer
        .set_document_title("Shape Drawing Demo")
        .set_document_author("HWP Writer")
        .set_document_subject("Demonstrating Shape Drawing Capabilities");

    // Add header
    writer.add_header("Shape Drawing Features Demo");

    // Add title
    writer.add_heading("Shape Drawing Examples", 1)?;

    // Add introduction paragraph
    writer.add_paragraph("This document demonstrates the shape drawing capabilities of the HWP writer, including basic shapes, custom styling, text within shapes, and shape grouping.")?;

    // Example 1: Basic Shapes
    writer.add_heading("1. Basic Shapes", 2)?;
    writer.add_paragraph("The following are basic geometric shapes with default styling:")?;

    // Add basic shapes in a row
    writer.add_rectangle(20.0, 40.0, 30.0, 20.0)?; // Rectangle
    writer.add_circle(70.0, 50.0, 12.0)?;          // Circle
    writer.add_ellipse(100.0, 40.0, 25.0, 20.0)?;  // Ellipse

    writer.add_paragraph("From left to right: Rectangle (30×20mm), Circle (12mm radius), Ellipse (25×20mm)")?;

    // Example 2: Lines and Arrows
    writer.add_heading("2. Lines and Arrows", 2)?;
    writer.add_paragraph("Various line styles and arrow types:")?;

    // Add different line types
    writer.add_line(20.0, 80.0, 60.0, 80.0)?;         // Solid line
    writer.add_dashed_line(20.0, 90.0, 60.0, 90.0)?;  // Dashed line
    writer.add_arrow(70.0, 80.0, 110.0, 90.0)?;       // Arrow

    writer.add_paragraph("Solid line, dashed line, and arrow demonstrating different stroke styles.")?;

    // Example 3: Custom Styled Shapes
    writer.add_heading("3. Custom Styling", 2)?;
    writer.add_paragraph("Shapes with custom colors, fills, and stroke properties:")?;

    // Create custom styled shapes
    let red_fill = ShapeFill::solid(0xFF4444);
    let thick_stroke = ShapeStroke::new(LineStyle::Solid, 0x000000, 2.0);
    writer.add_custom_shape(ShapeType::Rectangle, 20.0, 120.0, 25.0, 15.0, red_fill, thick_stroke)?;

    let gradient_fill = ShapeFill::gradient(0x4444FF, 0xFF44FF, 45.0);
    let dashed_stroke = ShapeStroke::new(LineStyle::Dash, 0x00AA00, 1.5);
    writer.add_custom_shape(ShapeType::Ellipse, 55.0, 120.0, 30.0, 15.0, gradient_fill, dashed_stroke)?;

    let transparent_fill = ShapeFill::transparent();
    let arrow_stroke = ShapeStroke::arrow(0x0000FF, 2.0);
    writer.add_custom_shape(ShapeType::Line, 95.0, 120.0, 30.0, 15.0, transparent_fill, arrow_stroke)?;

    writer.add_paragraph("Red rectangle with thick border, gradient ellipse with dashed green border, blue arrow line.")?;

    // Example 4: Shapes with Text
    writer.add_heading("4. Shapes with Text", 2)?;
    writer.add_paragraph("Shapes can contain text with different alignments:")?;

    writer.add_shape_with_text(
        ShapeType::Rectangle,
        20.0, 150.0, 40.0, 20.0,
        "Centered".to_string(),
        ShapeAlignment::Center
    )?;

    writer.add_shape_with_text(
        ShapeType::Ellipse,
        70.0, 150.0, 40.0, 20.0,
        "Left Aligned".to_string(),
        ShapeAlignment::Left
    )?;

    writer.add_shape_with_text(
        ShapeType::Rectangle,
        120.0, 150.0, 40.0, 20.0,
        "Right".to_string(),
        ShapeAlignment::Right
    )?;

    writer.add_paragraph("Text can be centered, left-aligned, or right-aligned within shapes.")?;

    // Example 5: Polygons
    writer.add_heading("5. Polygon Shapes", 2)?;
    writer.add_paragraph("Custom polygons created from point coordinates:")?;

    // Create a triangle
    let triangle_points = vec![
        (30.0, 180.0),  // Top point
        (20.0, 200.0),  // Bottom left
        (40.0, 200.0),  // Bottom right
    ];
    writer.add_polygon(triangle_points)?;

    // Create a pentagon
    let pentagon_points = vec![
        (70.0, 180.0),  // Top
        (80.0, 185.0),  // Top right
        (75.0, 195.0),  // Bottom right
        (65.0, 195.0),  // Bottom left
        (60.0, 185.0),  // Top left
    ];
    writer.add_polygon(pentagon_points)?;

    // Create a star (more complex polygon)
    let star_points = vec![
        (110.0, 180.0), // Top
        (113.0, 187.0), // Inner right
        (120.0, 187.0), // Outer right
        (115.0, 192.0), // Inner bottom right
        (117.0, 200.0), // Bottom right
        (110.0, 196.0), // Inner bottom
        (103.0, 200.0), // Bottom left
        (105.0, 192.0), // Inner bottom left
        (100.0, 187.0), // Outer left
        (107.0, 187.0), // Inner left
    ];
    writer.add_polygon(star_points)?;

    writer.add_paragraph("Triangle, pentagon, and star polygons created from custom point arrays.")?;

    // Example 6: Shape Groups
    writer.add_heading("6. Shape Groups", 2)?;
    writer.add_paragraph("Multiple shapes can be grouped together and transformed as a unit:")?;

    // Create individual shapes for grouping
    let group_rect = Shape::rectangle_mm(20.0, 220.0, 25.0, 15.0)
        .with_fill(ShapeFill::solid(0xFF6666))
        .with_stroke(ShapeStroke::new(LineStyle::Solid, 0x000000, 1.0))
        .with_name("Group Rectangle".to_string());

    let group_circle = Shape::circle_mm(60.0, 227.0, 8.0)
        .with_fill(ShapeFill::solid(0x66FF66))
        .with_stroke(ShapeStroke::new(LineStyle::Solid, 0x000000, 1.0))
        .with_name("Group Circle".to_string());

    let group_line = Shape::line_mm(25.0, 240.0, 55.0, 240.0)
        .with_stroke(ShapeStroke::new(LineStyle::Dash, 0x6666FF, 1.5))
        .with_name("Group Line".to_string());

    // Create and add the group
    let shape_group = writer.create_shape_group("Example Group")
        .add_shape(group_rect)
        .add_shape(group_circle)
        .add_shape(group_line)
        .with_transform(0, 0, 0.0, 1.0, 1.0)
        .finish();

    writer.add_shape_group(shape_group)?;

    writer.add_paragraph("A group containing a red rectangle, green circle, and blue dashed line. Groups can be moved, rotated, and scaled together.")?;

    // Example 7: Complex Composition
    writer.add_heading("7. Complex Shape Composition", 2)?;
    writer.add_paragraph("Combining multiple shape types to create complex diagrams:")?;

    // Create a simple flowchart-like diagram
    
    // Start node (circle)
    writer.add_shape_with_text(
        ShapeType::Ellipse,
        30.0, 270.0, 20.0, 15.0,
        "Start".to_string(),
        ShapeAlignment::Center
    )?;

    // Arrow to process
    writer.add_arrow(40.0, 277.0, 55.0, 277.0)?;

    // Process node (rectangle)
    writer.add_shape_with_text(
        ShapeType::Rectangle,
        55.0, 270.0, 30.0, 15.0,
        "Process".to_string(),
        ShapeAlignment::Center
    )?;

    // Arrow to decision
    writer.add_arrow(70.0, 285.0, 70.0, 295.0)?;

    // Decision node (diamond-like polygon)
    let diamond_points = vec![
        (70.0, 295.0),  // Top
        (80.0, 305.0),  // Right
        (70.0, 315.0),  // Bottom
        (60.0, 305.0),  // Left
    ];
    writer.add_polygon(diamond_points)?;
    
    // Decision text (separate text box)
    writer.add_styled_text_box("Decision?", "info")?;

    // End node (circle)
    writer.add_shape_with_text(
        ShapeType::Ellipse,
        100.0, 300.0, 20.0, 15.0,
        "End".to_string(),
        ShapeAlignment::Center
    )?;

    // Arrow to end
    writer.add_arrow(80.0, 305.0, 100.0, 307.0)?;

    writer.add_paragraph("A simple flowchart demonstrating how shapes can be combined to create diagrams and visual workflows.")?;

    // Example 8: Technical Specifications
    writer.add_heading("8. Technical Specifications", 2)?;
    writer.add_paragraph("All measurements and technical details:")?;

    // Create a technical specification table
    let specs_table = writer.add_table(8, 3)?
        .set_cell(0, 0, "Shape Type")
        .set_cell(0, 1, "Features")
        .set_cell(0, 2, "Use Cases")
        .set_cell(1, 0, "Rectangle")
        .set_cell(1, 1, "Width, Height, Fill, Stroke")
        .set_cell(1, 2, "Boxes, Frames, Backgrounds")
        .set_cell(2, 0, "Circle")
        .set_cell(2, 1, "Radius, Fill, Stroke")
        .set_cell(2, 2, "Buttons, Indicators, Highlights")
        .set_cell(3, 0, "Ellipse")
        .set_cell(3, 1, "Width, Height, Fill, Stroke")
        .set_cell(3, 2, "Oval frames, Speech bubbles")
        .set_cell(4, 0, "Line")
        .set_cell(4, 1, "Points, Stroke, End caps")
        .set_cell(4, 2, "Connectors, Dividers, Arrows")
        .set_cell(5, 0, "Polygon")
        .set_cell(5, 1, "Custom points, Fill, Stroke")
        .set_cell(5, 2, "Stars, Triangles, Custom shapes")
        .set_cell(6, 0, "Group")
        .set_cell(6, 1, "Multiple shapes, Transform")
        .set_cell(6, 2, "Complex objects, Reusable components")
        .set_cell(7, 0, "Text Shape")
        .set_cell(7, 1, "Shape + Text + Alignment")
        .set_cell(7, 2, "Labels, Captions, Annotations");
    specs_table.finish()?;

    // Example 9: Color and Style Guide
    writer.add_heading("9. Color and Style Guide", 2)?;
    writer.add_paragraph("Available colors, gradients, and stroke styles:")?;

    // Color palette
    let colors = [
        ("Red", 0xFF0000),
        ("Green", 0x00FF00),
        ("Blue", 0x0000FF),
        ("Yellow", 0xFFFF00),
        ("Cyan", 0x00FFFF),
        ("Magenta", 0xFF00FF),
    ];

    for (i, (name, color)) in colors.iter().enumerate() {
        let x = 20.0 + (i as f32 * 25.0);
        let fill = ShapeFill::solid(*color);
        let stroke = ShapeStroke::new(LineStyle::Solid, 0x000000, 1.0);
        writer.add_custom_shape(ShapeType::Rectangle, x, 350.0, 20.0, 15.0, fill, stroke)?;
        
        // Add color name below
        writer.add_shape_with_text(
            ShapeType::Rectangle,
            x, 370.0, 20.0, 10.0,
            name.to_string(),
            ShapeAlignment::Center
        )?;
    }

    writer.add_paragraph("Color palette showing primary and secondary colors available for shape fills.")?;

    // Stroke styles demonstration
    let stroke_styles = [
        ("Solid", LineStyle::Solid),
        ("Dash", LineStyle::Dash),
        ("Dot", LineStyle::Dot),
        ("DashDot", LineStyle::DashDot),
    ];

    for (i, (name, style)) in stroke_styles.iter().enumerate() {
        let y = 390.0 + (i as f32 * 10.0);
        let stroke = ShapeStroke::new(*style, 0x000000, 1.5);
        let line = Shape::line_mm(20.0, y, 80.0, y).with_stroke(stroke);
        writer.add_shape(line)?;
        
        // Add style name
        writer.add_shape_with_text(
            ShapeType::Rectangle,
            85.0, y - 5.0, 25.0, 8.0,
            name.to_string(),
            ShapeAlignment::Left
        )?;
    }

    writer.add_paragraph("Different stroke styles: solid, dashed, dotted, and dash-dot patterns.")?;

    // Summary
    writer.add_heading("Summary", 2)?;
    writer.add_paragraph("This document has demonstrated the comprehensive shape drawing capabilities of the HWP writer, including basic geometric shapes, custom styling with colors and gradients, text within shapes, polygon creation, shape grouping, and complex compositions. These features enable the creation of technical diagrams, flowcharts, illustrations, and visual documentation.")?;

    // Add footer with page numbers
    writer.add_footer_with_page_number("Page", hwpers::model::PageNumberFormat::Numeric);

    // Update statistics and save
    writer.update_document_statistics();
    writer.save_to_file("shape_document.hwp")?;

    println!("Shape drawing demo document created: shape_document.hwp");
    
    // Show final statistics
    if let Some(stats) = writer.get_document_statistics() {
        println!("Document statistics:");
        println!("  - Characters: {}", stats.total_character_count);
        println!("  - Words: {}", stats.total_word_count);
        println!("  - Lines: {}", stats.line_count);
        println!("  - Pages: {}", stats.total_page_count);
    }

    // Count shapes in the document
    let document = writer.document();
    let total_shapes: usize = document.body_texts[0].sections[0]
        .paragraphs
        .iter()
        .map(|p| p.shapes.len())
        .sum();
    
    let total_groups: usize = document.body_texts[0].sections[0]
        .paragraphs
        .iter()
        .map(|p| p.shape_groups.len())
        .sum();

    println!("Shape content:");
    println!("  - Total shapes: {}", total_shapes);
    println!("  - Shape groups: {}", total_groups);
    println!("  - Total paragraphs: {}", document.body_texts[0].sections[0].paragraphs.len());

    Ok(())
}