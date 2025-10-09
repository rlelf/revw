use super::{App, FileOperation};
use std::fs;

impl App {
    // Start delete file operation
    pub fn explorer_delete_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = self.explorer_entries[self.explorer_selected_index].clone();
            if selected.is_file() {
                let filename = selected.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                self.file_op_pending = Some(FileOperation::Delete(selected));
                self.set_status(&format!("Delete '{}'? (y/n)", filename));
            } else {
                self.set_status("Cannot delete directories");
            }
        }
    }

    // Start copy file operation
    pub fn explorer_copy_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = self.explorer_entries[self.explorer_selected_index].clone();
            if selected.is_file() {
                self.file_op_pending = Some(FileOperation::Copy(selected.clone()));
                self.file_op_prompt_buffer = String::new();
                let filename = selected.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                self.set_status(&format!("Copy '{}' to (must end with .json):", filename));
            } else {
                self.set_status("Cannot copy directories");
            }
        }
    }

    // Start rename file operation
    pub fn explorer_rename_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = self.explorer_entries[self.explorer_selected_index].clone();
            if selected.is_file() {
                self.file_op_pending = Some(FileOperation::Rename(selected.clone()));
                // Pre-fill with current filename
                self.file_op_prompt_buffer = selected.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                self.set_status("Rename to (must end with .json):");
            } else {
                self.set_status("Cannot rename directories");
            }
        }
    }

    // Start create new file operation
    pub fn explorer_create_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        self.file_op_pending = Some(FileOperation::Create);
        self.file_op_prompt_buffer = String::new();
        self.set_status("New file name (must end with .json):");
    }

    // Handle confirmation for delete operation (y/n)
    pub fn handle_file_op_confirmation(&mut self, response: char) {
        if let Some(FileOperation::Delete(path)) = self.file_op_pending.clone() {
            if response == 'y' || response == 'Y' {
                // Perform delete
                match fs::remove_file(&path) {
                    Ok(()) => {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        self.set_status(&format!("Deleted '{}'", filename));
                        // Reload explorer
                        self.load_explorer_entries();
                    }
                    Err(e) => {
                        self.set_status(&format!("Error deleting file: {}", e));
                    }
                }
            } else {
                self.set_status("Delete cancelled");
            }
            self.file_op_pending = None;
        }
    }

    // Execute file operation based on prompt buffer
    pub fn execute_file_operation(&mut self) {
        let filename = self.file_op_prompt_buffer.trim().to_string();

        // Validate .json extension
        if !filename.ends_with(".json") {
            self.set_status("Error: Filename must end with .json");
            self.file_op_pending = None;
            self.file_op_prompt_buffer.clear();
            return;
        }

        match self.file_op_pending.clone() {
            Some(FileOperation::Create) => {
                // Create new file in current directory
                let new_path = self.explorer_current_dir.join(&filename);

                if new_path.exists() {
                    self.set_status(&format!("Error: File '{}' already exists", filename));
                } else {
                    // Create file with default JSON content
                    let now = chrono::Local::now();
                    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    let default_json = serde_json::json!({
                        "outside": [
                            {
                                "name": "",
                                "context": "",
                                "url": "",
                                "percentage": null
                            }
                        ],
                        "inside": [
                            {
                                "date": timestamp,
                                "context": ""
                            }
                        ]
                    });

                    let default_content = serde_json::to_string_pretty(&default_json)
                        .unwrap_or_else(|_| String::from(r#"{"outside":[],"inside":[]}"#));

                    match fs::write(&new_path, default_content) {
                        Ok(()) => {
                            self.set_status(&format!("Created '{}'", filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error creating file: {}", e));
                        }
                    }
                }
            }
            Some(FileOperation::Copy(source_path)) => {
                // Copy file to new location in current directory
                let dest_path = self.explorer_current_dir.join(&filename);

                if dest_path.exists() {
                    self.set_status(&format!("Error: File '{}' already exists", filename));
                } else {
                    match fs::copy(&source_path, &dest_path) {
                        Ok(_) => {
                            let source_filename = source_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            self.set_status(&format!("Copied '{}' to '{}'", source_filename, filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error copying file: {}", e));
                        }
                    }
                }
            }
            Some(FileOperation::Rename(old_path)) => {
                // Rename file in current directory
                let new_path = self.explorer_current_dir.join(&filename);

                if new_path.exists() && new_path != old_path {
                    self.set_status(&format!("Error: File '{}' already exists", filename));
                } else {
                    match fs::rename(&old_path, &new_path) {
                        Ok(()) => {
                            let old_filename = old_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            self.set_status(&format!("Renamed '{}' to '{}'", old_filename, filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error renaming file: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }

        self.file_op_pending = None;
        self.file_op_prompt_buffer.clear();
    }

    // Cancel pending file operation
    pub fn cancel_file_operation(&mut self) {
        self.file_op_pending = None;
        self.file_op_prompt_buffer.clear();
        self.set_status("Operation cancelled");
    }
}
