use super::App;
use printpdf::*;
use std::fs;

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
        let mut doc = PdfDocument::new("Revw Export");

        // Page dimensions
        let page_width = Mm(210.0);
        let page_height = Mm(297.0);
        let margin_left = Mm(20.0);
        let margin_top = Mm(20.0);
        let line_height = Mm(5.0);

        // Collect all pages
        let mut all_pages: Vec<PdfPage> = Vec::new();
        let mut current_page_ops: Vec<Op> = Vec::new();
        let mut current_y = page_height - margin_top;

        // Built-in Helvetica font (available in PDF viewers without embedding)
        let font = BuiltinFont::Helvetica;

        // Parse markdown and render
        for line in markdown_content.lines() {
            // Check if we need a new page
            if current_y < Mm(30.0) {
                // Save current page
                let page = PdfPage::new(page_width, page_height, current_page_ops.clone());
                all_pages.push(page);

                // Reset for new page
                current_page_ops.clear();
                current_y = page_height - margin_top;
            }

            let trimmed = line.trim();

            // Determine font size based on markdown syntax
            let font_size = if trimmed.starts_with("## ") {
                16.0
            } else if trimmed.starts_with("### ") {
                14.0
            } else {
                11.0
            };

            // Extract text content (remove markdown syntax)
            let text = if trimmed.starts_with("## ") {
                &trimmed[3..]
            } else if trimmed.starts_with("### ") {
                &trimmed[4..]
            } else {
                trimmed
            };

            if !text.is_empty() {
                // Each line gets its own text section
                current_page_ops.push(Op::SaveGraphicsState);
                current_page_ops.push(Op::StartTextSection);

                // Set cursor position
                current_page_ops.push(Op::SetTextCursor {
                    pos: Point::new(margin_left, current_y),
                });

                // Set font size
                current_page_ops.push(Op::SetFontSizeBuiltinFont {
                    size: Pt(font_size),
                    font: font.clone(),
                });

                // Write text
                current_page_ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(text.to_string())],
                    font: font.clone(),
                });

                current_page_ops.push(Op::EndTextSection);
                current_page_ops.push(Op::RestoreGraphicsState);
            }

            current_y = current_y - line_height;
        }

        // Save final page (no need to end text section since each line handles its own)
        if !current_page_ops.is_empty() {
            let page = PdfPage::new(page_width, page_height, current_page_ops);
            all_pages.push(page);
        }

        // Save PDF with all pages
        let mut _warnings = Vec::new();
        let pdf_bytes = doc
            .with_pages(all_pages)
            .save(&PdfSaveOptions::default(), &mut _warnings);

        // Write to file
        fs::write(&pdf_path, pdf_bytes)
            .map_err(|e| format!("Failed to write PDF file: {}", e))?;

        Ok(pdf_path.to_string_lossy().to_string())
    }
}
