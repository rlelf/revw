use super::{App, FileOperation};
use std::fs;
use std::path::Path;

// Helper function to recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

impl App {
    // Start delete file/directory operation
    pub fn explorer_delete_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];
            let name = selected.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let item_type = if selected.path.is_dir() { "directory" } else { "file" };
            self.file_op_pending = Some(FileOperation::Delete(selected.path.clone()));
            self.set_status(&format!("Delete {} '{}'? (y/n)", item_type, name));
        }
    }

    // Start copy file/directory operation
    pub fn explorer_copy_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];
            self.file_op_pending = Some(FileOperation::Copy(selected.path.clone()));
            self.file_op_prompt_buffer = String::new();
            let name = selected.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if selected.path.is_dir() {
                self.set_status(&format!("Copy directory '{}' to:", name));
            } else {
                self.set_status(&format!("Copy '{}' to (must end with .json):", name));
            }
        }
    }

    // Start rename file/directory operation
    pub fn explorer_rename_file(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        if self.explorer_selected_index < self.explorer_entries.len() {
            let selected = &self.explorer_entries[self.explorer_selected_index];
            self.file_op_pending = Some(FileOperation::Rename(selected.path.clone()));
            // Pre-fill with current filename/directory name
            self.file_op_prompt_buffer = selected.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if selected.path.is_dir() {
                self.set_status("Rename directory to:");
            } else {
                self.set_status("Rename to (must end with .json):");
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

    // Start create new directory operation
    pub fn explorer_create_dir(&mut self) {
        if !self.explorer_open || !self.explorer_has_focus {
            return;
        }

        self.file_op_pending = Some(FileOperation::CreateDir);
        self.file_op_prompt_buffer = String::new();
        self.set_status("New directory name:");
    }

    // Handle confirmation for delete operation (y/n)
    pub fn handle_file_op_confirmation(&mut self, response: char) {
        if let Some(FileOperation::Delete(path)) = self.file_op_pending.clone() {
            if response == 'y' || response == 'Y' {
                // Perform delete
                let result = if path.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };

                match result {
                    Ok(()) => {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        let item_type = if path.is_dir() { "directory" } else { "file" };
                        self.set_status(&format!("Deleted {} '{}'", item_type, name));
                        // Reload explorer
                        self.load_explorer_entries();
                    }
                    Err(e) => {
                        self.set_status(&format!("Error deleting: {}", e));
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

        match self.file_op_pending.clone() {
            Some(FileOperation::CreateDir) => {
                // Create new directory (no .json requirement)
                if filename.is_empty() {
                    self.set_status("Error: Directory name cannot be empty");
                    self.file_op_pending = None;
                    self.file_op_prompt_buffer.clear();
                    return;
                }

                let new_path = self.get_target_directory().join(&filename);

                if new_path.exists() {
                    self.set_status(&format!("Error: Directory '{}' already exists", filename));
                } else {
                    match fs::create_dir(&new_path) {
                        Ok(()) => {
                            self.set_status(&format!("Created directory '{}'", filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error creating directory: {}", e));
                        }
                    }
                }
            }
            Some(FileOperation::Create) => {
                // Validate .json extension for files
                if !filename.ends_with(".json") {
                    self.set_status("Error: Filename must end with .json");
                    self.file_op_pending = None;
                    self.file_op_prompt_buffer.clear();
                    return;
                }
                // Create new file in target directory
                let new_path = self.get_target_directory().join(&filename);

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
                let is_dir = source_path.is_dir();

                // Validate .json extension for files only
                if !is_dir && !filename.ends_with(".json") {
                    self.set_status("Error: Filename must end with .json");
                    self.file_op_pending = None;
                    self.file_op_prompt_buffer.clear();
                    return;
                }

                // Copy file/directory to new location in target directory
                let dest_path = self.get_target_directory().join(&filename);

                if dest_path.exists() {
                    let item_type = if is_dir { "directory" } else { "file" };
                    self.set_status(&format!("Error: {} '{}' already exists", item_type, filename));
                } else {
                    let result = if is_dir {
                        copy_dir_recursive(&source_path, &dest_path)
                    } else {
                        fs::copy(&source_path, &dest_path).map(|_| ())
                    };

                    match result {
                        Ok(_) => {
                            let source_name = source_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            let item_type = if is_dir { "directory" } else { "file" };
                            self.set_status(&format!("Copied {} '{}' to '{}'", item_type, source_name, filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error copying: {}", e));
                        }
                    }
                }
            }
            Some(FileOperation::Rename(old_path)) => {
                let is_dir = old_path.is_dir();

                // Validate .json extension for files only
                if !is_dir && !filename.ends_with(".json") {
                    self.set_status("Error: Filename must end with .json");
                    self.file_op_pending = None;
                    self.file_op_prompt_buffer.clear();
                    return;
                }

                // Rename file/directory in its parent directory
                let new_path = old_path.parent()
                    .map(|p| p.join(&filename))
                    .unwrap_or_else(|| self.explorer_current_dir.join(&filename));

                if new_path.exists() && new_path != old_path {
                    let item_type = if is_dir { "directory" } else { "file" };
                    self.set_status(&format!("Error: {} '{}' already exists", item_type, filename));
                } else {
                    match fs::rename(&old_path, &new_path) {
                        Ok(()) => {
                            let old_name = old_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            let item_type = if is_dir { "directory" } else { "file" };
                            self.set_status(&format!("Renamed {} '{}' to '{}'", item_type, old_name, filename));
                            // Reload explorer
                            self.load_explorer_entries();
                        }
                        Err(e) => {
                            self.set_status(&format!("Error renaming: {}", e));
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
