use super::super::{App, FormatMode};
use serde_json::Value;

impl App {
    /// Clear the INSIDE section
    pub fn clear_inside(&mut self) {
        // Clear INSIDE section
        self.save_undo_state();

        // For Markdown files
        if self.is_markdown_file() {
            self.clear_markdown_section("INSIDE");
            return;
        }

        // For Toon files
        if self.is_toon_file() {
            match serde_json::from_str::<Value>(&self.json_input) {
                Ok(mut current_json) => {
                    if let Some(obj) = current_json.as_object_mut() {
                        obj.insert("inside".to_string(), Value::Array(vec![]));

                        match serde_json::to_string_pretty(&current_json) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.is_modified = true;
                                self.sync_toon_from_json();
                                self.convert_json();
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
                            self.sync_markdown_from_json();
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

    /// Clear the OUTSIDE section
    pub fn clear_outside(&mut self) {
        // Clear OUTSIDE section
        self.save_undo_state();

        // For Markdown files
        if self.is_markdown_file() {
            self.clear_markdown_section("OUTSIDE");
            return;
        }

        // For Toon files
        if self.is_toon_file() {
            match serde_json::from_str::<Value>(&self.json_input) {
                Ok(mut current_json) => {
                    if let Some(obj) = current_json.as_object_mut() {
                        obj.insert("outside".to_string(), Value::Array(vec![]));

                        match serde_json::to_string_pretty(&current_json) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.is_modified = true;
                                self.sync_toon_from_json();
                                self.convert_json();
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
                            self.sync_markdown_from_json();
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

    /// Delete selected card(s) in Visual or View mode
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
                        self.sync_toon_from_json();
                        self.sync_markdown_from_json();
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

    /// Helper function to clear a section in Markdown files
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
