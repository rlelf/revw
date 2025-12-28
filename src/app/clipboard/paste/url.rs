use super::super::super::{App, FormatMode};
use arboard::Clipboard;
use serde_json::Value;

impl App {
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
}
