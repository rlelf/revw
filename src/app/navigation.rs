use super::{App, FormatMode};
use super::super::json_ops::JsonOperations;
use super::super::navigation::Navigator;
use serde_json::Value;

impl App {
    pub fn relf_is_entry_start(&self, line: &str) -> bool {
        Navigator::relf_is_entry_start(line)
    }
    pub fn relf_is_boundary(&self, line: &str) -> bool {
        Navigator::relf_is_boundary(line)
    }
    pub fn relf_jump_down(&mut self) {
        if self.rendered_content.is_empty() {
            return;
        }
        let curr = self.scroll as usize;
        let mut i = curr.saturating_add(1);
        let lim = 12usize.min(self.get_visible_height() as usize); // keep jumps modest
        let mut steps = 0usize;
        // Prefer entry start first
        while i < self.rendered_content.len() && steps < lim {
            if self.relf_is_entry_start(&self.rendered_content[i]) {
                let max_scroll = self.relf_content_max_scroll();
                self.scroll = (i as u16).min(max_scroll);
                return;
            }
            i += 1;
            steps += 1;
        }
        // Fallback to other boundaries (blank/header)
        i = curr.saturating_add(1);
        steps = 0;
        while i < self.rendered_content.len() && steps < lim {
            if self.relf_is_boundary(&self.rendered_content[i]) {
                let max_scroll = self.relf_content_max_scroll();
                self.scroll = (i as u16).min(max_scroll);
                return;
            }
            i += 1;
            steps += 1;
        }
        // Fallback: move down but never beyond the last content page
        let content_max = self.relf_content_max_scroll();
        if self.scroll < content_max {
            self.scroll += 1;
        }
    }

    pub fn relf_jump_up(&mut self) {
        if self.rendered_content.is_empty() {
            return;
        }
        let lim = 12isize.min(self.get_visible_height() as isize);
        let mut i = self.scroll as isize - 1;
        let mut steps = 0isize;
        // Prefer entry start first
        while i >= 0 && steps < lim {
            if self.relf_is_entry_start(&self.rendered_content[i as usize]) {
                let max_scroll = self.relf_content_max_scroll();
                let target = i as u16;
                self.scroll = std::cmp::min(target, max_scroll);
                return;
            }
            i -= 1;
            steps += 1;
        }
        // Fallback to other boundaries
        i = self.scroll as isize - 1;
        steps = 0;
        while i >= 0 && steps < lim {
            if self.relf_is_boundary(&self.rendered_content[i as usize]) {
                let max_scroll = self.relf_content_max_scroll();
                let target = i as u16;
                self.scroll = std::cmp::min(target, max_scroll);
                return;
            }
            i -= 1;
            steps += 1;
        }
        self.scroll_up();
    }

    pub fn relf_max_hscroll(&self) -> u16 {
        let w = self.get_content_width() as usize;
        let mut max_cols = 0usize;
        for l in &self.rendered_content {
            let cols = self.display_width_str(l);
            if cols > max_cols {
                max_cols = cols;
            }
        }
        if max_cols > w {
            (max_cols - w) as u16
        } else {
            0
        }
    }

    pub fn relf_content_max_scroll(&self) -> u16 {
        let total = self.rendered_content.len() as u16;
        let vis = self.get_visible_height();
        total.saturating_sub(vis)
    }

    pub fn relf_hscroll_by(&mut self, delta: i16) {
        let max_off = self.relf_max_hscroll();
        if delta >= 0 {
            let d = delta as u16;
            self.hscroll = (self.hscroll.saturating_add(d)).min(max_off);
        } else {
            let d = (-delta) as u16;
            self.hscroll = self.hscroll.saturating_sub(d);
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    pub fn page_up(&mut self) {
        // In Edit mode, move the cursor up by a full page of visual lines
        if self.format_mode == FormatMode::Edit {
            let count = self.get_visible_height() as usize;
            for _ in 0..count {
                self.move_cursor_up();
            }
        } else {
            self.scroll = self.scroll.saturating_sub(self.get_visible_height());
        }
    }

    pub fn page_down(&mut self) {
        // In Edit mode, move the cursor down by a full page of visual lines
        if self.format_mode == FormatMode::Edit {
            let count = self.get_visible_height() as usize;
            for _ in 0..count {
                self.move_cursor_down();
            }
        } else {
            let vis = self.get_visible_height();
            self.scroll = (self.scroll + vis).min(self.max_scroll);
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll;
    }

    pub fn delete_selected_entry(&mut self) {
        // Delete the selected entry from relf_entries by removing it from JSON
        if self.relf_entries.is_empty() || self.selected_entry_index >= self.relf_entries.len() {
            self.set_status("No entry to delete");
            return;
        }

        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut json_value) => {
                if let Some(obj) = json_value.as_object_mut() {
                    // Count entries to find which section and index
                    let mut current_idx = 0;
                    let target_idx = self.selected_entry_index;
                    let mut found = false;

                    // Check outside section first
                    if let Some(outside) = obj.get_mut("outside") {
                        if let Some(outside_array) = outside.as_array_mut() {
                            let outside_count = outside_array.len();
                            if target_idx < current_idx + outside_count {
                                let local_idx = target_idx - current_idx;
                                outside_array.remove(local_idx);
                                found = true;
                            } else {
                                current_idx += outside_count;
                            }
                        }
                    }

                    // Check inside section if not found
                    if !found {
                        if let Some(inside) = obj.get_mut("inside") {
                            if let Some(inside_array) = inside.as_array_mut() {
                                let local_idx = target_idx - current_idx;
                                if local_idx < inside_array.len() {
                                    inside_array.remove(local_idx);
                                    found = true;
                                }
                            }
                        }
                    }

                    if found {
                        // Update JSON and re-render
                        match serde_json::to_string_pretty(&json_value) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.convert_json();

                                // Move selection up (to previous entry)
                                if !self.relf_entries.is_empty() {
                                    if self.selected_entry_index > 0 {
                                        self.selected_entry_index -= 1;
                                    } else if self.selected_entry_index >= self.relf_entries.len() {
                                        self.selected_entry_index = self.relf_entries.len() - 1;
                                    }
                                }

                                self.set_status("Entry deleted");
                            }
                            Err(e) => self.set_status(&format!("Error formatting JSON: {}", e)),
                        }
                    } else {
                        self.set_status("Could not find entry to delete");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn delete_current_entry(&mut self) {
        // Save undo state before modification
        self.save_undo_state();

        let lines = self.get_json_lines();
        match JsonOperations::delete_entry_at_cursor(
            &self.json_input,
            self.content_cursor_line,
            &lines,
        ) {
            Ok((formatted, message)) => {
                self.json_input = formatted;
                self.convert_json();

                // Adjust cursor position
                let new_lines = self.get_json_lines();
                if self.content_cursor_line >= new_lines.len() && !new_lines.is_empty() {
                    self.content_cursor_line = new_lines.len() - 1;
                }
                self.content_cursor_col = 0;
                self.ensure_cursor_visible();
                self.set_status(&message);
            }
            Err(e) => self.set_status(&e),
        }
    }

    pub fn jump_to_first_outside(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // In Edit mode, find the first outside entry
            let lines = self.get_json_lines();
            for (i, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("\"outside\"") {
                    // Move to the first entry after "outside": [
                    if i + 1 < lines.len() {
                        self.content_cursor_line = i + 1;
                        self.content_cursor_col = 0;
                        self.ensure_cursor_visible();
                        self.set_status("Jumped to first OUTSIDE entry");
                        return;
                    }
                }
            }
            self.set_status("No OUTSIDE entries found");
        } else {
            // In View mode, jump to first card in outside section
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    if let Some(outside) = obj.get("outside") {
                        if let Some(outside_array) = outside.as_array() {
                            if !outside_array.is_empty() {
                                self.selected_entry_index = 0;
                                self.set_status("Jumped to first OUTSIDE entry");
                                return;
                            }
                        }
                    }
                }
            }
            self.set_status("No OUTSIDE entries found");
        }
    }

    pub fn jump_to_first_inside(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // In Edit mode, find the first inside entry
            let lines = self.get_json_lines();
            for (i, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("\"inside\"") {
                    // Move to the first entry after "inside": [
                    if i + 1 < lines.len() {
                        self.content_cursor_line = i + 1;
                        self.content_cursor_col = 0;
                        self.ensure_cursor_visible();
                        self.set_status("Jumped to first INSIDE entry");
                        return;
                    }
                }
            }
            self.set_status("No INSIDE entries found");
        } else {
            // In View mode, jump to first card in inside section
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    if let Some(inside) = obj.get("inside") {
                        if let Some(inside_array) = inside.as_array() {
                            if !inside_array.is_empty() && outside_count < self.relf_entries.len() {
                                self.selected_entry_index = outside_count;
                                self.set_status("Jumped to first INSIDE entry");
                                return;
                            }
                        }
                    }
                }
            }
            self.set_status("No INSIDE entries found");
        }
    }

    pub fn move_to_next_word_end(&mut self) {
        // Vim-like 'e': always make forward progress to the end of the next word
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self
            .content_cursor_line
            .min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        // Iterator: advance one position forward from (li, ci)
        let next_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() {
                return None;
            }
            // move to next char on the same line
            if ci2 + 1 < line_chars[li2].len() {
                return Some((li2, ci2 + 1, line_chars[li2][ci2 + 1]));
            }
            // otherwise, jump to the first char of the next non-empty line
            li2 += 1;
            while li2 < line_chars.len() {
                if !line_chars[li2].is_empty() {
                    return Some((li2, 0, line_chars[li2][0]));
                }
                li2 += 1;
            }
            None
        };

        // Start scanning strictly after current position to guarantee progress
        let mut in_word = false;
        let mut saw_any_word = false;
        while let Some((nli, nci, ch)) = next_pos(li, ci) {
            if is_word(ch) {
                saw_any_word = true;
                in_word = true; // we are inside a word
            } else if in_word {
                // We just stepped onto a non-word after a run of word chars.
                // li,ci still point to the last word char from the previous iteration.
                self.content_cursor_line = li;
                self.content_cursor_col = ci;
                self.ensure_cursor_visible();
                return;
            }
            // advance current position
            li = nli;
            ci = nci;
        }

        // Reached EOF: if we were inside a word, li,ci are at the last word char
        if !saw_any_word {
            // No more words: place at last char of file if any
            if let Some(last_line) = line_chars.len().checked_sub(1) {
                let last_col = line_chars[last_line].len();
                self.content_cursor_line = last_line;
                self.content_cursor_col = last_col.saturating_sub(1);
            }
        } else {
            self.content_cursor_line = li;
            self.content_cursor_col = ci;
        }
        self.ensure_cursor_visible();
    }

    pub fn move_to_previous_word_start(&mut self) {
        // Vim-like 'b': always make backward progress to the start of the previous word
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self
            .content_cursor_line
            .min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        let prev_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() {
                return None;
            }
            if ci2 > 0 {
                return Some((li2, ci2 - 1, line_chars[li2][ci2 - 1]));
            }
            if li2 == 0 {
                return None;
            }
            li2 -= 1;
            while let Some(line) = line_chars.get(li2) {
                if !line.is_empty() {
                    return Some((li2, line.len() - 1, line[line.len() - 1]));
                }
                if li2 == 0 {
                    break;
                }
                li2 -= 1;
            }
            None
        };

        if li == 0 && ci == 0 {
            return;
        }

        // Start scanning strictly before current position to guarantee progress
        let mut in_word = false;
        let mut start_li = li;
        let mut start_ci = ci; // will hold the start index of the found word
        let mut saw_any_word = false;
        while let Some((pli, pci, ch)) = prev_pos(li, ci) {
            if is_word(ch) {
                saw_any_word = true;
                start_li = pli;
                start_ci = pci; // keep updating until we leave the word
                in_word = true;
            } else {
                if in_word {
                    // We just left a word while moving left; current saved pos is the word start
                    self.content_cursor_line = start_li;
                    self.content_cursor_col = start_ci;
                    self.ensure_cursor_visible();
                    return;
                }
            }
            li = pli;
            ci = pci;
        }

        // Reached BOF
        if saw_any_word {
            self.content_cursor_line = start_li;
            self.content_cursor_col = start_ci;
        } else {
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
        }
        self.ensure_cursor_visible();
    }

}
