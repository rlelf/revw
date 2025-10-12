use super::{App, FormatMode};
use super::super::json_ops::JsonOperations;
use serde_json::Value;

impl App {
    pub fn open_entry_overlay(&mut self) {
        self.start_editing_entry();
    }

    pub fn start_editing_entry(&mut self) {
        // Get the original index from the selected entry (accounts for filtering)
        let target_idx = if self.selected_entry_index < self.relf_entries.len() {
            self.relf_entries[self.selected_entry_index].original_index
        } else {
            return; // Invalid selection
        };

        // Load fields from JSON (not from rendered lines) to include empty fields
        if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                let mut current_idx = 0;

                // Check outside section
                if let Some(outside) = obj.get("outside") {
                    if let Some(outside_array) = outside.as_array() {
                        if target_idx < current_idx + outside_array.len() {
                            let local_idx = target_idx - current_idx;
                            if let Some(entry_obj) = outside_array[local_idx].as_object() {
                                // Load all fields including empty ones, use placeholder if empty
                                let name = entry_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let context = entry_obj.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let url = entry_obj.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let percentage = entry_obj.get("percentage").and_then(|v| v.as_i64());

                                let name_is_empty = name.is_empty();
                                let context_is_empty = context.is_empty();
                                let url_is_empty = url.is_empty();

                                self.edit_buffer = vec![
                                    if name_is_empty { "name".to_string() } else { name },
                                    if context_is_empty { "context".to_string() } else { context },
                                    if url_is_empty { "url".to_string() } else { url },
                                    if let Some(pct) = percentage { pct.to_string() } else { "percentage".to_string() },
                                    "Exit".to_string(),
                                ];
                                self.edit_buffer_is_placeholder = vec![
                                    name_is_empty,
                                    context_is_empty,
                                    url_is_empty,
                                    percentage.is_none(),
                                    false, // Exit is never a placeholder
                                ];
                                self.edit_field_index = 0;
                                self.editing_entry = true;
                                self.edit_field_editing_mode = false;
                                self.edit_insert_mode = false;
                                self.edit_cursor_pos = 0;
                                return;
                            }
                        }
                        current_idx += outside_array.len();
                    }
                }

                // Check inside section
                if let Some(inside) = obj.get("inside") {
                    if let Some(inside_array) = inside.as_array() {
                        if target_idx < current_idx + inside_array.len() {
                            let local_idx = target_idx - current_idx;
                            if let Some(entry_obj) = inside_array[local_idx].as_object() {
                                // Load all fields including empty ones, use placeholder if empty
                                let date = entry_obj.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let context = entry_obj.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                let date_is_empty = date.is_empty();
                                let context_is_empty = context.is_empty();

                                self.edit_buffer = vec![
                                    if date_is_empty { "date".to_string() } else { date },
                                    if context_is_empty { "context".to_string() } else { context },
                                    "Exit".to_string(),
                                ];
                                self.edit_buffer_is_placeholder = vec![
                                    date_is_empty,
                                    context_is_empty,
                                    false, // Exit is never a placeholder
                                ];
                                self.edit_field_index = 0;
                                self.editing_entry = true;
                                self.edit_field_editing_mode = false;
                                self.edit_insert_mode = false;
                                self.edit_cursor_pos = 0;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save_edited_entry(&mut self) {
        // Save the edited entry back to JSON
        if self.edit_buffer.is_empty() {
            self.editing_entry = false;
            return;
        }

        // Get the original index from the selected entry (accounts for filtering)
        let target_idx = if self.selected_entry_index < self.relf_entries.len() {
            self.relf_entries[self.selected_entry_index].original_index
        } else {
            self.editing_entry = false;
            return; // Invalid selection
        };

        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut json_value) => {
                if let Some(obj) = json_value.as_object_mut() {
                    let mut current_idx = 0;
                    let mut found = false;

                    // Check outside section
                    if let Some(outside) = obj.get_mut("outside") {
                        if let Some(outside_array) = outside.as_array_mut() {
                            if target_idx < current_idx + outside_array.len() {
                                let local_idx = target_idx - current_idx;
                                if let Some(entry_obj) = outside_array[local_idx].as_object_mut() {
                                    // Update fields - use placeholder flags to determine if value is placeholder
                                    if self.edit_buffer.len() >= 1 && self.edit_buffer_is_placeholder.len() >= 1 {
                                        let name_val = &self.edit_buffer[0];
                                        let is_placeholder = self.edit_buffer_is_placeholder[0];
                                        entry_obj.insert("name".to_string(),
                                            Value::String(if is_placeholder { String::new() } else { name_val.clone() }));
                                    }
                                    if self.edit_buffer.len() >= 2 && self.edit_buffer_is_placeholder.len() >= 2 {
                                        let context_val = &self.edit_buffer[1];
                                        let is_placeholder = self.edit_buffer_is_placeholder[1];
                                        entry_obj.insert("context".to_string(),
                                            Value::String(if is_placeholder { String::new() } else { context_val.clone() }));
                                    }
                                    if self.edit_buffer.len() >= 3 && self.edit_buffer_is_placeholder.len() >= 3 {
                                        let url_val = &self.edit_buffer[2];
                                        let is_placeholder = self.edit_buffer_is_placeholder[2];
                                        entry_obj.insert("url".to_string(),
                                            Value::String(if is_placeholder { String::new() } else { url_val.clone() }));
                                    }
                                    if self.edit_buffer.len() >= 4 && self.edit_buffer_is_placeholder.len() >= 4 {
                                        // Parse percentage - save null if placeholder
                                        let pct_val = &self.edit_buffer[3];
                                        let is_placeholder = self.edit_buffer_is_placeholder[3];
                                        if is_placeholder {
                                            entry_obj.insert("percentage".to_string(), Value::Null);
                                        } else if let Ok(pct) = pct_val.trim_end_matches('%').parse::<i64>() {
                                            entry_obj.insert("percentage".to_string(), Value::Number(pct.into()));
                                        }
                                    }
                                    found = true;
                                }
                            } else {
                                current_idx += outside_array.len();
                            }
                        }
                    }

                    // Check inside section
                    if !found {
                        if let Some(inside) = obj.get_mut("inside") {
                            if let Some(inside_array) = inside.as_array_mut() {
                                let local_idx = target_idx - current_idx;
                                if local_idx < inside_array.len() {
                                    if let Some(entry_obj) = inside_array[local_idx].as_object_mut() {
                                        // Update fields (date and context for inside) - use placeholder flags
                                        if self.edit_buffer.len() >= 1 && self.edit_buffer_is_placeholder.len() >= 1 {
                                            let date_val = &self.edit_buffer[0];
                                            let is_placeholder = self.edit_buffer_is_placeholder[0];
                                            entry_obj.insert("date".to_string(),
                                                Value::String(if is_placeholder { String::new() } else { date_val.clone() }));
                                        }
                                        if self.edit_buffer.len() >= 2 && self.edit_buffer_is_placeholder.len() >= 2 {
                                            let context_val = &self.edit_buffer[1];
                                            let is_placeholder = self.edit_buffer_is_placeholder[1];
                                            entry_obj.insert("context".to_string(),
                                                Value::String(if is_placeholder { String::new() } else { context_val.clone() }));
                                        }
                                        found = true;
                                    }
                                }
                            }
                        }
                    }

                    if found {
                        match serde_json::to_string_pretty(&json_value) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("Entry updated");
                                // Auto-save after editing
                                self.save_file();
                            }
                            Err(e) => self.set_status(&format!("Error formatting JSON: {}", e)),
                        }
                    }
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }

        self.editing_entry = false;
    }

    pub fn cancel_editing_entry(&mut self) {
        self.editing_entry = false;
        self.edit_buffer.clear();
        self.edit_buffer_is_placeholder.clear();
        self.edit_field_index = 0;
        self.edit_insert_mode = false;
        self.edit_cursor_pos = 0;
        self.edit_hscroll = 0;
        self.edit_vscroll = 0;
    }

    pub fn insert_char(&mut self, c: char) {
        if self.format_mode == FormatMode::Edit {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                lines.push(String::new());
                self.content_cursor_line = 0;
                self.content_cursor_col = 0;
            }

            // Ensure cursor is within bounds
            if self.content_cursor_line >= lines.len() {
                self.content_cursor_line = lines.len().saturating_sub(1);
            }

            let line = &mut lines[self.content_cursor_line];
            let mut chars: Vec<char> = line.chars().collect();
            let pos = self.content_cursor_col.min(chars.len());
            chars.insert(pos, c);

            // Update the line with the new character
            lines[self.content_cursor_line] = chars.into_iter().collect();
            self.content_cursor_col += 1;
            self.set_json_from_lines(lines);
            self.ensure_cursor_visible();
        }
    }

    pub fn insert_newline(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                lines.push(String::new());
                lines.push(String::new());
                self.content_cursor_line = 1;
                self.content_cursor_col = 0;
            } else {
                // Ensure cursor is within bounds
                if self.content_cursor_line >= lines.len() {
                    self.content_cursor_line = lines.len().saturating_sub(1);
                }

                let line = lines[self.content_cursor_line].clone();
                let split_pos = self.content_cursor_col.min(line.len());
                let (left, right) = line.split_at(split_pos);
                lines[self.content_cursor_line] = left.to_string();
                lines.insert(self.content_cursor_line + 1, right.to_string());
                self.content_cursor_line += 1;
                self.content_cursor_col = 0;
            }
            self.set_json_from_lines(lines);
            self.ensure_cursor_visible();
        }
    }

    pub fn backspace(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                return;
            }
            if self.content_cursor_col > 0 && self.content_cursor_line < lines.len() {
                // Remove character before cursor (handle multi-byte chars)
                let mut chars: Vec<char> = lines[self.content_cursor_line].chars().collect();
                if self.content_cursor_col > 0 && self.content_cursor_col <= chars.len() {
                    chars.remove(self.content_cursor_col - 1);
                    lines[self.content_cursor_line] = chars.into_iter().collect();
                    self.content_cursor_col -= 1;
                    self.set_json_from_lines(lines);
                }
            } else if self.content_cursor_col == 0 && self.content_cursor_line > 0 {
                // Join with previous line
                let current_line = lines.remove(self.content_cursor_line);
                self.content_cursor_line -= 1;
                let prev_line_len = lines[self.content_cursor_line].chars().count();
                lines[self.content_cursor_line].push_str(&current_line);
                self.content_cursor_col = prev_line_len;
                self.set_json_from_lines(lines);
            }
        }
    }

    pub fn delete_char(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                return;
            }
            if self.content_cursor_line < lines.len() {
                let mut chars: Vec<char> = lines[self.content_cursor_line].chars().collect();
                if self.content_cursor_col < chars.len() {
                    chars.remove(self.content_cursor_col);
                    lines[self.content_cursor_line] = chars.into_iter().collect();
                    self.set_json_from_lines(lines);
                } else if self.content_cursor_line + 1 < lines.len() {
                    // Join with next line
                    let next_line = lines.remove(self.content_cursor_line + 1);
                    lines[self.content_cursor_line].push_str(&next_line);
                    self.set_json_from_lines(lines);
                }
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.content_cursor_col > 0 {
            self.content_cursor_col -= 1;
        } else if self.content_cursor_line > 0 {
            self.content_cursor_line -= 1;
            let lines = self.get_json_lines();
            if self.content_cursor_line < lines.len() {
                self.content_cursor_col = lines[self.content_cursor_line].chars().count();
            }
        }
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_right(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }
        if self.content_cursor_line < lines.len() {
            let line_len = lines[self.content_cursor_line].chars().count();
            if self.content_cursor_col < line_len {
                self.content_cursor_col += 1;
            } else if self.content_cursor_line + 1 < lines.len() {
                self.content_cursor_line += 1;
                self.content_cursor_col = 0;
            }
        }
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_up(&mut self) {
        if self.content_cursor_line > 0 {
            self.content_cursor_line -= 1;
            let lines = self.get_json_lines();
            if self.content_cursor_line < lines.len() {
                let line_len = lines[self.content_cursor_line].chars().count();
                self.content_cursor_col = self.content_cursor_col.min(line_len);
            }
        }
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_down(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }

        // Keep cursor within actual content lines
        let content_lines = if self.format_mode == FormatMode::Edit {
            lines.len()
        } else {
            self.rendered_content.len()
        };

        // Move cursor down if there's content, otherwise just scroll screen
        if self.content_cursor_line + 1 < content_lines {
            // Normal cursor movement within content
            self.content_cursor_line += 1;

            let line_len =
                if self.format_mode == FormatMode::Edit && self.content_cursor_line < lines.len() {
                    lines[self.content_cursor_line].chars().count()
                } else if self.content_cursor_line < self.rendered_content.len() {
                    self.rendered_content[self.content_cursor_line]
                        .chars()
                        .count()
                } else {
                    0
                };

            self.content_cursor_col = self.content_cursor_col.min(line_len);
        } else {
            // Cursor is at last line, just scroll the screen down (Mario style)
            // Don't add virtual padding in help mode
            let virtual_padding = if self.showing_help { 0 } else { 10 };
            let max_scroll =
                (content_lines as u16 + virtual_padding).saturating_sub(self.get_visible_height());
            if self.scroll < max_scroll {
                self.scroll += 1;
            }
        }
        self.ensure_cursor_visible();
    }

    pub fn append_inside(&mut self) {
        match JsonOperations::add_inside_entry(&self.json_input) {
            Ok((formatted, line, col, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();

                // Jump to the new entry (don't open edit overlay or insert mode)
                if self.format_mode == FormatMode::View {
                    // New inside entry is added at the beginning of inside array
                    // Index = outside.length (start of INSIDE section)
                    if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            let outside_count = obj
                                .get("outside")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            // INSIDE section starts right after OUTSIDE
                            self.selected_entry_index = outside_count;
                            self.scroll = 0;
                        }
                    }
                } else {
                    // In Edit mode, just move cursor to the new entry
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                }
                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    pub fn append_outside(&mut self) {
        match JsonOperations::add_outside_entry(&self.json_input) {
            Ok((formatted, line, col, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();

                // Jump to the new entry (don't open edit overlay or insert mode)
                if self.format_mode == FormatMode::View {
                    // New outside entry is added at the end of outside array
                    // Index = outside.length - 1 (last OUTSIDE entry)
                    if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            let outside_count = obj
                                .get("outside")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            // Last outside entry
                            self.selected_entry_index = outside_count.saturating_sub(1);
                            self.scroll = 0;
                        }
                    }
                } else {
                    // In Edit mode, just move cursor to the new entry
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                }
                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    pub fn ensure_cursor_visible(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
            return;
        }

        // Ensure cursor stays within actual content bounds
        let content_lines = if self.format_mode == FormatMode::Edit {
            lines.len()
        } else {
            self.rendered_content.len()
        };

        // Keep cursor within actual content lines (not in virtual padding)
        if self.content_cursor_line >= content_lines {
            self.content_cursor_line = content_lines.saturating_sub(1);
        }

        // Handle cursor column bounds
        let line_len =
            if self.format_mode == FormatMode::Edit && self.content_cursor_line < lines.len() {
                lines[self.content_cursor_line].chars().count()
            } else if self.content_cursor_line < self.rendered_content.len() {
                self.rendered_content[self.content_cursor_line]
                    .chars()
                    .count()
            } else {
                0
            };
        if self.content_cursor_col > line_len {
            self.content_cursor_col = line_len;
        }

        // Vertical scrolling
        let cursor_line = if self.format_mode == FormatMode::Edit {
            self.content_cursor_line as u16
        } else {
            self.calculate_cursor_visual_position().0
        };
        let visible_height = self.get_visible_height();
        let scrolloff = 3u16;
        if cursor_line < self.scroll {
            self.scroll = cursor_line;
        } else if visible_height > 0 && cursor_line >= self.scroll + visible_height {
            self.scroll = cursor_line.saturating_sub(visible_height - 1);
        } else if visible_height > scrolloff * 2 {
            if cursor_line < self.scroll + scrolloff {
                self.scroll = cursor_line.saturating_sub(scrolloff);
            } else if cursor_line > self.scroll + visible_height - scrolloff - 1 {
                self.scroll = cursor_line + scrolloff + 1 - visible_height;
            }
        }

        // Allow scrolling into virtual padding
        let virtual_padding = 10;
        let max_scroll = (content_lines as u16 + virtual_padding).saturating_sub(visible_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        // Horizontal follow for Edit mode
        if self.format_mode == FormatMode::Edit {
            if self.content_cursor_line < lines.len() {
                let current = &lines[self.content_cursor_line];
                let cursor_display_col = self.prefix_display_width(current, self.content_cursor_col) as u16;
                let w = self.get_content_width();

                // Add margin to scroll before cursor reaches edge
                let margin = 8u16;

                if cursor_display_col < self.hscroll + margin {
                    // Cursor near left edge - scroll left
                    self.hscroll = cursor_display_col.saturating_sub(margin);
                } else if cursor_display_col >= self.hscroll + w.saturating_sub(margin) {
                    // Cursor near right edge - scroll right
                    self.hscroll = cursor_display_col + margin - w + 1;
                }
            }
        }
    }

    pub fn order_entries(&mut self) {
        match JsonOperations::order_entries(&self.json_input) {
            Ok((formatted, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();

                // Auto-save in view mode
                if self.format_mode == FormatMode::View {
                    self.save_file();
                }

                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

}
