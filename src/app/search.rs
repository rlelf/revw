use super::{App, FormatMode, InputMode};

impl App {
    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_buffer.clear();
        self.search_history_index = None;
        self.set_status("/");
    }

    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            self.input_mode = InputMode::Normal;
            return;
        }

        // If outline has focus, search in outline entries
        if self.outline_open && self.outline_has_focus {
            self.input_mode = InputMode::Normal;
            // Jump to first match in outline
            let search_pattern = self.search_buffer.clone();
            let entries = self.get_outline_entries();

            for i in 0..entries.len() {
                if entries[i].to_lowercase().contains(&search_pattern.to_lowercase()) {
                    let found_name = entries[i].clone();
                    self.outline_selected_index = i;
                    self.set_status(&format!("Found: {}", found_name));
                    return;
                }
            }

            self.set_status(&format!("Pattern not found: {}", search_pattern));
            return;
        }

        // If explorer has focus, search in explorer entries
        if self.explorer_open && self.explorer_has_focus {
            self.input_mode = InputMode::Normal;
            // Jump to first match in explorer
            let search_pattern = self.search_buffer.clone();

            for i in 0..self.explorer_entries.len() {
                if let Some(filename) = self.explorer_entries[i].path.file_name().and_then(|n| n.to_str()) {
                    if filename.to_lowercase().contains(&search_pattern.to_lowercase()) {
                        let found_name = filename.to_string();
                        self.explorer_selected_index = i;
                        self.explorer_update_scroll();
                        self.set_status(&format!("Found: {}", found_name));
                        return;
                    }
                }
            }

            self.set_status(&format!("Pattern not found: {}", search_pattern));
            return;
        }

        self.search_query = self.search_buffer.clone();
        self.find_matches();
        self.input_mode = InputMode::Normal;

        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.jump_to_current_match();
            self.set_status(&format!(
                "Found {} matches for '{}'",
                self.search_matches.len(),
                self.search_query
            ));
        } else {
            self.current_match_index = None;
            self.set_status(&format!("Pattern not found: {}", self.search_query));
        }
    }

    pub fn clear_search_highlight(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;
        self.set_status("Search highlight cleared");
    }

    pub fn find_matches(&mut self) {
        self.search_matches.clear();

        // For card view, search within entry content
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            let query_lower = self.search_query.to_lowercase();

            for (entry_idx, entry) in self.relf_entries.iter().enumerate() {
                for (_line_idx, line) in entry.lines.iter().enumerate() {
                    let line_lower = line.to_lowercase();
                    let mut byte_pos = 0;

                    while byte_pos < line_lower.len() {
                        if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
                            let actual_byte_pos = byte_pos + match_pos;
                            // Convert byte position to char position
                            let char_pos = line[..actual_byte_pos.min(line.len())].chars().count();
                            // Store entry_idx in line position, and char position in col position
                            self.search_matches.push((entry_idx, char_pos));
                            // Move past this match, ensuring we stay on char boundary
                            byte_pos = actual_byte_pos + query_lower.len();
                            while byte_pos < line_lower.len() && !line_lower.is_char_boundary(byte_pos) {
                                byte_pos += 1;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            return;
        }

        let search_content = if self.format_mode == FormatMode::Edit {
            &self.get_content_lines()
        } else {
            &self.rendered_content
        };

        let query_lower = self.search_query.to_lowercase();

        for (line_idx, line) in search_content.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut byte_pos = 0;

            while byte_pos < line_lower.len() {
                if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
                    let actual_byte_pos = byte_pos + match_pos;
                    // Convert byte position to char position for storage
                    let char_pos = line[..actual_byte_pos.min(line.len())].chars().count();
                    self.search_matches.push((line_idx, char_pos));
                    // Move past this match, ensuring we stay on char boundary
                    byte_pos = actual_byte_pos + query_lower.len();
                    // If we're not on a char boundary, find the next one
                    while byte_pos < line_lower.len() && !line_lower.is_char_boundary(byte_pos) {
                        byte_pos += 1;
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(&format!("No matches for '{}'", self.search_query));
            }
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let next_idx = if current_idx + 1 >= self.search_matches.len() {
            0 // Wrap to beginning
        } else {
            current_idx + 1
        };

        self.current_match_index = Some(next_idx);
        self.jump_to_current_match();
        self.set_status(&format!(
            "Match {} of {} for '{}'",
            next_idx + 1,
            self.search_matches.len(),
            self.search_query
        ));
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(&format!("No matches for '{}'", self.search_query));
            }
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.search_matches.len() - 1 // Wrap to end
        } else {
            current_idx - 1
        };

        self.current_match_index = Some(prev_idx);
        self.jump_to_current_match();
        self.set_status(&format!(
            "Match {} of {} for '{}'",
            prev_idx + 1,
            self.search_matches.len(),
            self.search_query
        ));
    }

    pub fn jump_to_current_match(&mut self) {
        if let Some(match_idx) = self.current_match_index {
            if let Some(&(line, col)) = self.search_matches.get(match_idx) {
                if self.format_mode == FormatMode::Edit {
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                } else if !self.relf_entries.is_empty() {
                    // For card view, jump to the entry
                    self.selected_entry_index = line;
                } else {
                    // For View mode, just scroll to the line
                    self.scroll = line as u16;
                    let max_scroll = self
                        .rendered_content
                        .len()
                        .saturating_sub(self.get_visible_height() as usize)
                        as u16;
                    if self.scroll > max_scroll {
                        self.scroll = max_scroll;
                    }
                    self.ensure_cursor_visible();
                }
            }
        }
    }

    /// Search forward in overlay field using last search query
    pub fn overlay_next_match(&mut self) {
        let query = if let Some(last) = self.search_history.last() {
            last.to_lowercase()
        } else {
            return;
        };

        if query.is_empty() || self.edit_field_index >= self.edit_buffer.len() {
            return;
        }

        let field = &self.edit_buffer[self.edit_field_index];
        let field_lower = field.to_lowercase();

        // Find next match after current cursor position
        let start_pos = self.edit_cursor_pos + 1;
        if let Some(rel_pos) = field_lower[field.char_indices().nth(start_pos).map(|(i, _)| i).unwrap_or(field.len())..].find(&query) {
            // Convert byte position to char position
            let byte_start = field.char_indices().nth(start_pos).map(|(i, _)| i).unwrap_or(field.len());
            let match_byte_pos = byte_start + rel_pos;
            let char_pos = field[..match_byte_pos].chars().count();
            self.edit_cursor_pos = char_pos;
            self.ensure_overlay_cursor_visible();
            return;
        }

        // Wrap around to beginning
        if let Some(rel_pos) = field_lower.find(&query) {
            let char_pos = field[..rel_pos].chars().count();
            self.edit_cursor_pos = char_pos;
            self.ensure_overlay_cursor_visible();
        }
    }

    /// Search backward in overlay field using last search query
    pub fn overlay_prev_match(&mut self) {
        let query = if let Some(last) = self.search_history.last() {
            last.to_lowercase()
        } else {
            return;
        };

        if query.is_empty() || self.edit_field_index >= self.edit_buffer.len() {
            return;
        }

        let field = &self.edit_buffer[self.edit_field_index];
        let field_lower = field.to_lowercase();

        // Find previous match before current cursor position
        let search_end = self.edit_cursor_pos;
        let search_str = &field_lower[..field.char_indices().nth(search_end).map(|(i, _)| i).unwrap_or(0)];

        if let Some(rel_pos) = search_str.rfind(&query) {
            let char_pos = field[..rel_pos].chars().count();
            self.edit_cursor_pos = char_pos;
            self.ensure_overlay_cursor_visible();
            return;
        }

        // Wrap around to end
        if let Some(rel_pos) = field_lower.rfind(&query) {
            let char_pos = field[..rel_pos].chars().count();
            self.edit_cursor_pos = char_pos;
            self.ensure_overlay_cursor_visible();
        }
    }
}
