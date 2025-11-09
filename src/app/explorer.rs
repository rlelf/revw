use super::{App, ExplorerEntry};
use std::fs;
use std::path::PathBuf;

impl App {
    pub fn toggle_explorer(&mut self) {
        self.explorer_open = !self.explorer_open;
        if self.explorer_open {
            self.load_explorer_entries();
            self.explorer_has_focus = true;
            self.explorer_horizontal_scroll = 0; // Reset horizontal scroll
        }
        self.explorer_dir_changed = true; // Signal watcher to update
    }

    pub fn load_explorer_entries(&mut self) {
        self.load_explorer_entries_with_selection_reset(true);
    }

    pub fn reload_explorer_entries(&mut self) {
        // Reload without resetting cursor position
        self.load_explorer_entries_with_selection_reset(false);
    }

    fn load_explorer_entries_with_selection_reset(&mut self, reset_selection: bool) {
        // Save currently selected path before rebuilding
        let selected_path = if !reset_selection && self.explorer_selected_index < self.explorer_entries.len() {
            Some(self.explorer_entries[self.explorer_selected_index].path.clone())
        } else {
            None
        };

        // Build tree from current directory (depth 0)
        self.explorer_entries = self.build_tree_from_dir(&self.explorer_current_dir.clone(), 0);

        if reset_selection {
            self.explorer_selected_index = 0;
            self.explorer_scroll = 0;
        } else if let Some(path) = selected_path {
            // Try to restore selection to the same path
            if let Some(new_index) = self.explorer_entries.iter().position(|e| e.path == path) {
                self.explorer_selected_index = new_index;
            } else {
                // If the path no longer exists, keep current index bounded
                self.explorer_selected_index = self.explorer_selected_index.min(self.explorer_entries.len().saturating_sub(1));
            }
        }
    }

    // Build tree structure recursively, only descending into expanded directories
    fn build_tree_from_dir(&self, dir: &PathBuf, depth: usize) -> Vec<ExplorerEntry> {
        let mut entries = Vec::new();

        // Read directory entries
        if let Ok(dir_entries) = fs::read_dir(dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in dir_entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_dir() {
                        dirs.push(path);
                    } else {
                        files.push(path);
                    }
                }
            }

            // Sort directories and files separately
            dirs.sort();
            files.sort();

            // Add directories first
            for dir_path in dirs {
                let is_expanded = self.is_directory_expanded(&dir_path);
                entries.push(ExplorerEntry {
                    path: dir_path.clone(),
                    is_expanded,
                    depth,
                });

                // If this directory is expanded, recursively add its children
                if is_expanded {
                    let children = self.build_tree_from_dir(&dir_path, depth + 1);
                    entries.extend(children);
                }
            }

            // Then add files
            for file_path in files {
                entries.push(ExplorerEntry {
                    path: file_path,
                    is_expanded: false, // Files are never expanded
                    depth,
                });
            }
        }

        entries
    }

    // Check if a directory is currently expanded in the tree
    fn is_directory_expanded(&self, dir_path: &PathBuf) -> bool {
        self.explorer_entries
            .iter()
            .find(|e| &e.path == dir_path)
            .map(|e| e.is_expanded)
            .unwrap_or(false)
    }

    pub fn explorer_move_up(&mut self) {
        if self.explorer_selected_index > 0 {
            self.explorer_selected_index -= 1;
            // Auto-scroll if selection moves above visible area
            if self.explorer_selected_index < self.explorer_scroll as usize {
                self.explorer_scroll = self.explorer_selected_index as u16;
            }
        }
    }

    pub fn explorer_move_down(&mut self) {
        if self.explorer_selected_index + 1 < self.explorer_entries.len() {
            self.explorer_selected_index += 1;
            // Auto-scroll if selection moves below visible area
            // Note: visible_height is calculated in UI, use a reasonable default
            let visible_height = self.visible_height.max(10) as usize; // Use app's visible_height as approximation
            if self.explorer_selected_index >= (self.explorer_scroll as usize + visible_height) {
                self.explorer_scroll = (self.explorer_selected_index - visible_height + 1) as u16;
            }
        }
    }

    pub fn explorer_select_entry(&mut self) {
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];

            if selected.path.is_dir() {
                // Toggle expand/collapse for directories
                self.toggle_directory_expansion();
            } else if selected.path.is_file() {
                // Open file
                if let Some(extension) = selected.path.extension() {
                    if extension == "json" || extension == "md" {
                        // Use load_file to properly reset all cursor positions
                        self.load_file(selected.path.clone());
                        // Move focus to file window
                        self.explorer_has_focus = false;
                    } else {
                        self.set_status(&format!("Error: Only JSON and Markdown files can be opened ({})", selected.path.display()));
                    }
                } else {
                    self.set_status(&format!("Error: Only JSON and Markdown files can be opened ({})", selected.path.display()));
                }
            }
        }
    }

    // Toggle expansion state of currently selected directory
    fn toggle_directory_expansion(&mut self) {
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected_path = self.explorer_entries[self.explorer_selected_index].path.clone();

            if selected_path.is_dir() {
                // Toggle the expansion state
                let new_state = !self.explorer_entries[self.explorer_selected_index].is_expanded;
                self.explorer_entries[self.explorer_selected_index].is_expanded = new_state;

                // Rebuild the tree to reflect the change
                self.explorer_entries = self.build_tree_from_dir(&self.explorer_current_dir.clone(), 0);

                // Try to keep selection on the same path
                if let Some(new_index) = self.explorer_entries.iter().position(|e| e.path == selected_path) {
                    self.explorer_selected_index = new_index;
                }
            }
        }
    }

    pub fn explorer_preview_entry(&mut self) {
        // Like NERDTree's 'go' command - preview without moving focus
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];

            if selected.path.is_file() {
                if let Some(extension) = selected.path.extension() {
                    if extension == "json" || extension == "md" {
                        // Use load_file to properly reset all cursor positions
                        self.load_file(selected.path.clone());
                        // Keep focus on explorer (unlike Enter which moves focus)
                    } else {
                        self.set_status(&format!("Error: Only JSON and Markdown files can be opened ({})", selected.path.display()));
                    }
                } else {
                    self.set_status(&format!("Error: Only JSON and Markdown files can be opened ({})", selected.path.display()));
                }
            }
        }
    }

    // Get the directory where a new file/folder should be created based on cursor position
    pub fn get_target_directory(&self) -> PathBuf {
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];

            if selected.path.is_dir() {
                // If cursor is on a directory, create inside it
                selected.path.clone()
            } else {
                // If cursor is on a file, create in its parent directory
                selected.path.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| self.explorer_current_dir.clone())
            }
        } else {
            // Default to current directory
            self.explorer_current_dir.clone()
        }
    }

    pub fn switch_window_focus(&mut self) {
        if self.explorer_open {
            self.explorer_has_focus = !self.explorer_has_focus;
        }
    }

    pub fn focus_explorer(&mut self) {
        if self.explorer_open {
            self.explorer_has_focus = true;
        }
    }

    pub fn focus_file(&mut self) {
        if self.explorer_open {
            self.explorer_has_focus = false;
        }
    }

    pub fn explorer_update_scroll(&mut self) {
        // Update scroll to keep selected item visible
        let visible_height = self.visible_height.max(10) as usize;
        if self.explorer_selected_index < self.explorer_scroll as usize {
            self.explorer_scroll = self.explorer_selected_index as u16;
        } else if self.explorer_selected_index >= (self.explorer_scroll as usize + visible_height) {
            self.explorer_scroll = (self.explorer_selected_index - visible_height + 1) as u16;
        }
    }

    pub fn explorer_next_match(&mut self) {
        // Use the last search from search history
        let search_pattern = if !self.search_history.is_empty() {
            self.search_history.last().unwrap().clone()
        } else {
            return;
        };

        if search_pattern.is_empty() {
            return;
        }

        let start_index = self.explorer_selected_index + 1;
        for i in start_index..self.explorer_entries.len() {
            if let Some(filename) = self.explorer_entries[i].path.file_name().and_then(|n| n.to_str()) {
                if filename.to_lowercase().contains(&search_pattern.to_lowercase()) {
                    self.explorer_selected_index = i;
                    self.explorer_update_scroll();
                    return;
                }
            }
        }

        // Wrap around to beginning
        for i in 0..start_index {
            if let Some(filename) = self.explorer_entries[i].path.file_name().and_then(|n| n.to_str()) {
                if filename.to_lowercase().contains(&search_pattern.to_lowercase()) {
                    self.explorer_selected_index = i;
                    self.explorer_update_scroll();
                    return;
                }
            }
        }
    }

    pub fn explorer_prev_match(&mut self) {
        // Use the last search from search history
        let search_pattern = if !self.search_history.is_empty() {
            self.search_history.last().unwrap().clone()
        } else {
            return;
        };

        if search_pattern.is_empty() {
            return;
        }

        let start_index = if self.explorer_selected_index > 0 {
            self.explorer_selected_index - 1
        } else {
            self.explorer_entries.len().saturating_sub(1)
        };

        // Search backwards from start_index to 0
        for i in (0..=start_index).rev() {
            if let Some(filename) = self.explorer_entries[i].path.file_name().and_then(|n| n.to_str()) {
                if filename.to_lowercase().contains(&search_pattern.to_lowercase()) {
                    self.explorer_selected_index = i;
                    self.explorer_update_scroll();
                    return;
                }
            }
        }

        // Wrap around to end
        for i in (start_index + 1..self.explorer_entries.len()).rev() {
            if let Some(filename) = self.explorer_entries[i].path.file_name().and_then(|n| n.to_str()) {
                if filename.to_lowercase().contains(&search_pattern.to_lowercase()) {
                    self.explorer_selected_index = i;
                    self.explorer_update_scroll();
                    return;
                }
            }
        }
    }
}
