use super::super::super::App;
use arboard::Clipboard;
use std::path::PathBuf;

impl App {
    pub fn paste_from_clipboard(&mut self) {
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(text) => {
                    let trimmed = text.trim();

                    // Check if it's a file path
                    if trimmed.starts_with('/')
                        || trimmed.starts_with("~/")
                        || trimmed.starts_with("./")
                        || trimmed.starts_with("file://")
                    {
                        // Try to load as file
                        let path = if trimmed.starts_with("file://") {
                            PathBuf::from(trimmed.strip_prefix("file://").unwrap_or(trimmed))
                        } else if trimmed.starts_with("~/") {
                            if let Ok(home) = std::env::var("HOME") {
                                PathBuf::from(trimmed.replacen("~/", &format!("{}/", home), 1))
                            } else {
                                PathBuf::from(trimmed)
                            }
                        } else {
                            PathBuf::from(trimmed)
                        };
                        self.load_file(path);
                    }
                    // For Markdown files, check if it looks like Markdown content
                    else if self.is_markdown_file() && (trimmed.contains("## INSIDE") || trimmed.contains("## OUTSIDE") || trimmed.starts_with("### ")) {
                        self.markdown_input = text;
                        match self.parse_markdown(&self.markdown_input) {
                            Ok(json_content) => {
                                self.json_input = json_content;
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("Pasted Markdown content");
                            }
                            Err(e) => {
                                self.set_status(&format!("Failed to parse Markdown: {}", e));
                            }
                        }
                    }
                    // Check if it looks like JSON
                    else if trimmed.starts_with('{') || trimmed.starts_with('[') {
                        self.json_input = text;
                        self.is_modified = true;
                        self.sync_markdown_from_json();
                        self.sync_toon_from_json();
                        self.set_status("Pasted JSON content");
                        self.convert_json();
                    }
                    // Try to parse as Toon format
                    else if let Ok(json_content) = self.parse_toon(&text) {
                        if self.is_toon_file() {
                            self.toon_input = text;
                        }
                        self.json_input = json_content;
                        self.is_modified = true;
                        self.sync_markdown_from_json();
                        self.sync_toon_from_json();
                        self.set_status("Pasted Toon content");
                        self.convert_json();
                    }
                    // Ignore status messages and other non-JSON text
                    else {
                        self.set_status("Clipboard doesn't contain JSON, Markdown, Toon, or file path");
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_from_text(&mut self, text: &str) {
        let trimmed = text.trim();

        // For Markdown files, check if it looks like Markdown content
        if self.is_markdown_file() && (trimmed.contains("## INSIDE") || trimmed.contains("## OUTSIDE") || trimmed.starts_with("### ")) {
            self.markdown_input = text.to_string();
            match self.parse_markdown(&self.markdown_input) {
                Ok(json_content) => {
                    self.json_input = json_content;
                    self.is_modified = true;
                    self.sync_toon_from_json();
                    self.convert_json();
                    self.set_status("Pasted Markdown content");
                }
                Err(e) => {
                    self.set_status(&format!("Failed to parse Markdown: {}", e));
                }
            }
        }
        // Check if it looks like JSON
        else if trimmed.starts_with('{') || trimmed.starts_with('[') {
            self.json_input = text.to_string();
            self.is_modified = true;
            self.sync_toon_from_json();
            self.sync_markdown_from_json();
            self.convert_json();
            self.set_status("Pasted JSON content");
        }
        else {
            self.set_status("Content doesn't contain JSON or Markdown");
        }
    }
}
