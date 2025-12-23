use super::{App, FormatMode};
use arboard::Clipboard;
use crate::toon_ops::ToonOperations;
use serde_json::Value;
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
                        self.set_status("Pasted JSON content");
                        self.convert_json();
                    }
                    // Ignore status messages and other non-JSON text
                    else {
                        self.set_status("Clipboard doesn't contain JSON, Markdown, or file path");
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        // In View mode with cards, copy all entries with OUTSIDE/INSIDE sections
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    let mut all_content = Vec::new();

                    // Add OUTSIDE section
                    if outside_count > 0 {
                        all_content.push("OUTSIDE".to_string());
                        all_content.push(String::new());

                        for (i, entry) in self.relf_entries.iter().enumerate() {
                            if i < outside_count {
                                if i > 0 {
                                    all_content.push(String::new());
                                }
                                for line in &entry.lines {
                                    all_content.push(line.clone());
                                }
                            }
                        }

                        all_content.push(String::new());
                    }

                    // Add INSIDE section
                    let inside_count = self.relf_entries.len() - outside_count;
                    if inside_count > 0 {
                        all_content.push("INSIDE".to_string());
                        all_content.push(String::new());

                        for (i, entry) in self.relf_entries.iter().enumerate() {
                            if i >= outside_count {
                                if i > outside_count {
                                    all_content.push(String::new());
                                }
                                for line in &entry.lines {
                                    all_content.push(line.clone());
                                }
                            }
                        }
                    }

                    if all_content.is_empty() {
                        self.set_status("Nothing to copy");
                        return;
                    }

                    let content = all_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
        }

        // Fallback to rendered_content
        if self.rendered_content.is_empty() {
            self.set_status("Nothing to copy");
            return;
        }

        let content = self.rendered_content.join("\n");
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(content) {
                Ok(()) => self.set_status("Copied to clipboard"),
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn copy_inside_data(&mut self) {
        // In view mode, copy all INSIDE entries from relf_entries
        if self.format_mode == FormatMode::View {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    // Collect INSIDE entries (indices >= outside_count)
                    let mut inside_content = Vec::new();
                    inside_content.push("INSIDE".to_string());
                    inside_content.push(String::new());

                    for (i, entry) in self.relf_entries.iter().enumerate() {
                        if i >= outside_count {
                            // Add blank line between entries (but not before first entry)
                            if i > outside_count {
                                inside_content.push(String::new());
                            }
                            for line in &entry.lines {
                                inside_content.push(line.clone());
                            }
                        }
                    }

                    if inside_content.is_empty() {
                        self.set_status("No INSIDE entries found");
                        return;
                    }

                    let content = inside_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied INSIDE section to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
            self.set_status("Failed to parse JSON");
            return;
        }

        // In Edit mode, copy with "inside: [...]" wrapper
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(json_value) => {
                if let Some(obj) = json_value.as_object() {
                    if let Some(inside) = obj.get("inside") {
                        // Create wrapper object with "inside" key
                        let mut wrapper = serde_json::Map::new();
                        wrapper.insert("inside".to_string(), inside.clone());
                        let wrapper_value = Value::Object(wrapper);

                        match serde_json::to_string_pretty(&wrapper_value) {
                            Ok(formatted) => match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(formatted) {
                                    Ok(()) => self.set_status("Copied inside data to clipboard"),
                                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                                },
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => {
                                self.set_status(&format!("Error formatting inside data: {}", e))
                            }
                        }
                    } else {
                        self.set_status("No 'inside' field found");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn copy_selected_url(&mut self) {
        // Copy URL from selected entry in Relf card mode
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            if let Some(entry) = self.relf_entries.get(self.selected_entry_index) {
                // Find URL in entry lines (usually starts with "http")
                let url = entry.lines.iter().find(|line| line.starts_with("http"));

                if let Some(url_str) = url {
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(url_str.clone()) {
                            Ok(()) => self.set_status(&format!("Copied URL: {}", url_str)),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                } else {
                    self.set_status("No URL found in selected entry");
                }
            } else {
                self.set_status("No entry selected");
            }
            return;
        }

        self.set_status("Not in card view mode");
    }

    pub fn paste_url_to_selected(&mut self) {
        // Paste URL from clipboard to selected entry in View mode
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(url) => {
                    let url = url.trim();

                    // Basic URL validation
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        self.set_status("Clipboard doesn't contain a valid URL (must start with http:// or https://)");
                        return;
                    }

                    if let Some(entry) = self.relf_entries.get_mut(self.selected_entry_index) {
                        // Update URL in the entry's lines
                        // Find and replace existing URL line
                        let mut url_found = false;
                        for line in entry.lines.iter_mut() {
                            if line.starts_with("http://") || line.starts_with("https://") {
                                *line = url.to_string();
                                url_found = true;
                                break;
                            }
                        }

                        // If no URL was found, add it
                        if !url_found {
                            entry.lines.push(url.to_string());
                        }

                        // Update the underlying JSON data
                        if let Ok(mut json_value) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(outside) = json_value.get_mut("outside").and_then(|v| v.as_array_mut()) {
                                // Find the matching outside entry
                                for outside_entry in outside.iter_mut() {
                                    if let Some(obj) = outside_entry.as_object_mut() {
                                        // Check if this is the right entry by comparing name
                                        if let Some(name_val) = obj.get("name") {
                                            if entry.lines.iter().any(|l| l.contains(&name_val.as_str().unwrap_or(""))) {
                                                obj.insert("url".to_string(), Value::String(url.to_string()));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            // Update json_input
                            self.json_input = serde_json::to_string_pretty(&json_value).unwrap_or(self.json_input.clone());
                        }

                        self.set_status(&format!("URL pasted: {}", url));
                        self.save_file();
                    } else {
                        self.set_status("No entry selected");
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

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

    pub fn clear_inside(&mut self) {
        // Clear INSIDE section
        self.save_undo_state();

        // For Markdown files
        if self.is_markdown_file() {
            self.clear_markdown_section("INSIDE");
            return;
        }

        // For JSON files
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut current_json) => {
                if let Some(obj) = current_json.as_object_mut() {
                    // Set inside to empty array
                    obj.insert("inside".to_string(), Value::Array(vec![]));

                    // Format and save
                    match serde_json::to_string_pretty(&current_json) {
                        Ok(formatted) => {
                            self.json_input = formatted;
                            self.is_modified = true;
                            self.convert_json();
                            // Reset selection to first entry if current selection is out of bounds
                            if !self.relf_entries.is_empty() && self.selected_entry_index >= self.relf_entries.len() {
                                self.selected_entry_index = 0;
                            }
                            self.set_status("INSIDE section cleared");
                        }
                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                    }
                } else {
                    self.set_status("Current JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn clear_outside(&mut self) {
        // Clear OUTSIDE section
        self.save_undo_state();

        // For Markdown files
        if self.is_markdown_file() {
            self.clear_markdown_section("OUTSIDE");
            return;
        }

        // For JSON files
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut current_json) => {
                if let Some(obj) = current_json.as_object_mut() {
                    // Set outside to empty array
                    obj.insert("outside".to_string(), Value::Array(vec![]));

                    // Format and save
                    match serde_json::to_string_pretty(&current_json) {
                        Ok(formatted) => {
                            self.json_input = formatted;
                            self.is_modified = true;
                            self.convert_json();
                            // Reset selection to first entry if current selection is out of bounds
                            if !self.relf_entries.is_empty() && self.selected_entry_index >= self.relf_entries.len() {
                                self.selected_entry_index = 0;
                            }
                            self.set_status("OUTSIDE section cleared");
                        }
                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                    }
                } else {
                    self.set_status("Current JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn copy_outside_data(&mut self) {
        // In view mode, copy all OUTSIDE entries from relf_entries
        if self.format_mode == FormatMode::View {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    // Collect OUTSIDE entries (indices < outside_count)
                    let mut outside_content = Vec::new();
                    outside_content.push("OUTSIDE".to_string());
                    outside_content.push(String::new());

                    for (i, entry) in self.relf_entries.iter().enumerate() {
                        if i < outside_count {
                            // Add blank line between entries (but not before first entry)
                            if i > 0 {
                                outside_content.push(String::new());
                            }
                            for line in &entry.lines {
                                outside_content.push(line.clone());
                            }
                        }
                    }

                    if outside_content.is_empty() {
                        self.set_status("No OUTSIDE entries found");
                        return;
                    }

                    let content = outside_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied OUTSIDE section to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
            self.set_status("Failed to parse JSON");
            return;
        }

        // In Edit mode, copy with "outside: [...]" wrapper
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(json_value) => {
                if let Some(obj) = json_value.as_object() {
                    if let Some(outside) = obj.get("outside") {
                        // Create wrapper object with "outside" key
                        let mut wrapper = serde_json::Map::new();
                        wrapper.insert("outside".to_string(), outside.clone());
                        let wrapper_value = Value::Object(wrapper);

                        match serde_json::to_string_pretty(&wrapper_value) {
                            Ok(formatted) => match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(formatted) {
                                    Ok(()) => self.set_status("Copied outside data to clipboard"),
                                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                                },
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => {
                                self.set_status(&format!("Error formatting outside data: {}", e))
                            }
                        }
                    } else {
                        self.set_status("No 'outside' field found");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn copy_cards_rendered(&mut self) {
        // Copy card(s) with rendering
        // In Visual mode: copy selected range and exit Visual mode
        // In View mode (non-Visual): copy current card only
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        // Get outside_count to determine which entries are OUTSIDE/INSIDE
        let outside_count = if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            json_value
                .as_object()
                .and_then(|obj| obj.get("outside"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        } else {
            0
        };

        let (start_idx, end_idx) = if self.visual_mode {
            let start = self.visual_start_index.min(self.visual_end_index);
            let end = self.visual_start_index.max(self.visual_end_index);
            (start, end)
        } else {
            // Single card mode
            (self.selected_entry_index, self.selected_entry_index)
        };

        // Separate OUTSIDE and INSIDE entries
        let mut outside_lines = Vec::new();
        let mut inside_lines = Vec::new();

        for idx in start_idx..=end_idx {
            if idx >= self.relf_entries.len() {
                break;
            }
            let entry = &self.relf_entries[idx];
            let original_idx = entry.original_index;

            if original_idx < outside_count {
                // OUTSIDE entry
                if !outside_lines.is_empty() {
                    outside_lines.push(String::new());
                }
                for line in &entry.lines {
                    outside_lines.push(line.clone());
                }
            } else {
                // INSIDE entry
                if !inside_lines.is_empty() {
                    inside_lines.push(String::new());
                }
                for line in &entry.lines {
                    inside_lines.push(line.clone());
                }
            }
        }

        // Build final content with headers
        let mut content_lines = Vec::new();

        if !outside_lines.is_empty() {
            content_lines.push("OUTSIDE".to_string());
            content_lines.push(String::new());
            content_lines.extend(outside_lines);
        }

        if !inside_lines.is_empty() {
            if !content_lines.is_empty() {
                content_lines.push(String::new());
            }
            content_lines.push("INSIDE".to_string());
            content_lines.push(String::new());
            content_lines.extend(inside_lines);
        }

        if content_lines.is_empty() {
            self.set_status("No cards to copy");
            return;
        }

        let content = content_lines.join("\n");
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(content) {
                Ok(()) => {
                    let count = end_idx - start_idx + 1;
                    self.set_status(&format!("Copied {} card(s)", count));
                    // Exit Visual mode after copy
                    if self.visual_mode {
                        self.visual_mode = false;
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn copy_json(&mut self) {
        // Copy current content as JSON (works in both Edit and View modes)
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(self.json_input.clone()) {
                Ok(()) => self.set_status("Copied as JSON"),
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn copy_markdown(&mut self) {
        // Copy current content as Markdown (works in both Edit and View modes)
        match self.convert_to_markdown() {
            Ok(markdown_content) => {
                match Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(markdown_content) {
                        Ok(()) => self.set_status("Copied as Markdown"),
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                }
            }
            Err(e) => self.set_status(&format!("Failed to convert to Markdown: {}", e)),
        }
    }

    pub fn copy_toon(&mut self) {
        // Copy current content as Toon (works in both Edit and View modes)
        match self.convert_to_toon() {
            Ok(toon_content) => {
                match Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(toon_content) {
                        Ok(()) => self.set_status("Copied as Toon"),
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                }
            }
            Err(e) => self.set_status(&format!("Failed to convert to Toon: {}", e)),
        }
    }

    pub fn copy_cards_markdown(&mut self) {
        // Copy card(s) as Markdown
        // In Visual mode: copy selected range and exit Visual mode
        // In View mode (non-Visual): copy current card only
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                let outside_count = obj
                    .get("outside")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                let (start_idx, end_idx) = if self.visual_mode {
                    let start = self.visual_start_index.min(self.visual_end_index);
                    let end = self.visual_start_index.max(self.visual_end_index);
                    (start, end)
                } else {
                    (self.selected_entry_index, self.selected_entry_index)
                };

                // Collect selected entries from JSON
                let mut selected_outside = Vec::new();
                let mut selected_inside = Vec::new();

                for idx in start_idx..=end_idx {
                    if idx >= self.relf_entries.len() {
                        break;
                    }
                    let original_idx = self.relf_entries[idx].original_index;

                    if original_idx < outside_count {
                        // Outside entry
                        if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                            if original_idx < outside.len() {
                                selected_outside.push(outside[original_idx].clone());
                            }
                        }
                    } else {
                        // Inside entry
                        let inside_idx = original_idx - outside_count;
                        if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                            if inside_idx < inside.len() {
                                selected_inside.push(inside[inside_idx].clone());
                            }
                        }
                    }
                }

                // Build JSON object with selected entries
                let mut result_obj = serde_json::Map::new();
                if !selected_outside.is_empty() {
                    result_obj.insert("outside".to_string(), Value::Array(selected_outside));
                }
                if !selected_inside.is_empty() {
                    result_obj.insert("inside".to_string(), Value::Array(selected_inside));
                }

                if result_obj.is_empty() {
                    self.set_status("No cards to copy");
                    return;
                }

                // Convert to markdown format using helper function
                match Self::json_to_markdown_string(&Value::Object(result_obj)) {
                    Ok(markdown_str) => {
                        match Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text(markdown_str) {
                                Ok(()) => {
                                    let count = end_idx - start_idx + 1;
                                    self.set_status(&format!("Copied {} card(s) as Markdown", count));
                                    // Exit Visual mode after copy
                                    if self.visual_mode {
                                        self.visual_mode = false;
                                    }
                                }
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        }
                    }
                    Err(e) => self.set_status(&format!("Markdown conversion error: {}", e)),
                }
            }
        }
    }

    pub fn copy_cards_json(&mut self) {
        // Copy card(s) as JSON
        // In Visual mode: copy selected range and exit Visual mode
        // In View mode (non-Visual): copy current card only
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                let outside_count = obj
                    .get("outside")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                let (start_idx, end_idx) = if self.visual_mode {
                    let start = self.visual_start_index.min(self.visual_end_index);
                    let end = self.visual_start_index.max(self.visual_end_index);
                    (start, end)
                } else {
                    (self.selected_entry_index, self.selected_entry_index)
                };

                // Collect selected entries from JSON
                let mut selected_outside = Vec::new();
                let mut selected_inside = Vec::new();

                for idx in start_idx..=end_idx {
                    if idx >= self.relf_entries.len() {
                        break;
                    }
                    let original_idx = self.relf_entries[idx].original_index;

                    if original_idx < outside_count {
                        // Outside entry
                        if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                            if original_idx < outside.len() {
                                selected_outside.push(outside[original_idx].clone());
                            }
                        }
                    } else {
                        // Inside entry
                        let inside_idx = original_idx - outside_count;
                        if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                            if inside_idx < inside.len() {
                                selected_inside.push(inside[inside_idx].clone());
                            }
                        }
                    }
                }

                // Build JSON object
                let mut result_obj = serde_json::Map::new();
                if !selected_outside.is_empty() {
                    result_obj.insert("outside".to_string(), Value::Array(selected_outside));
                }
                if !selected_inside.is_empty() {
                    result_obj.insert("inside".to_string(), Value::Array(selected_inside));
                }

                if result_obj.is_empty() {
                    self.set_status("No cards to copy");
                    return;
                }

                match serde_json::to_string_pretty(&Value::Object(result_obj)) {
                    Ok(json_str) => {
                        match Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text(json_str) {
                                Ok(()) => {
                                    let count = end_idx - start_idx + 1;
                                    self.set_status(&format!("Copied {} card(s) as JSON", count));
                                    // Exit Visual mode after copy
                                    if self.visual_mode {
                                        self.visual_mode = false;
                                    }
                                }
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        }
                    }
                    Err(e) => self.set_status(&format!("JSON error: {}", e)),
                }
            }
        }
    }

    pub fn copy_cards_toon(&mut self) {
        // Copy card(s) as Toon
        // In Visual mode: copy selected range and exit Visual mode
        // In View mode (non-Visual): copy current card only
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                let outside_count = obj
                    .get("outside")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                let (start_idx, end_idx) = if self.visual_mode {
                    let start = self.visual_start_index.min(self.visual_end_index);
                    let end = self.visual_start_index.max(self.visual_end_index);
                    (start, end)
                } else {
                    (self.selected_entry_index, self.selected_entry_index)
                };

                // Collect selected entries from JSON
                let mut selected_outside = Vec::new();
                let mut selected_inside = Vec::new();

                for idx in start_idx..=end_idx {
                    if idx >= self.relf_entries.len() {
                        break;
                    }
                    let original_idx = self.relf_entries[idx].original_index;

                    if original_idx < outside_count {
                        // Outside entry
                        if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                            if original_idx < outside.len() {
                                selected_outside.push(outside[original_idx].clone());
                            }
                        }
                    } else {
                        // Inside entry
                        let inside_idx = original_idx - outside_count;
                        if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                            if inside_idx < inside.len() {
                                selected_inside.push(inside[inside_idx].clone());
                            }
                        }
                    }
                }

                // Build JSON object
                let mut result_obj = serde_json::Map::new();
                if !selected_outside.is_empty() {
                    result_obj.insert("outside".to_string(), Value::Array(selected_outside));
                }
                if !selected_inside.is_empty() {
                    result_obj.insert("inside".to_string(), Value::Array(selected_inside));
                }

                if result_obj.is_empty() {
                    self.set_status("No cards to copy");
                    return;
                }

                match serde_json::to_string_pretty(&Value::Object(result_obj)) {
                    Ok(json_str) => match ToonOperations::json_to_toon(&json_str) {
                        Ok(toon_str) => {
                            match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(toon_str) {
                                    Ok(()) => {
                                        let count = end_idx - start_idx + 1;
                                        self.set_status(&format!("Copied {} card(s) as Toon", count));
                                        // Exit Visual mode after copy
                                        if self.visual_mode {
                                            self.visual_mode = false;
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                                },
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            }
                        }
                        Err(e) => self.set_status(&format!("Toon conversion error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("JSON error: {}", e)),
                }
            }
        }
    }

    pub fn delete_cards(&mut self) {
        // Delete card(s)
        // In Visual mode: delete selected range and exit Visual mode
        // In View mode (non-Visual): delete current card only
        if self.format_mode != FormatMode::View || self.relf_entries.is_empty() {
            self.set_status("Not in card view mode");
            return;
        }

        let (start_idx, end_idx) = if self.visual_mode {
            let start = self.visual_start_index.min(self.visual_end_index);
            let end = self.visual_start_index.max(self.visual_end_index);
            (start, end)
        } else {
            (self.selected_entry_index, self.selected_entry_index)
        };

        // Get original indices to delete
        let mut original_indices = Vec::new();
        for idx in start_idx..=end_idx {
            if idx < self.relf_entries.len() {
                original_indices.push(self.relf_entries[idx].original_index);
            }
        }

        if original_indices.is_empty() {
            self.set_status("No cards to delete");
            return;
        }

        // Delete from JSON
        if let Ok(mut json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object_mut() {
                let outside_count = obj
                    .get("outside")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                // Separate indices into outside and inside
                let mut outside_to_delete = Vec::new();
                let mut inside_to_delete = Vec::new();

                for original_idx in original_indices {
                    if original_idx < outside_count {
                        outside_to_delete.push(original_idx);
                    } else {
                        inside_to_delete.push(original_idx - outside_count);
                    }
                }

                // Sort in reverse to delete from end to start
                outside_to_delete.sort_by(|a, b| b.cmp(a));
                inside_to_delete.sort_by(|a, b| b.cmp(a));

                // Delete from outside
                if let Some(outside) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                    for idx in outside_to_delete {
                        if idx < outside.len() {
                            outside.remove(idx);
                        }
                    }
                }

                // Delete from inside
                if let Some(inside) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                    for idx in inside_to_delete {
                        if idx < inside.len() {
                            inside.remove(idx);
                        }
                    }
                }

                // Update JSON and re-render
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => {
                        self.save_undo_state();
                        self.json_input = formatted;
                        self.is_modified = true;
                        self.convert_json();

                        // Adjust selected index
                        if self.selected_entry_index >= self.relf_entries.len() && !self.relf_entries.is_empty() {
                            self.selected_entry_index = self.relf_entries.len() - 1;
                        }

                        let count = end_idx - start_idx + 1;
                        self.set_status(&format!("Deleted {} card(s)", count));

                        // Exit Visual mode and save
                        if self.visual_mode {
                            self.visual_mode = false;
                        }
                        self.save_file();
                    }
                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                }
            }
        }
    }

    /// Helper function to convert JSON value to Markdown string
    pub(crate) fn json_to_markdown_string(json_value: &Value) -> Result<String, String> {
        let mut output_lines = Vec::new();

        if let Some(obj) = json_value.as_object() {
            // OUTSIDE section
            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                if !outside.is_empty() {
                    output_lines.push("## OUTSIDE".to_string());
                    output_lines.push("".to_string());

                    for item in outside {
                        if let Some(item_obj) = item.as_object() {
                            let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                            let url = item_obj.get("url").and_then(|v| v.as_str());
                            let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                            if !name.is_empty() {
                                output_lines.push(format!("### {}", name));
                            }

                            if !context.is_empty() {
                                output_lines.push(context.to_string());
                            }

                            // Only output URL if it's not null and not empty
                            if let Some(url_str) = url {
                                if !url_str.is_empty() {
                                    output_lines.push("".to_string());
                                    output_lines.push(format!("**URL:** {}", url_str));
                                }
                            }

                            // Only output percentage if it's not null
                            if let Some(pct) = percentage {
                                output_lines.push("".to_string());
                                output_lines.push(format!("**Percentage:** {}%", pct));
                            }

                            // Only add blank line if we had any content
                            if !name.is_empty() || !context.is_empty() || url.is_some() || percentage.is_some() {
                                output_lines.push("".to_string());
                            }
                        }
                    }
                }
            }

            // INSIDE section
            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                if !inside.is_empty() {
                    output_lines.push("## INSIDE".to_string());
                    output_lines.push("".to_string());

                    for item in inside {
                        if let Some(item_obj) = item.as_object() {
                            let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                            if !date.is_empty() {
                                output_lines.push(format!("### {}", date));
                            }

                            if !context.is_empty() {
                                output_lines.push(context.to_string());
                            }

                            // Only add blank line if we had content
                            if !date.is_empty() || !context.is_empty() {
                                output_lines.push("".to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(output_lines.join("\n"))
    }

    fn clipboard_text_to_json_value(&self, clipboard_text: &str) -> Result<Value, String> {
        if let Ok(json_value) = serde_json::from_str::<Value>(clipboard_text) {
            return Ok(json_value);
        }

        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
            let json_str = self
                .parse_markdown(clipboard_text)
                .map_err(|e| format!("Clipboard is not valid Markdown: {}", e))?;
            return serde_json::from_str::<Value>(&json_str)
                .map_err(|e| format!("Clipboard is not valid JSON: {}", e));
        }

        let json_str = self
            .parse_toon(clipboard_text)
            .map_err(|e| format!("Clipboard is not valid JSON, Markdown, or Toon: {}", e))?;
        serde_json::from_str::<Value>(&json_str)
            .map_err(|e| format!("Clipboard is not valid JSON: {}", e))
    }

    fn set_json_and_sync_toon(&mut self, formatted: String) -> Result<(), String> {
        self.json_input = formatted;
        let toon_content = self
            .convert_to_toon()
            .map_err(|e| format!("Toon conversion error: {}", e))?;
        self.toon_input = toon_content;
        self.is_modified = true;
        self.convert_json();
        Ok(())
    }

    /// Helper function to paste Markdown section content (INSIDE or OUTSIDE) from clipboard (overwrite)
    fn paste_markdown_section_overwrite(&mut self, clipboard_text: &str, section: &str) {
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

    /// Helper function to paste Markdown section content (INSIDE or OUTSIDE) from clipboard
    fn paste_markdown_section_append(&mut self, clipboard_text: &str, section: &str) {
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

    pub fn duplicate_selected_entry(&mut self) {
        // Duplicate selected entry in View mode or current line in Edit mode
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            // Get the original index from the selected entry (accounts for filtering)
            let target_idx = self.relf_entries[self.selected_entry_index].original_index;

            // View mode: duplicate selected entry in JSON
            if let Ok(mut json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object_mut() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    // Determine which section the selected entry belongs to
                    if target_idx < outside_count {
                        // Duplicate OUTSIDE entry
                        if let Some(outside) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                            if target_idx < outside.len() {
                                let entry_clone = outside[target_idx].clone();
                                outside.insert(target_idx + 1, entry_clone);

                                // Update JSON and re-render
                                match serde_json::to_string_pretty(&json_value) {
                                    Ok(formatted) => {
                                        self.save_undo_state();
                                        self.json_input = formatted;
                                        // Update markdown_input if this is a markdown file
                                        if self.is_markdown_file() {
                                            if let Ok(md) = self.convert_to_markdown() {
                                                self.markdown_input = md;
                                            }
                                        }
                                        self.is_modified = true;
                                        self.convert_json();
                                        self.selected_entry_index += 1; // Move to duplicated entry
                                        self.set_status("Entry duplicated");
                                        self.save_file(); // Auto-save in View mode
                                    }
                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                }
                            }
                        }
                    } else {
                        // Duplicate INSIDE entry
                        let inside_index = target_idx - outside_count;
                        if let Some(inside) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                            if inside_index < inside.len() {
                                let entry_clone = inside[inside_index].clone();
                                inside.insert(inside_index + 1, entry_clone);

                                // Update JSON and re-render
                                match serde_json::to_string_pretty(&json_value) {
                                    Ok(formatted) => {
                                        self.save_undo_state();
                                        self.json_input = formatted;
                                        // Update markdown_input if this is a markdown file
                                        if self.is_markdown_file() {
                                            if let Ok(md) = self.convert_to_markdown() {
                                                self.markdown_input = md;
                                            }
                                        }
                                        self.is_modified = true;
                                        self.convert_json();
                                        self.selected_entry_index += 1; // Move to duplicated entry
                                        self.set_status("Entry duplicated");
                                        self.save_file(); // Auto-save in View mode
                                    }
                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                }
                            }
                        }
                    }
                }
            }
        } else if self.format_mode == FormatMode::Edit {
            // Edit mode: duplicate current entry
            self.save_undo_state();

            let lines = self.get_content_lines();
            let ops = self.get_operations();
            let content = if self.is_markdown_file() && !self.markdown_input.is_empty() {
                &self.markdown_input
            } else {
                &self.json_input
            };

            match ops.duplicate_entry_at_cursor(
                content,
                self.content_cursor_line,
                &lines,
            ) {
                Ok((formatted, message)) => {
                    if self.is_markdown_file() {
                        self.markdown_input = formatted;
                        match self.parse_markdown(&self.markdown_input) {
                            Ok(json_content) => {
                                self.json_input = json_content;
                            }
                            Err(_) => {}
                        }
                    } else {
                        self.json_input = formatted;
                    }
                    self.convert_json();
                    self.is_modified = true;
                    self.set_status(&message);
                }
                Err(e) => self.set_status(&e),
            }
        }
    }

    // Public functions for pasting from file content (used by CLI)

    /// Paste content from text
    pub fn paste_from_text(&mut self, text: &str) {
        let trimmed = text.trim();

        // For Markdown files, check if it looks like Markdown content
        if self.is_markdown_file() && (trimmed.contains("## INSIDE") || trimmed.contains("## OUTSIDE") || trimmed.starts_with("### ")) {
            self.markdown_input = text.to_string();
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
            self.json_input = text.to_string();
            self.is_modified = true;
            self.set_status("Pasted JSON content");
            self.convert_json();
        }
        else {
            self.set_status("Content doesn't contain JSON or Markdown");
        }
    }

    /// Paste INSIDE section from text (overwrite)
    pub fn paste_inside_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_overwrite(text, "INSIDE");
        } else {
            // For JSON files, parse clipboard for INSIDE section
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_inside) = clipboard_obj.get("inside") {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                current_obj.insert("inside".to_string(), new_inside.clone());
                                self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("INSIDE section overwritten");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste OUTSIDE section from text (overwrite)
    pub fn paste_outside_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_overwrite(text, "OUTSIDE");
        } else {
            // For JSON files, parse clipboard for OUTSIDE section
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_outside) = clipboard_obj.get("outside") {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                current_obj.insert("outside".to_string(), new_outside.clone());
                                self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("OUTSIDE section overwritten");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste INSIDE section from text (append)
    pub fn paste_inside_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_append(text, "INSIDE");
        } else {
            // For JSON files, append INSIDE entries
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                let current_inside = current_obj
                                    .entry("inside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(inside_array) = current_inside.as_array_mut() {
                                    inside_array.extend(new_inside.clone());
                                    self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                    self.is_modified = true;
                                    self.convert_json();
                                    self.set_status(&format!("Appended {} INSIDE entries", new_inside.len()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste OUTSIDE section from text (append)
    pub fn paste_outside_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_append(text, "OUTSIDE");
        } else {
            // For JSON files, append OUTSIDE entries
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                let current_outside = current_obj
                                    .entry("outside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(outside_array) = current_outside.as_array_mut() {
                                    outside_array.extend(new_outside.clone());
                                    self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                    self.is_modified = true;
                                    self.convert_json();
                                    self.set_status(&format!("Appended {} OUTSIDE entries", new_outside.len()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste both INSIDE and OUTSIDE from text (append)
    pub fn paste_all_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            // For Markdown files, append both sections
            self.paste_markdown_section_append(text, "OUTSIDE");
            self.paste_markdown_section_append(text, "INSIDE");
        } else {
            // For JSON files
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(current_obj) = current_json.as_object_mut() {
                            let mut appended_count = 0;

                            // Append INSIDE
                            if let Some(new_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                                let current_inside = current_obj
                                    .entry("inside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(inside_array) = current_inside.as_array_mut() {
                                    inside_array.extend(new_inside.clone());
                                    appended_count += new_inside.len();
                                }
                            }

                            // Append OUTSIDE
                            if let Some(new_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                                let current_outside = current_obj
                                    .entry("outside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(outside_array) = current_outside.as_array_mut() {
                                    outside_array.extend(new_outside.clone());
                                    appended_count += new_outside.len();
                                }
                            }

                            self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                            self.is_modified = true;
                            self.convert_json();
                            self.set_status(&format!("Appended {} entries total", appended_count));
                        }
                    }
                }
            }
        }
    }

    fn clear_markdown_section(&mut self, section: &str) {
        // Clear a section in markdown file by removing all content under that section header
        let section_header = format!("## {}", section);
        let current_lines: Vec<&str> = self.markdown_input.lines().collect();
        let mut result_lines = Vec::new();
        let mut in_section_to_clear = false;
        let mut found_section = false;

        for line in current_lines {
            let trimmed = line.trim();

            // Found target section header
            if trimmed == section_header.trim() {
                found_section = true;
                in_section_to_clear = true;
                result_lines.push(line.to_string());
                // Add blank line after header
                result_lines.push("".to_string());
                continue;
            }

            // Check if we're entering a different section (end of section to clear)
            if in_section_to_clear && trimmed.starts_with("## ") {
                in_section_to_clear = false;
                result_lines.push(line.to_string());
                continue;
            }

            // Skip lines in section to clear
            if in_section_to_clear {
                continue;
            }

            // Keep all other lines
            result_lines.push(line.to_string());
        }

        if !found_section {
            // If section doesn't exist, add it as empty
            if !result_lines.is_empty() && !result_lines.last().unwrap().is_empty() {
                result_lines.push("".to_string());
            }
            result_lines.push(section_header);
            result_lines.push("".to_string());
        }

        self.markdown_input = result_lines.join("\n");
        match self.parse_markdown(&self.markdown_input.clone()) {
            Ok(json_content) => {
                self.json_input = json_content;
                self.is_modified = true;
                self.convert_json();
                // Reset selection to first entry if current selection is out of bounds
                if !self.relf_entries.is_empty() && self.selected_entry_index >= self.relf_entries.len() {
                    self.selected_entry_index = 0;
                }
                self.set_status(&format!("{} section cleared", section));
            }
            Err(e) => {
                self.set_status(&format!("Failed to parse Markdown: {}", e));
            }
        }
    }
}
