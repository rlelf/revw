use super::super::super::App;
use arboard::Clipboard;
use serde_json::Value;

impl App {
    pub fn paste_inside_append(&mut self) {
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
                                    self.paste_markdown_section_append(&md_text, "INSIDE");
                                    return;
                                }
                            }
                        }

                        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
                            self.paste_markdown_section_append(&clipboard_text, "INSIDE");
                            return;
                        }

                        // Try to parse as Toon
                        if let Ok(clipboard_json) = self.clipboard_text_to_json_value(&clipboard_text) {
                            if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                self.paste_markdown_section_append(&md_text, "INSIDE");
                                return;
                            }
                        }

                        // Otherwise treat as Markdown
                        self.paste_markdown_section_append(&clipboard_text, "INSIDE");
                        return;
                    }

                    // For Toon files, parse Toon or JSON format
                    if self.is_toon_file() {
                        match self.clipboard_text_to_json_value(&clipboard_text) {
                            Ok(clipboard_json) => {
                                let new_inside = if let Some(obj) = clipboard_json.as_object() {
                                    obj.get("inside").and_then(|v| v.as_array()).cloned()
                                } else {
                                    None
                                };

                                if let Some(new_inside_items) = new_inside {
                                    match serde_json::from_str::<Value>(&self.json_input) {
                                        Ok(mut current_json) => {
                                            if let Some(obj) = current_json.as_object_mut() {
                                                let inside_array = obj.entry("inside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = inside_array.as_array_mut() {
                                                    for (idx, item) in new_inside_items.into_iter().enumerate() {
                                                        arr.insert(idx, item);
                                                    }

                                                    match serde_json::to_string_pretty(&current_json) {
                                                        Ok(formatted) => {
                                                            if let Err(e) = self.set_json_and_sync_toon(formatted) {
                                                                self.set_status(&e);
                                                                return;
                                                            }
                                                            self.set_status("INSIDE entries inserted at top from clipboard");
                                                        }
                                                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                    }
                                                } else {
                                                    self.set_status("Current 'inside' is not an array");
                                                }
                                            } else {
                                                self.set_status("Current JSON is not an object");
                                            }
                                        }
                                        Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                    }
                                } else {
                                    self.set_status("No 'inside' array in clipboard JSON");
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
                                obj.get("inside").and_then(|v| v.as_array()).cloned()
                            } else {
                                None
                            };

                            if let Some(new_inside_items) = new_inside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Get or create inside array
                                            let inside_array = obj.entry("inside".to_string())
                                                .or_insert(Value::Array(vec![]));

                                            if let Some(arr) = inside_array.as_array_mut() {
                                                // Insert new items at the beginning (like :ai)
                                                for (idx, item) in new_inside_items.into_iter().enumerate() {
                                                    arr.insert(idx, item);
                                                }

                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.sync_toon_from_json();
                                                        self.sync_markdown_from_json();
                                                        self.convert_json();
                                                        self.set_status("INSIDE entries inserted at top from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current 'inside' is not an array");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'inside' array in clipboard JSON");
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

    pub fn paste_outside_append(&mut self) {
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
                                    self.paste_markdown_section_append(&md_text, "OUTSIDE");
                                    return;
                                }
                            }
                        }

                        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
                            self.paste_markdown_section_append(&clipboard_text, "OUTSIDE");
                            return;
                        }

                        // Try to parse as Toon
                        if let Ok(clipboard_json) = self.clipboard_text_to_json_value(&clipboard_text) {
                            if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                self.paste_markdown_section_append(&md_text, "OUTSIDE");
                                return;
                            }
                        }

                        // Otherwise treat as Markdown
                        self.paste_markdown_section_append(&clipboard_text, "OUTSIDE");
                        return;
                    }

                    // For Toon files, parse Toon or JSON format
                    if self.is_toon_file() {
                        match self.clipboard_text_to_json_value(&clipboard_text) {
                            Ok(clipboard_json) => {
                                let new_outside = if let Some(obj) = clipboard_json.as_object() {
                                    obj.get("outside").and_then(|v| v.as_array()).cloned()
                                } else {
                                    None
                                };

                                if let Some(new_outside_items) = new_outside {
                                    match serde_json::from_str::<Value>(&self.json_input) {
                                        Ok(mut current_json) => {
                                            if let Some(obj) = current_json.as_object_mut() {
                                                let outside_array = obj.entry("outside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = outside_array.as_array_mut() {
                                                    for item in new_outside_items {
                                                        arr.push(item);
                                                    }

                                                    match serde_json::to_string_pretty(&current_json) {
                                                        Ok(formatted) => {
                                                            if let Err(e) = self.set_json_and_sync_toon(formatted) {
                                                                self.set_status(&e);
                                                                return;
                                                            }
                                                            self.set_status("OUTSIDE entries appended from clipboard");
                                                        }
                                                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                    }
                                                } else {
                                                    self.set_status("Current 'outside' is not an array");
                                                }
                                            } else {
                                                self.set_status("Current JSON is not an object");
                                            }
                                        }
                                        Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                    }
                                } else {
                                    self.set_status("No 'outside' array in clipboard JSON");
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
                                obj.get("outside").and_then(|v| v.as_array()).cloned()
                            } else {
                                None
                            };

                            if let Some(new_outside_items) = new_outside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Get or create outside array
                                            let outside_array = obj.entry("outside".to_string())
                                                .or_insert(Value::Array(vec![]));

                                            if let Some(arr) = outside_array.as_array_mut() {
                                                // Append new items
                                                for item in new_outside_items {
                                                    arr.push(item);
                                                }

                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.sync_toon_from_json();
                                                        self.sync_markdown_from_json();
                                                        self.convert_json();
                                                        self.set_status("OUTSIDE entries appended from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current 'outside' is not an array");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'outside' array in clipboard JSON");
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

    pub fn paste_append_all(&mut self) {
        // Append both inside and outside from clipboard
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    if self.is_markdown_file() {
                        let trimmed = clipboard_text.trim();

                        if trimmed.starts_with('{') || trimmed.starts_with('[') {
                            if let Ok(clipboard_json) = serde_json::from_str::<Value>(&clipboard_text) {
                                if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                    self.paste_markdown_section_append(&md_text, "OUTSIDE");
                                    self.paste_markdown_section_append(&md_text, "INSIDE");
                                    return;
                                }
                            }
                        }

                        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
                            self.paste_markdown_section_append(&clipboard_text, "OUTSIDE");
                            self.paste_markdown_section_append(&clipboard_text, "INSIDE");
                            return;
                        }

                        if let Ok(clipboard_json) = self.clipboard_text_to_json_value(&clipboard_text) {
                            if let Ok(md_text) = Self::json_to_markdown_string(&clipboard_json) {
                                self.paste_markdown_section_append(&md_text, "OUTSIDE");
                                self.paste_markdown_section_append(&md_text, "INSIDE");
                                return;
                            }
                        }
                    }

                    if self.is_toon_file() {
                        match self.clipboard_text_to_json_value(&clipboard_text) {
                            Ok(clipboard_json) => {
                                if let Some(clipboard_obj) = clipboard_json.as_object() {
                                    match serde_json::from_str::<Value>(&self.json_input) {
                                        Ok(mut current_json) => {
                                            if let Some(current_obj) = current_json.as_object_mut() {
                                                let mut appended_sections = Vec::new();

                                                if let Some(clipboard_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                                                    let inside_array = current_obj.entry("inside".to_string())
                                                        .or_insert(Value::Array(vec![]));

                                                    if let Some(arr) = inside_array.as_array_mut() {
                                                        for item in clipboard_inside {
                                                            arr.push(item.clone());
                                                        }
                                                        appended_sections.push("INSIDE");
                                                    }
                                                }

                                                if let Some(clipboard_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                                                    let outside_array = current_obj.entry("outside".to_string())
                                                        .or_insert(Value::Array(vec![]));

                                                    if let Some(arr) = outside_array.as_array_mut() {
                                                        for item in clipboard_outside {
                                                            arr.push(item.clone());
                                                        }
                                                        appended_sections.push("OUTSIDE");
                                                    }
                                                }

                                                if !appended_sections.is_empty() {
                                                    match serde_json::to_string_pretty(&current_json) {
                                                        Ok(formatted) => {
                                                            if let Err(e) = self.set_json_and_sync_toon(formatted) {
                                                                self.set_status(&e);
                                                                return;
                                                            }
                                                            self.set_status(&format!("{} appended from clipboard", appended_sections.join(" and ")));
                                                        }
                                                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                    }
                                                } else {
                                                    self.set_status("No inside/outside arrays in clipboard");
                                                }
                                            } else {
                                                self.set_status("Current JSON is not an object");
                                            }
                                        }
                                        Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                    }
                                } else {
                                    self.set_status("Clipboard JSON is not an object");
                                }
                            }
                            Err(e) => self.set_status(&e),
                        }
                        return;
                    }

                    match self.clipboard_text_to_json_value(&clipboard_text) {
                        Ok(clipboard_json) => {
                            if let Some(clipboard_obj) = clipboard_json.as_object() {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(current_obj) = current_json.as_object_mut() {
                                            let mut appended_sections = Vec::new();

                                            // Append INSIDE entries
                                            if let Some(clipboard_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                                                let inside_array = current_obj.entry("inside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = inside_array.as_array_mut() {
                                                    for item in clipboard_inside {
                                                        arr.push(item.clone());
                                                    }
                                                    appended_sections.push("INSIDE");
                                                }
                                            }

                                            // Append OUTSIDE entries
                                            if let Some(clipboard_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                                                let outside_array = current_obj.entry("outside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = outside_array.as_array_mut() {
                                                    for item in clipboard_outside {
                                                        arr.push(item.clone());
                                                    }
                                                    appended_sections.push("OUTSIDE");
                                                }
                                            }

                                            if !appended_sections.is_empty() {
                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.sync_toon_from_json();
                                                        self.sync_markdown_from_json();
                                                        self.convert_json();
                                                        self.set_status(&format!("{} appended from clipboard", appended_sections.join(" and ")));
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("No inside/outside arrays in clipboard");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("Clipboard JSON is not an object");
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

    /// Helper function to paste Markdown section content (INSIDE or OUTSIDE) from clipboard
    pub(super) fn paste_markdown_section_append(&mut self, clipboard_text: &str, section: &str) {
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

        // Now append these lines to the current markdown file
        let current_lines: Vec<&str> = self.markdown_input.lines().collect();
        let mut result_lines = Vec::new();
        let mut found_section = false;
        let mut inserted = false;

        for (i, line) in current_lines.iter().enumerate() {
            result_lines.push(line.to_string());

            let trimmed = line.trim();

            // Found target section header
            if trimmed == section_header.trim() {
                found_section = true;
            }

            // If we're in INSIDE section and about to hit end, or if we're in OUTSIDE and about to hit INSIDE, insert
            if found_section && !inserted {
                // Check if next line is a different section or EOF
                let is_before_new_section = if i + 1 < current_lines.len() {
                    current_lines[i + 1].trim().starts_with("## ") && current_lines[i + 1].trim() != section_header.trim()
                } else {
                    true // EOF
                };

                if is_before_new_section {
                    // Insert at the end of the current section
                    for section_line in &section_lines {
                        result_lines.push(section_line.to_string());
                    }
                    inserted = true;
                }
            }
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
                self.set_status(&format!("{} entries appended from clipboard", section));
            }
            Err(e) => {
                self.set_status(&format!("Failed to parse markdown: {}", e));
            }
        }
    }
}
