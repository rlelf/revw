use super::App;
use printpdf::*;
use std::fs;
use std::path::PathBuf;

/// Wrap text to fit within a specified width
/// Returns a vector of text chunks that should each fit on one line
fn wrap_text(text: &str, max_width_mm: f32, font_size_pt: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    // Rough approximation: 1 pt ≈ 0.35mm width per character for proportional fonts
    // For Japanese/CJK characters, use slightly wider estimate
    let avg_char_width_mm = font_size_pt * 0.2;
    let max_chars_per_line = (max_width_mm / avg_char_width_mm) as usize;

    if max_chars_per_line == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for ch in text.chars() {
        let char_width = if ch.is_ascii() { 1 } else { 2 }; // CJK chars count as 2

        if current_width + char_width > max_chars_per_line && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line.clear();
            current_width = 0;
        }

        current_line.push(ch);
        current_width += char_width;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}

/// Find a suitable Japanese font on the system
fn find_japanese_font() -> Option<PathBuf> {
    let font_paths = if cfg!(target_os = "windows") {
        vec![
            "C:\\Windows\\Fonts\\msgothic.ttc",     // MS Gothic
            "C:\\Windows\\Fonts\\msmincho.ttc",     // MS Mincho
            "C:\\Windows\\Fonts\\meiryo.ttc",       // Meiryo
            "C:\\Windows\\Fonts\\YuGothM.ttc",      // Yu Gothic Medium
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ]
    } else {
        // Linux
        vec![
            "/usr/share/fonts/truetype/takao-gothic/TakaoPGothic.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/vlgothic/VL-Gothic-Regular.ttf",
            "/usr/share/fonts/truetype/fonts-japanese-gothic.ttf",
        ]
    };

    for path in font_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

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
        let margin_right = Mm(20.0);
        let margin_top = Mm(20.0);
        let line_height = Mm(5.0);
        let max_text_width = page_width.0 - margin_left.0 - margin_right.0;

        // Collect all pages
        let mut all_pages: Vec<PdfPage> = Vec::new();
        let mut current_page_ops: Vec<Op> = Vec::new();
        let mut current_y = page_height - margin_top;

        // Try to load Japanese font from system, fallback to Helvetica if not found
        let font_id = if let Some(font_path) = find_japanese_font() {
            match fs::read(&font_path) {
                Ok(font_bytes) => {
                    let mut warnings = Vec::new();
                    if let Some(parsed_font) = ParsedFont::from_bytes(&font_bytes, 0, &mut warnings) {
                        Some(doc.add_font(&parsed_font))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Parse markdown and render
        for line in markdown_content.lines() {
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
                // Wrap text to fit within page width
                let wrapped_lines = wrap_text(text, max_text_width, font_size);

                for wrapped_line in wrapped_lines {
                    // Check if we need a new page
                    if current_y < Mm(30.0) {
                        // Save current page
                        let page = PdfPage::new(page_width, page_height, current_page_ops.clone());
                        all_pages.push(page);

                        // Reset for new page
                        current_page_ops.clear();
                        current_y = page_height - margin_top;
                    }

                    // Each line gets its own text section
                    current_page_ops.push(Op::SaveGraphicsState);
                    current_page_ops.push(Op::StartTextSection);

                    // Set cursor position
                    current_page_ops.push(Op::SetTextCursor {
                        pos: Point::new(margin_left, current_y),
                    });

                    if let Some(ref fid) = font_id {
                        // Use custom font (Japanese)
                        current_page_ops.push(Op::SetFontSize {
                            size: Pt(font_size),
                            font: fid.clone(),
                        });

                        current_page_ops.push(Op::WriteText {
                            items: vec![TextItem::Text(wrapped_line.clone())],
                            font: fid.clone(),
                        });
                    } else {
                        // Fallback to built-in Helvetica font
                        let font = BuiltinFont::Helvetica;
                        current_page_ops.push(Op::SetFontSizeBuiltinFont {
                            size: Pt(font_size),
                            font: font.clone(),
                        });

                        current_page_ops.push(Op::WriteTextBuiltinFont {
                            items: vec![TextItem::Text(wrapped_line.clone())],
                            font: font.clone(),
                        });
                    }

                    current_page_ops.push(Op::EndTextSection);
                    current_page_ops.push(Op::RestoreGraphicsState);

                    current_y = current_y - line_height;
                }
            } else {
                current_y = current_y - line_height;
            }
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
