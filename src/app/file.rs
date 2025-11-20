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
                // Check file extension to determine format
                let is_markdown = fixed_path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("md"))
                    .unwrap_or(false);

                if is_markdown {
                    self.markdown_input = content.clone();
                    // Parse Markdown and convert to JSON
                    match self.parse_markdown(&content) {
                        Ok(json_content) => {
                            self.json_input = json_content;
                        }
                        Err(e) => {
                            self.set_status(&format!("Error parsing markdown: {}", e));
                            return;
                        }
                    }
                } else {
                    self.markdown_input = String::new();
                    // Load as JSON directly
                    self.json_input = content;
                }

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
                    // Clear undo/redo history when switching files
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                }
            }
            Err(e) => {
                // If file doesn't exist, create it with default entries
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Check file extension to determine format
                    let is_markdown = fixed_path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false);

                    // Get current timestamp
                    let now = chrono::Local::now();
                    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

                    let default_content = if is_markdown {
                        // Create Markdown format
                        format!(
                            "## OUTSIDE\n### \n\n**URL:** \n\n**Percentage:** \n\n## INSIDE\n### {}\n",
                            timestamp
                        )
                    } else {
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
                        serde_json::to_string_pretty(&default_value)
                            .unwrap_or_else(|_| String::from(r#"{"outside":[],"inside":[]}"#))
                    };

                    match fs::write(&fixed_path, &default_content) {
                        Ok(()) => {
                            if is_markdown {
                                self.markdown_input = default_content.clone();
                                // Parse Markdown and convert to JSON
                                match self.parse_markdown(&default_content) {
                                    Ok(json_content) => {
                                        self.json_input = json_content;
                                    }
                                    Err(e) => {
                                        self.set_status(&format!("Error parsing markdown: {}", e));
                                        return;
                                    }
                                }
                            } else {
                                self.json_input = default_content;
                            }
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
                                // Clear undo/redo history when switching files
                                self.undo_stack.clear();
                                self.redo_stack.clear();
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
            // Check if this is a Markdown file
            let is_markdown = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false);

            let content_to_save = if is_markdown {
                // Convert to markdown if we don't have markdown content yet
                if self.markdown_input.is_empty() {
                    match self.convert_to_markdown() {
                        Ok(md_content) => {
                            self.markdown_input = md_content.clone();
                            md_content
                        }
                        Err(e) => {
                            self.set_status(&format!("Error converting to markdown: {}", e));
                            return;
                        }
                    }
                } else {
                    self.markdown_input.clone()
                }
            } else {
                // Save as JSON
                self.json_input.clone()
            };

            match fs::write(path, &content_to_save) {
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
            self.set_status("No filename. Use :w filename");
        }
    }

    pub fn save_file_as(&mut self, filename: &str) {
        let path = PathBuf::from(filename);

        // Check if this is a Markdown file
        let is_markdown = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);

        let content_to_save = if is_markdown {
            // If we already have Markdown content, use it directly
            // Otherwise, convert JSON to Markdown
            if self.is_markdown_file() && !self.markdown_input.is_empty() {
                self.markdown_input.clone()
            } else {
                match self.convert_to_markdown() {
                    Ok(md_content) => {
                        // Store the converted markdown
                        self.markdown_input = md_content.clone();
                        md_content
                    }
                    Err(e) => {
                        self.set_status(&format!("Error converting to markdown: {}", e));
                        return;
                    }
                }
            }
        } else {
            // Save as JSON
            self.json_input.clone()
        };

        match fs::write(&path, &content_to_save) {
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
                    // Check if this is a Markdown file
                    let is_markdown = path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false);

                    if is_markdown {
                        self.markdown_input = content.clone();
                        // Parse Markdown and convert to JSON
                        match self.parse_markdown(&content) {
                            Ok(json_content) => {
                                self.json_input = json_content;
                            }
                            Err(e) => {
                                self.set_status(&format!("Error parsing markdown: {}", e));
                                return;
                            }
                        }
                    } else {
                        self.markdown_input = String::new();
                        self.json_input = content;
                    }

                    self.is_modified = false;
                    self.convert_json();

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

    pub fn export_to_json(&mut self) {
        // Check if a file is currently open
        if self.file_path.is_none() {
            self.set_status("Error: No file open");
            return;
        }

        let current_path = self.file_path.as_ref().unwrap();

        // Create JSON filename (same name, different extension)
        let json_path = current_path.with_extension("json");

        // Format JSON with pretty print
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&self.json_input) {
            if let Ok(formatted_json) = serde_json::to_string_pretty(&json_value) {
                // Write to file
                match fs::write(&json_path, formatted_json) {
                    Ok(()) => {
                        self.set_status(&format!("Exported to: {}", json_path.display()));
                        // Reload explorer if open
                        if self.explorer_open {
                            self.reload_explorer_entries();
                        }
                    }
                    Err(e) => {
                        self.set_status(&format!("Error exporting JSON: {}", e));
                    }
                }
            } else {
                self.set_status("Error formatting JSON");
            }
        } else {
            self.set_status("Error: Invalid JSON data");
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
                                let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                let url = item_obj.get("url").and_then(|v| v.as_str());
                                let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                if !name.is_empty() {
                                    output_lines.push(format!("### {}", name));
                                }

                                // Replace literal \n with actual newlines in context
                                if !context.is_empty() {
                                    let formatted_context = context.replace("\\n", "\n");
                                    output_lines.push(formatted_context);
                                }

                                // Only output URL if it's not null and not empty
                                if let Some(url_str) = url {
                                    if !url_str.is_empty() {
                                        output_lines.push("".to_string());
                                        output_lines.push(format!("**URL:** {}", url_str));
                                    }
                                }

                                // Only output percentage if it's not null
                                if let Some(pct) = percentage {
                                    output_lines.push("".to_string());
                                    output_lines.push(format!("**Percentage:** {}%", pct));
                                }

                                // Only add blank line if we had any content
                                if !name.is_empty() || !context.is_empty() || url.is_some() || percentage.is_some() {
                                    output_lines.push("".to_string());
                                }
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

                                // Only add blank line if we had content
                                if !date.is_empty() || !context.is_empty() {
                                    output_lines.push("".to_string());
                                }
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
