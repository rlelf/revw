use super::super::super::App;
use serde_json::Value;

impl App {
    /// Paste INSIDE section from text (overwrite)
    pub fn paste_inside_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_overwrite(text, "INSIDE");
        } else {
            // For JSON files, parse clipboard for INSIDE section
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_inside) = clipboard_obj.get("inside") {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                current_obj.insert("inside".to_string(), new_inside.clone());
                                self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("INSIDE section overwritten");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste OUTSIDE section from text (overwrite)
    pub fn paste_outside_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_overwrite(text, "OUTSIDE");
        } else {
            // For JSON files, parse clipboard for OUTSIDE section
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_outside) = clipboard_obj.get("outside") {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                current_obj.insert("outside".to_string(), new_outside.clone());
                                self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("OUTSIDE section overwritten");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste INSIDE section from text (append)
    pub fn paste_inside_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_append(text, "INSIDE");
        } else {
            // For JSON files, append INSIDE entries
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                let current_inside = current_obj
                                    .entry("inside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(inside_array) = current_inside.as_array_mut() {
                                    inside_array.extend(new_inside.clone());
                                    self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                    self.is_modified = true;
                                    self.convert_json();
                                    self.set_status(&format!("Appended {} INSIDE entries", new_inside.len()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste OUTSIDE section from text (append)
    pub fn paste_outside_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            self.paste_markdown_section_append(text, "OUTSIDE");
        } else {
            // For JSON files, append OUTSIDE entries
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Some(new_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                        if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                            if let Some(current_obj) = current_json.as_object_mut() {
                                let current_outside = current_obj
                                    .entry("outside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(outside_array) = current_outside.as_array_mut() {
                                    outside_array.extend(new_outside.clone());
                                    self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                                    self.is_modified = true;
                                    self.convert_json();
                                    self.set_status(&format!("Appended {} OUTSIDE entries", new_outside.len()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Paste both INSIDE and OUTSIDE from text (append)
    pub fn paste_all_append_from_text(&mut self, text: &str) {
        if self.is_markdown_file() {
            // For Markdown files, append both sections
            self.paste_markdown_section_append(text, "OUTSIDE");
            self.paste_markdown_section_append(text, "INSIDE");
        } else {
            // For JSON files
            if let Ok(clipboard_json) = serde_json::from_str::<Value>(text) {
                if let Some(clipboard_obj) = clipboard_json.as_object() {
                    if let Ok(mut current_json) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(current_obj) = current_json.as_object_mut() {
                            let mut appended_count = 0;

                            // Append INSIDE
                            if let Some(new_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                                let current_inside = current_obj
                                    .entry("inside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(inside_array) = current_inside.as_array_mut() {
                                    inside_array.extend(new_inside.clone());
                                    appended_count += new_inside.len();
                                }
                            }

                            // Append OUTSIDE
                            if let Some(new_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                                let current_outside = current_obj
                                    .entry("outside".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));

                                if let Some(outside_array) = current_outside.as_array_mut() {
                                    outside_array.extend(new_outside.clone());
                                    appended_count += new_outside.len();
                                }
                            }

                            self.json_input = serde_json::to_string_pretty(&current_json).unwrap();
                            self.is_modified = true;
                            self.convert_json();
                            self.set_status(&format!("Appended {} entries total", appended_count));
                        }
                    }
                }
            }
        }
    }
}
