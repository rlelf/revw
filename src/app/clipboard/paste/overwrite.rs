use super::super::super::App;
use arboard::Clipboard;
use serde_json::Value;

impl App {
    pub fn paste_inside_overwrite(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // For Markdown files, check if clipboard contains JSON, Toon, or Markdown
                    if self.is_markdown_file() {
                        let trimmed = clipboard_text.trim();

                        // Try to parse as JSON first
                        if trimmed.starts_with('{') || trimmed.starts_with('[') {
                            if let Ok(clipboard_json) = serde_json::from_str::<Value>(&clipboard_text) {
                                // Convert JSON to Markdown
                                if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                    self.paste_markdown_section_overwrite(&md_text, "INSIDE");
                                    return;
                                }
                            }
                        }

                        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
                            self.paste_markdown_section_overwrite(&clipboard_text, "INSIDE");
                            return;
                        }

                        // Try to parse as Toon
                        if let Ok(clipboard_json) = self.clipboard_text_to_json_value(&clipboard_text) {
                            if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                self.paste_markdown_section_overwrite(&md_text, "INSIDE");
                                return;
                            }
                        }

                        // Otherwise treat as Markdown
                        self.paste_markdown_section_overwrite(&clipboard_text, "INSIDE");
                        return;
                    }

                    // For Toon files, parse Toon or JSON format
                    if self.is_toon_file() {
                        match self.clipboard_text_to_json_value(&clipboard_text) {
                            Ok(clipboard_json) => {
                                let new_inside = if let Some(obj) = clipboard_json.as_object() {
                                    obj.get("inside").cloned()
                                } else {
                                    None
                                };

                                if let Some(new_inside) = new_inside {
                                    match serde_json::from_str::<Value>(&self.json_input) {
                                        Ok(mut current_json) => {
                                            if let Some(obj) = current_json.as_object_mut() {
                                                obj.insert("inside".to_string(), new_inside);

                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        if let Err(e) = self.set_json_and_sync_toon(formatted) {
                                                            self.set_status(&e);
                                                            return;
                                                        }
                                                        self.set_status("INSIDE section overwritten from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current JSON is not an object");
                                            }
                                        }
                                        Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                    }
                                } else {
                                    self.set_status("No 'inside' field in clipboard JSON");
                                }
                            }
                            Err(e) => self.set_status(&e),
                        }
                        return;
                    }

                    // For JSON files, parse JSON format
                    // Try to parse as JSON
                    match self.clipboard_text_to_json_value(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "inside" array from clipboard
                            let new_inside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("inside").cloned()
                            } else {
                                None
                            };

                            if let Some(new_inside) = new_inside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Overwrite inside
                                            obj.insert("inside".to_string(), new_inside);

                                            // Format and save
                                            match serde_json::to_string_pretty(&current_json) {
                                                Ok(formatted) => {
                                                    self.json_input = formatted;
                                                    self.is_modified = true;
                                                    self.sync_toon_from_json();
                                                    self.sync_markdown_from_json();
                                                    self.convert_json();
                                                    self.set_status("INSIDE section overwritten from clipboard");
                                                }
                                                Err(e) => self.set_status(&format!("Format error: {}", e)),
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'inside' field in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&e),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_outside_overwrite(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // For Markdown files, check if clipboard contains JSON, Toon, or Markdown
                    if self.is_markdown_file() {
                        let trimmed = clipboard_text.trim();

                        // Try to parse as JSON first
                        if trimmed.starts_with('{') || trimmed.starts_with('[') {
                            if let Ok(clipboard_json) = serde_json::from_str::<Value>(&clipboard_text) {
                                // Convert JSON to Markdown
                                if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                    self.paste_markdown_section_overwrite(&md_text, "OUTSIDE");
                                    return;
                                }
                            }
                        }

                        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
                            self.paste_markdown_section_overwrite(&clipboard_text, "OUTSIDE");
                            return;
                        }

                        // Try to parse as Toon
                        if let Ok(clipboard_json) = self.clipboard_text_to_json_value(&clipboard_text) {
                            if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                self.paste_markdown_section_overwrite(&md_text, "OUTSIDE");
                                return;
                            }
                        }

                        // Otherwise treat as Markdown
                        self.paste_markdown_section_overwrite(&clipboard_text, "OUTSIDE");
                        return;
                    }

                    // For Toon files, parse Toon or JSON format
                    if self.is_toon_file() {
                        match self.clipboard_text_to_json_value(&clipboard_text) {
                            Ok(clipboard_json) => {
                                let new_outside = if let Some(obj) = clipboard_json.as_object() {
                                    obj.get("outside").cloned()
                                } else {
                                    None
                                };

                                if let Some(new_outside) = new_outside {
                                    match serde_json::from_str::<Value>(&self.json_input) {
                                        Ok(mut current_json) => {
                                            if let Some(obj) = current_json.as_object_mut() {
                                                obj.insert("outside".to_string(), new_outside);

                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        if let Err(e) = self.set_json_and_sync_toon(formatted) {
                                                            self.set_status(&e);
                                                            return;
                                                        }
                                                        self.set_status("OUTSIDE section overwritten from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current JSON is not an object");
                                            }
                                        }
                                        Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                    }
                                } else {
                                    self.set_status("No 'outside' field in clipboard JSON");
                                }
                            }
                            Err(e) => self.set_status(&e),
                        }
                        return;
                    }

                    // For JSON files, parse JSON format
                    // Try to parse as JSON
                    match self.clipboard_text_to_json_value(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "outside" array from clipboard
                            let new_outside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("outside").cloned()
                            } else {
                                None
                            };

                            if let Some(new_outside) = new_outside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Overwrite outside
                                            obj.insert("outside".to_string(), new_outside);

                                            // Format and save
                                            match serde_json::to_string_pretty(&current_json) {
                                                Ok(formatted) => {
                                                    self.json_input = formatted;
                                                    self.is_modified = true;
                                                    self.sync_toon_from_json();
                                                    self.sync_markdown_from_json();
                                                    self.convert_json();
                                                    self.set_status("OUTSIDE section overwritten from clipboard");
                                                }
                                                Err(e) => self.set_status(&format!("Format error: {}", e)),
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'outside' field in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&e),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    /// Helper function to paste Markdown section content (INSIDE or OUTSIDE) from clipboard (overwrite)
    pub(super) fn paste_markdown_section_overwrite(&mut self, clipboard_text: &str, section: &str) {
        // Parse the clipboard content to extract the section
        let lines: Vec<&str> = clipboard_text.lines().collect();
        let mut section_lines = Vec::new();
        let mut in_target_section = false;
        let section_header = format!("## {}", section);

        for line in lines {
            let trimmed = line.trim();

            // Check if we're entering the target section
            if trimmed == section_header.trim() {
                in_target_section = true;
                continue;
            }

            // Check if we're entering a different section
            if trimmed.starts_with("## ") && trimmed != section_header.trim() {
                in_target_section = false;
                continue;
            }

            // Collect lines from target section
            if in_target_section {
                section_lines.push(line);
            }
        }

        if section_lines.is_empty() {
            self.set_status(&format!("No {} section found in clipboard", section));
            return;
        }

        // Now replace the section in the current markdown file
        let current_lines: Vec<&str> = self.markdown_input.lines().collect();
        let mut result_lines = Vec::new();
        let mut in_section_to_replace = false;
        let mut found_section = false;

        for line in current_lines {
            let trimmed = line.trim();

            // Found target section header
            if trimmed == section_header.trim() {
                found_section = true;
                in_section_to_replace = true;
                result_lines.push(line.to_string());
                result_lines.push("".to_string());
                // Insert new content
                for section_line in &section_lines {
                    result_lines.push(section_line.to_string());
                }
                continue;
            }

            // Check if we're entering a different section (end of section to replace)
            if in_section_to_replace && trimmed.starts_with("## ") {
                in_section_to_replace = false;
                result_lines.push(line.to_string());
                continue;
            }

            // Skip lines in the section we're replacing
            if in_section_to_replace {
                continue;
            }

            result_lines.push(line.to_string());
        }

        // If section not found, create it
        if !found_section {
            result_lines.push("".to_string());
            result_lines.push(section_header);
            result_lines.push("".to_string());
            for section_line in &section_lines {
                result_lines.push(section_line.to_string());
            }
        }

        self.markdown_input = result_lines.join("\n");

        // Re-parse markdown to update JSON
        match self.parse_markdown(&self.markdown_input) {
            Ok(json_content) => {
                self.json_input = json_content;
                self.is_modified = true;
                self.convert_json();
                self.set_status(&format!("{} section overwritten from clipboard", section));
            }
            Err(e) => {
                self.set_status(&format!("Failed to parse markdown: {}", e));
            }
        }
    }
}
