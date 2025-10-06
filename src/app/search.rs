use super::{App, FormatMode, InputMode};

impl App {
    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_buffer.clear();
        self.set_status("/");
    }

    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            self.input_mode = InputMode::Normal;
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
            &self.get_json_lines()
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

}
