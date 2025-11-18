use super::App;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

impl App {
    /// Export current content to PDF format
    pub fn export_to_pdf(&self) -> Result<String, String> {
        // Determine output path
        let pdf_path = if let Some(current_path) = &self.file_path {
            current_path.with_extension("pdf")
        } else {
            std::path::PathBuf::from("output.pdf")
        };

        // Convert to Markdown first (for consistency)
        let markdown_content = if self.is_markdown_file() {
            self.markdown_input.clone()
        } else {
            // Convert JSON to Markdown
            self.convert_to_markdown()?
        };

        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            "Revw Export",
            Mm(210.0), // A4 width
            Mm(297.0), // A4 height
            "Layer 1",
        );

        // Use built-in font
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| format!("Failed to load font: {}", e))?;

        // Page margins
        let margin_left = Mm(20.0);
        let margin_top = Mm(20.0);
        let page_height = Mm(297.0);

        let mut current_y = page_height - margin_top;
        let line_height = Mm(5.0);
        let mut pages = vec![(page1, layer1)];
        let mut current_page_idx = 0;

        // Parse markdown and render
        for line in markdown_content.lines() {
            // Check if we need a new page
            if current_y < Mm(30.0) {
                // Add new page
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                pages.push((new_page, new_layer));
                current_page_idx += 1;
                current_y = page_height - margin_top;
            }

            let trimmed = line.trim();

            // Determine font size based on markdown syntax
            let (text, font_size) = if trimmed.starts_with("## ") {
                // Section header (OUTSIDE/INSIDE)
                (&trimmed[3..], 16.0)
            } else if trimmed.starts_with("### ") {
                // Entry header
                (&trimmed[4..], 14.0)
            } else if trimmed.starts_with("**") && trimmed.ends_with("**") {
                // Bold text (like **URL:** or **Percentage:**)
                (trimmed, 11.0)
            } else {
                // Regular text
                (trimmed, 11.0)
            };

            if !text.is_empty() {
                let (page_ref, layer_ref) = pages[current_page_idx];
                let current_layer = doc.get_page(page_ref).get_layer(layer_ref);

                current_layer.use_text(
                    text,
                    font_size,
                    margin_left,
                    current_y,
                    &font,
                );
            }

            current_y -= line_height;
        }

        // Save PDF
        let file = File::create(&pdf_path)
            .map_err(|e| format!("Failed to create PDF file: {}", e))?;
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer)
            .map_err(|e| format!("Failed to save PDF: {}", e))?;

        Ok(pdf_path.to_string_lossy().to_string())
    }
}
