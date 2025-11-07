use super::App;
use std::{{fs, path::PathBuf, time::Instant}};
use serde_json::json;

impl App {
    pub fn load_file(&mut self, path: PathBuf) {
        // Path cleaning - remove all kinds of quotes and whitespace
        let path_display = path.display().to_string();
        let cleaned_path_str = path_display
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches('`')
            .trim();

        let fixed_path = PathBuf::from(cleaned_path_str);
        let final_path_display = fixed_path.display().to_string();

        match fs::read_to_string(&fixed_path) {
            Ok(content) => {
                self.json_input = content;

                let path_changed = self.file_path.as_ref() != Some(&fixed_path);
                self.file_path = Some(fixed_path.clone());
                if path_changed {
                    self.file_path_changed = true;
                }

                self.set_status(&format!("Loaded: {}", final_path_display));

                self.convert_json();

                // Reset card selection and cursor position when opening a new file
                if path_changed {
                    self.selected_entry_index = 0;
                    self.hscroll = 0;
                    self.content_cursor_line = 0;
                    self.content_cursor_col = 0;
                    self.scroll = 0;
                }
            }
            Err(e) => {
                // If file doesn't exist, create it with default entries
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Get current timestamp
                    let now = chrono::Local::now();
                    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

                    // Create formatted JSON with proper indentation
                    let default_value = json!({
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

                    let default_json = serde_json::to_string_pretty(&default_value)
                        .unwrap_or_else(|_| String::from(r#"{"outside":[],"inside":[]}"#));

                    match fs::write(&fixed_path, &default_json) {
                        Ok(()) => {
                            self.json_input = default_json;
                            let path_changed = self.file_path.as_ref() != Some(&fixed_path);
                            self.file_path = Some(fixed_path.clone());
                            if path_changed {
                                self.file_path_changed = true;
                            }
                            self.set_status(&format!("Created new file: {}", final_path_display));
                            self.convert_json();
                            // Reset card selection and cursor position when creating a new file
                            if path_changed {
                                self.selected_entry_index = 0;
                                self.hscroll = 0;
                                self.content_cursor_line = 0;
                                self.content_cursor_col = 0;
                                self.scroll = 0;
                            }
                            // Reload explorer if open
                            if self.explorer_open {
                                self.load_explorer_entries();
                            }
                        }
                        Err(create_err) => {
                            self.set_status(&format!("Error creating '{}': {}", final_path_display, create_err));
                        }
                    }
                } else {
                    self.set_status(&format!("Error loading '{}': {}", final_path_display, e));
                }
            }
        }
    }
    pub fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            match fs::write(path, &self.json_input) {
                Ok(()) => {
                    self.is_modified = false;
                    self.last_save_time = Some(Instant::now());
                    self.set_status(&format!("Saved: {}", path.display()));
                    // Reload explorer if open (without resetting cursor position)
                    if self.explorer_open {
                        self.reload_explorer_entries();
                    }
                }
                Err(e) => {
                    self.set_status(&format!("Error saving: {}", e));
                }
            }
        } else {
            self.set_status("No filename. Use :w filename.json");
        }
    }

    pub fn save_file_as(&mut self, filename: &str) {
        let path = PathBuf::from(filename);
        match fs::write(&path, &self.json_input) {
            Ok(()) => {
                let path_changed = self.file_path.as_ref() != Some(&path);
                self.file_path = Some(path.clone());
                self.is_modified = false;
                self.last_save_time = Some(Instant::now());
                if path_changed {
                    self.file_path_changed = true;
                }
                self.set_status(&format!("Saved: {}", path.display()));
                // Reload explorer if open
                if self.explorer_open {
                    self.load_explorer_entries();
                }
            }
            Err(e) => {
                self.set_status(&format!("Error saving: {}", e));
            }
        }
    }

    pub fn reload_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.json_input = content;
                    self.is_modified = false;
                    self.convert_json();
                    self.content_cursor_line = 0;
                    self.content_cursor_col = 0;
                    self.scroll = 0;
                    self.set_status(&format!("Reloaded: {}", path.display()));
                }
                Err(e) => {
                    self.set_status(&format!("Error reloading: {}", e));
                }
            }
        } else {
            self.set_status("No file to reload");
        }
    }

    pub fn export_to_markdown(&mut self) {
        // Check if a file is currently open
        if self.file_path.is_none() {
            self.set_status("Error: No file open");
            return;
        }

        let json_path = self.file_path.as_ref().unwrap();

        // Create markdown filename (same name, different extension)
        let md_path = json_path.with_extension("md");

        // Generate markdown content
        let mut output_lines = Vec::new();

        // Parse JSON to determine which section each entry belongs to
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                // OUTSIDE section
                if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                    if !outside.is_empty() {
                        output_lines.push("## OUTSIDE".to_string());
                        output_lines.push("".to_string());

                        for item in outside {
                            if let Some(item_obj) = item.as_object() {
                                let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("name");
                                let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                let url = item_obj.get("url").and_then(|v| v.as_str());
                                let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                output_lines.push(format!("### {}", name));

                                // Replace literal \n with actual newlines in context
                                if !context.is_empty() {
                                    let formatted_context = context.replace("\\n", "\n");
                                    output_lines.push(formatted_context);
                                }

                                // Only output URL if it's not null and not empty
                                if let Some(url_str) = url {
                                    if !url_str.is_empty() {
                                        output_lines.push(format!("#### URL: {}", url_str));
                                    }
                                }

                                // Only output percentage if it's not null
                                if let Some(pct) = percentage {
                                    output_lines.push(format!("#### Percentage: {}%", pct));
                                }

                                output_lines.push("".to_string());
                            }
                        }
                    }
                }

                // INSIDE section
                if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                    if !inside.is_empty() {
                        output_lines.push("## INSIDE".to_string());
                        output_lines.push("".to_string());

                        for item in inside {
                            if let Some(item_obj) = item.as_object() {
                                let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                                let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                                if !date.is_empty() {
                                    output_lines.push(format!("### {}", date));
                                }

                                // Replace literal \n with actual newlines in context
                                if !context.is_empty() {
                                    let formatted_context = context.replace("\\n", "\n");
                                    output_lines.push(formatted_context);
                                }

                                output_lines.push("".to_string());
                            }
                        }
                    }
                }
            }
        }

        let markdown_content = output_lines.join("\n");

        // Write to file
        match fs::write(&md_path, markdown_content) {
            Ok(()) => {
                self.set_status(&format!("Exported to: {}", md_path.display()));
                // Reload explorer if open
                if self.explorer_open {
                    self.reload_explorer_entries();
                }
            }
            Err(e) => {
                self.set_status(&format!("Error exporting markdown: {}", e));
            }
        }
    }

}
