use super::App;
use std::fs;

impl App {
    pub fn toggle_explorer(&mut self) {
        self.explorer_open = !self.explorer_open;
        if self.explorer_open {
            self.load_explorer_entries();
            self.explorer_has_focus = true;
        }
    }

    pub fn load_explorer_entries(&mut self) {
        let mut entries = Vec::new();

        // Don't add parent directory - keep flat navigation
        // Read directory entries
        if let Ok(dir_entries) = fs::read_dir(&self.explorer_current_dir) {
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

            // Add directories first, then files
            entries.extend(dirs);
            entries.extend(files);
        }

        self.explorer_entries = entries;
        self.explorer_selected_index = 0;
        self.explorer_scroll = 0;
    }

    pub fn explorer_move_up(&mut self) {
        if self.explorer_selected_index > 0 {
            self.explorer_selected_index -= 1;
        }
    }

    pub fn explorer_move_down(&mut self) {
        if self.explorer_selected_index + 1 < self.explorer_entries.len() {
            self.explorer_selected_index += 1;
        }
    }

    pub fn explorer_select_entry(&mut self) {
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = self.explorer_entries[self.explorer_selected_index].clone();

            if selected.is_dir() {
                // Navigate into directory (but don't implement this - keep flat)
                // Do nothing for directories
            } else if selected.is_file() {
                // Open file
                if let Some(extension) = selected.extension() {
                    if extension == "json" {
                        self.file_path = Some(selected.clone());
                        self.file_path_changed = true;
                        if let Ok(content) = fs::read_to_string(&selected) {
                            self.json_input = content;
                            self.convert_json();
                            // Move focus to file window
                            self.explorer_has_focus = false;
                        }
                    }
                }
            }
        }
    }

    pub fn explorer_preview_entry(&mut self) {
        // Like NERDTree's 'go' command - preview without moving focus
        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = self.explorer_entries[self.explorer_selected_index].clone();

            if selected.is_file() {
                if let Some(extension) = selected.extension() {
                    if extension == "json" {
                        self.file_path = Some(selected.clone());
                        self.file_path_changed = true;
                        if let Ok(content) = fs::read_to_string(&selected) {
                            self.json_input = content;
                            self.convert_json();
                            // Keep focus on explorer
                        }
                    }
                }
            }
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
}
