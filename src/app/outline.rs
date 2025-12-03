use super::{App, FormatMode};

impl App {
    pub fn toggle_outline(&mut self) {
        if self.outline_open {
            // Close outline
            self.outline_open = false;
            self.outline_selected_index = 0;
            self.outline_scroll = 0;
            self.outline_horizontal_scroll = 0;
            self.outline_has_focus = false;
            // Clear search state
            self.outline_search_query.clear();
            self.outline_search_matches.clear();
            self.outline_search_current = 0;
        } else {
            // Open outline (reset cursor to top)
            self.outline_open = true;
            self.outline_selected_index = 0;
            self.outline_scroll = 0;
            self.outline_horizontal_scroll = 0;
            self.outline_has_focus = false;
            self.outline_opened_from_explorer = self.explorer_open && self.explorer_has_focus;
        }
    }

    /// Preview entry from outline without closing (like go in explorer)
    pub fn outline_preview_entry(&mut self) {
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            // Jump to selected card in View mode without closing outline
            if self.outline_selected_index < self.relf_entries.len() {
                self.selected_entry_index = self.outline_selected_index;
            }
        } else if self.format_mode == FormatMode::Edit {
            // Jump to selected entry in Edit mode without closing outline
            if let Some(line) = self.get_entry_start_line(self.outline_selected_index) {
                self.content_cursor_line = line;
                self.content_cursor_col = 0;
                self.ensure_cursor_visible();
            }
        }
    }

    pub fn outline_move_up(&mut self) {
        if self.outline_selected_index > 0 {
            self.outline_selected_index -= 1;
        }
    }

    pub fn outline_move_down(&mut self) {
        let max_index = if self.format_mode == FormatMode::View {
            self.relf_entries.len().saturating_sub(1)
        } else {
            // In Edit mode, get entry count from markdown/json
            self.get_entry_count_from_content().saturating_sub(1)
        };

        if self.outline_selected_index < max_index {
            self.outline_selected_index += 1;
        }
    }

    pub fn outline_page_down(&mut self) {
        let max_index = if self.format_mode == FormatMode::View {
            self.relf_entries.len().saturating_sub(1)
        } else {
            self.get_entry_count_from_content().saturating_sub(1)
        };

        // Move down by 10 entries (or to the end)
        let new_index = (self.outline_selected_index + 10).min(max_index);
        self.outline_selected_index = new_index;
    }

    pub fn outline_page_up(&mut self) {
        // Move up by 10 entries (or to the beginning)
        self.outline_selected_index = self.outline_selected_index.saturating_sub(10);
    }

    pub fn outline_jump_to_selected(&mut self) {
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            // Jump to selected card in View mode (keep outline open)
            if self.outline_selected_index < self.relf_entries.len() {
                self.selected_entry_index = self.outline_selected_index;
                // Reset horizontal scroll when jumping to new card
                self.hscroll = 0;
            }
        } else if self.format_mode == FormatMode::Edit {
            // Jump to selected entry in Edit mode (keep outline open)
            if let Some(line) = self.get_entry_start_line(self.outline_selected_index) {
                self.content_cursor_line = line;
                self.content_cursor_col = 0;
                self.ensure_cursor_visible();
            }
        }
    }

    pub fn get_entry_count_from_content(&self) -> usize {
        if self.is_markdown_file() {
            // Count ### headers in markdown, excluding code blocks
            let mut in_code_block = false;
            let mut count = 0;
            for line in self.markdown_input.lines() {
                // Track code block state
                if line.trim_start().starts_with("```") {
                    in_code_block = !in_code_block;
                    continue;
                }

                // Skip lines inside code blocks
                if in_code_block {
                    continue;
                }

                if line.trim_start().starts_with("### ") {
                    count += 1;
                }
            }
            count
        } else {
            // Count entries in JSON (outside + inside)
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj.get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);
                    let inside_count = obj.get("inside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);
                    return outside_count + inside_count;
                }
            }
            0
        }
    }

    fn get_entry_start_line(&self, entry_index: usize) -> Option<usize> {
        if self.is_markdown_file() {
            // Find the nth ### header, excluding code blocks
            let lines = self.markdown_input.lines().collect::<Vec<_>>();
            let mut in_code_block = false;
            let mut count = 0;
            for (i, line) in lines.iter().enumerate() {
                // Track code block state
                if line.trim_start().starts_with("```") {
                    in_code_block = !in_code_block;
                    continue;
                }

                // Skip lines inside code blocks
                if in_code_block {
                    continue;
                }

                if line.trim_start().starts_with("### ") {
                    if count == entry_index {
                        return Some(i);
                    }
                    count += 1;
                }
            }
        } else {
            // For JSON, find entry in rendered content
            // This is approximate but should work
            let lines = self.json_input.lines().collect::<Vec<_>>();
            let mut entries_found = 0;

            // Look for name/date fields which indicate entry boundaries
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if (trimmed.starts_with("\"name\":") || trimmed.starts_with("\"date\":"))
                    && entries_found == entry_index {
                    return Some(i);
                }
                if trimmed.starts_with("\"name\":") || trimmed.starts_with("\"date\":") {
                    entries_found += 1;
                }
            }
        }
        None
    }

    pub fn get_outline_entries(&self) -> Vec<String> {
        let mut entries = Vec::new();

        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            // Use relf_entries for View mode
            for entry in self.relf_entries.iter() {
                // Get the first line as the title/summary
                let title = entry.lines.first()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "".to_string());

                // Truncate if too long
                let display_title = if title.len() > 80 {
                    // Use char_indices to safely truncate at UTF-8 boundary
                    let truncate_at = title.char_indices()
                        .take(77)
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    format!("{}...", &title[..truncate_at])
                } else {
                    title
                };

                entries.push(display_title);
            }
        } else if self.format_mode == FormatMode::Edit {
            // Parse from markdown or JSON
            if self.is_markdown_file() {
                let mut in_code_block = false;
                for line in self.markdown_input.lines() {
                    // Track code block state
                    if line.trim_start().starts_with("```") {
                        in_code_block = !in_code_block;
                        continue;
                    }

                    // Skip lines inside code blocks
                    if in_code_block {
                        continue;
                    }

                    if line.trim_start().starts_with("### ") {
                        let title = line.trim_start()[4..].trim();
                        let display_title = if title.len() > 80 {
                            // Use char_indices to safely truncate at UTF-8 boundary
                            let truncate_at = title.char_indices()
                                .take(77)
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            format!("{}...", &title[..truncate_at])
                        } else {
                            title.to_string()
                        };
                        entries.push(display_title);
                    }
                }
            } else {
                // Parse JSON
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&self.json_input) {
                    if let Some(obj) = json_value.as_object() {
                        // Add OUTSIDE entries
                        if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                            for item in outside {
                                if let Some(item_obj) = item.as_object() {
                                    let name = item_obj.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unnamed");
                                    let display_title = if name.len() > 80 {
                                        // Use char_indices to safely truncate at UTF-8 boundary
                                        let truncate_at = name.char_indices()
                                            .take(77)
                                            .last()
                                            .map(|(i, _)| i)
                                            .unwrap_or(0);
                                        format!("{}...", &name[..truncate_at])
                                    } else {
                                        name.to_string()
                                    };
                                    entries.push(display_title);
                                }
                            }
                        }

                        // Add INSIDE entries
                        if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                            for item in inside {
                                if let Some(item_obj) = item.as_object() {
                                    let date = item_obj.get("date")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("No date");
                                    let display_title = if date.len() > 80 {
                                        // Use char_indices to safely truncate at UTF-8 boundary
                                        let truncate_at = date.char_indices()
                                            .take(77)
                                            .last()
                                            .map(|(i, _)| i)
                                            .unwrap_or(0);
                                        format!("{}...", &date[..truncate_at])
                                    } else {
                                        date.to_string()
                                    };
                                    entries.push(display_title);
                                }
                            }
                        }
                    }
                }
            }
        }

        entries
    }

    pub fn outline_next_match(&mut self) {
        // Use the last search from search history
        let search_pattern = if !self.search_history.is_empty() {
            self.search_history.last().unwrap().clone()
        } else {
            return;
        };

        if search_pattern.is_empty() {
            return;
        }

        let entries = self.get_outline_entries();
        let start_index = self.outline_selected_index + 1;

        // Search forward from current position
        for i in start_index..entries.len() {
            if entries[i].to_lowercase().contains(&search_pattern.to_lowercase()) {
                self.outline_selected_index = i;
                return;
            }
        }

        // Wrap around to beginning
        for i in 0..start_index {
            if entries[i].to_lowercase().contains(&search_pattern.to_lowercase()) {
                self.outline_selected_index = i;
                return;
            }
        }
    }

    pub fn outline_prev_match(&mut self) {
        // Use the last search from search history
        let search_pattern = if !self.search_history.is_empty() {
            self.search_history.last().unwrap().clone()
        } else {
            return;
        };

        if search_pattern.is_empty() {
            return;
        }

        let entries = self.get_outline_entries();
        let start_index = if self.outline_selected_index > 0 {
            self.outline_selected_index - 1
        } else {
            entries.len().saturating_sub(1)
        };

        // Search backwards from start_index to 0
        for i in (0..=start_index).rev() {
            if entries[i].to_lowercase().contains(&search_pattern.to_lowercase()) {
                self.outline_selected_index = i;
                return;
            }
        }

        // Wrap around to end
        for i in (start_index + 1..entries.len()).rev() {
            if entries[i].to_lowercase().contains(&search_pattern.to_lowercase()) {
                self.outline_selected_index = i;
                return;
            }
        }
    }
}
