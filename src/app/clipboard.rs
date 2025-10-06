use super::{App, FormatMode};
use super::super::json_ops::JsonOperations;
use arboard::Clipboard;
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
                    // Check if it looks like JSON
                    else if trimmed.starts_with('{') || trimmed.starts_with('[') {
                        self.json_input = text;
                        self.set_status("Pasted JSON content");
                        self.convert_json();
                    }
                    // Ignore status messages and other non-JSON text
                    else {
                        self.set_status("Clipboard doesn't contain JSON or file path");
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
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
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
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
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
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
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
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
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
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
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
                                                // Append new items
                                                for item in new_inside_items {
                                                    arr.push(item);
                                                }

                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.convert_json();
                                                        self.set_status("INSIDE entries appended from clipboard");
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
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
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
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
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
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
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
                    match serde_json::from_str::<Value>(&clipboard_text) {
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
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
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

            let lines = self.get_json_lines();
            match JsonOperations::duplicate_entry_at_cursor(
                &self.json_input,
                self.content_cursor_line,
                &lines,
            ) {
                Ok((formatted, message)) => {
                    self.json_input = formatted;
                    self.convert_json();
                    self.is_modified = true;
                    self.set_status(&message);
                }
                Err(e) => self.set_status(&e),
            }
        }
    }
}
