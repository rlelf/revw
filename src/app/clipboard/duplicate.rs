use super::super::{App, FormatMode};
use serde_json::Value;

impl App {
    /// Duplicate the selected entry in View mode or current entry in Edit mode
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
                                        self.sync_toon_from_json();
                                        self.sync_markdown_from_json();
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
                                        self.sync_toon_from_json();
                                        self.sync_markdown_from_json();
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
                            Err(e) => {
                                eprintln!("Warning: Failed to parse markdown: {}", e);
                            }
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
}
