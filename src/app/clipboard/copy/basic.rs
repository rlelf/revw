use super::super::super::{App, FormatMode};
use arboard::Clipboard;
use serde_json::Value;

impl App {
    /// Copy all content to clipboard (both OUTSIDE and INSIDE sections)
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

    /// Copy INSIDE section data to clipboard
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

    /// Copy OUTSIDE section data to clipboard
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
}
