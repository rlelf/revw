use super::super::super::{App, FormatMode};
use arboard::Clipboard;
use crate::toon_ops::ToonOperations;
use serde_json::Value;

impl App {
    /// Copy selected card(s) with rendering
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

    /// Copy selected card(s) as Markdown format
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

    /// Copy selected card(s) as JSON format
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

    /// Copy selected card(s) as Toon format
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
}
