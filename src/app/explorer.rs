use super::App;
use std::fs;

impl App {
    pub fn toggle_explorer(&mut self) {
        self.explorer_open = !self.explorer_open;
        if self.explorer_open {
            self.load_explorer_entries();
            self.set_status("Explorer opened");
        } else {
            self.set_status("Explorer closed");
        }
    }

    pub fn load_explorer_entries(&mut self) {
        let mut entries = Vec::new();

        // Add parent directory entry if not at root
        if self.explorer_current_dir.parent().is_some() {
            entries.push(self.explorer_current_dir.join(".."));
        }

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
                // Navigate into directory
                if let Ok(canonical) = selected.canonicalize() {
                    self.explorer_current_dir = canonical;
                    self.load_explorer_entries();
                }
            } else if selected.is_file() {
                // Open file
                if let Some(extension) = selected.extension() {
                    if extension == "json" {
                        self.file_path = Some(selected.clone());
                        self.file_path_changed = true;
                        if let Ok(content) = fs::read_to_string(&selected) {
                            self.json_input = content;
                            self.convert_json();
                            self.set_status(&format!("Loaded: {}", selected.display()));
                        }
                    } else {
                        self.set_status("Only JSON files can be opened");
                    }
                } else {
                    self.set_status("Only JSON files can be opened");
                }
            }
        }
    }
}
