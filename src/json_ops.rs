use chrono::Local;
use serde_json::Value;
use regex::RegexBuilder;
use crate::content_ops::ContentOperations;

pub struct JsonOperations;

impl JsonOperations {
    pub fn delete_entry_at_cursor(
        json_input: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        if lines.is_empty() || cursor_line >= lines.len() {
            return Err("Invalid cursor position".to_string());
        }

        let mut deleted = false;

        // Single pass: find which array key and item index contains the cursor
        if let Some(obj) = json_value.as_object_mut() {
            let mut current_line = 0;
            let mut current_key: Option<String> = None;
            let mut item_index = 0;
            let mut in_object = false;
            let mut depth = 0;

            // Single scan to find cursor position
            for i in 0..=cursor_line {
                if i >= lines.len() {
                    break;
                }
                let line = &lines[i];
                let trimmed = line.trim();

                // Check if this line starts a new key section
                for key in obj.keys() {
                    if line.contains(&format!("\"{}\"", key)) && line.contains('[') {
                        current_key = Some(key.clone());
                        item_index = 0;
                        in_object = false;
                        depth = 0;
                        current_line = i;
                        break;
                    }
                }

                // Track object boundaries
                if trimmed.starts_with('{') {
                    if !in_object && i > current_line {
                        in_object = true;
                        depth = 0;
                    }
                    depth += 1;
                }
                if trimmed.contains('}') {
                    depth -= 1;
                    if depth == 0 && in_object {
                        if i < cursor_line {
                            item_index += 1;
                        } else if i >= cursor_line {
                            // Cursor is in this object
                            if let Some(ref key) = current_key {
                                if let Some(arr) = obj.get_mut(key).and_then(|v| v.as_array_mut()) {
                                    if item_index < arr.len() {
                                        arr.remove(item_index);
                                        deleted = true;
                                        break;
                                    }
                                }
                            }
                        }
                        in_object = false;
                    }
                }
            }
        }

        if deleted {
            let formatted = serde_json::to_string_pretty(&json_value)
                .map_err(|e| format!("Failed to format JSON: {}", e))?;
            Ok((formatted, "Entry deleted".to_string()))
        } else {
            Err("Could not delete entry at cursor position".to_string())
        }
    }

    pub fn add_inside_entry(json_input: &str) -> Result<(String, usize, usize, String), String> {
        let mut json_value: Value = if json_input.is_empty() {
            serde_json::json!({ "outside": [], "inside": [] })
        } else {
            serde_json::from_str(json_input)
                .unwrap_or_else(|_| serde_json::json!({ "outside": [], "inside": [] }))
        };

        let now = Local::now();
        let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(obj) = json_value.as_object_mut() {
            if !obj.contains_key("inside") {
                obj.insert("inside".to_string(), Value::Array(vec![]));
            }

            if let Some(inside_array) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                let new_entry = serde_json::json!({
                    "date": date_str,
                    "context": ""
                });

                // Insert at the beginning (index 0) for newest first
                inside_array.insert(0, new_entry);

                let formatted = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| format!("Failed to format JSON: {}", e))?;

                let lines: Vec<String> = formatted.lines().map(|s| s.to_string()).collect();

                // Find the first context field (which should be the one we just added)
                for (i, line) in lines.iter().enumerate() {
                    if line.trim().contains("\"context\": \"\"") {
                        let col = line.find("\"context\": \"").unwrap_or(0) + 12;
                        return Ok((formatted, i, col, "Added inside".to_string()));
                    }
                }

                Ok((formatted, 0, 0, "Added inside".to_string()))
            } else {
                Err("'inside' is not an array".to_string())
            }
        } else {
            Err("Invalid JSON structure".to_string())
        }
    }

    pub fn add_outside_entry(json_input: &str) -> Result<(String, usize, usize, String), String> {
        let mut json_value: Value = if json_input.is_empty() {
            serde_json::json!({ "outside": [], "inside": [] })
        } else {
            serde_json::from_str(json_input)
                .unwrap_or_else(|_| serde_json::json!({ "outside": [], "inside": [] }))
        };

        if let Some(obj) = json_value.as_object_mut() {
            if !obj.contains_key("outside") {
                obj.insert("outside".to_string(), Value::Array(vec![]));
            }

            if let Some(outside_array) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                let new_entry = serde_json::json!({
                    "name": "",
                    "context": "",
                    "url": "",
                    "percentage": null
                });

                outside_array.push(new_entry);

                let formatted = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| format!("Failed to format JSON: {}", e))?;

                let lines: Vec<String> = formatted.lines().map(|s| s.to_string()).collect();

                // Find the last name field
                for (i, line) in lines.iter().rev().enumerate() {
                    let actual_i = lines.len() - 1 - i;
                    if line.trim().contains("\"name\": \"\"") {
                        let col = line.find("\"name\": \"").unwrap_or(0) + 9;
                        return Ok((formatted, actual_i, col, "Added outside".to_string()));
                    }
                }

                Ok((formatted, 0, 0, "Added outside".to_string()))
            } else {
                Err("'outside' is not an array".to_string())
            }
        } else {
            Err("Invalid JSON structure".to_string())
        }
    }

    pub fn duplicate_entry_at_cursor(
        json_input: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        if lines.is_empty() || cursor_line >= lines.len() {
            return Err("Invalid cursor position".to_string());
        }

        let mut duplicated = false;

        // Single pass: find which array key and item index contains the cursor
        if let Some(obj) = json_value.as_object_mut() {
            let mut current_line = 0;
            let mut current_key: Option<String> = None;
            let mut item_index = 0;
            let mut in_object = false;
            let mut depth = 0;

            // Single scan to find cursor position
            for i in 0..=cursor_line {
                if i >= lines.len() {
                    break;
                }
                let line = &lines[i];
                let trimmed = line.trim();

                // Check if this line starts a new key section
                for key in obj.keys() {
                    if line.contains(&format!("\"{}\"", key)) && line.contains('[') {
                        current_key = Some(key.clone());
                        item_index = 0;
                        in_object = false;
                        depth = 0;
                        current_line = i;
                        break;
                    }
                }

                // Track object boundaries
                if trimmed.starts_with('{') {
                    if !in_object && i > current_line {
                        in_object = true;
                        depth = 0;
                    }
                    depth += 1;
                }
                if trimmed.contains('}') {
                    depth -= 1;
                    if depth == 0 && in_object {
                        if i < cursor_line {
                            item_index += 1;
                        } else if i >= cursor_line {
                            // Cursor is in this object
                            if let Some(ref key) = current_key {
                                if let Some(arr) = obj.get_mut(key).and_then(|v| v.as_array_mut()) {
                                    if item_index < arr.len() {
                                        let entry_clone = arr[item_index].clone();
                                        arr.insert(item_index + 1, entry_clone);
                                        duplicated = true;
                                        break;
                                    }
                                }
                            }
                        }
                        in_object = false;
                    }
                }
            }
        }

        if duplicated {
            let formatted = serde_json::to_string_pretty(&json_value)
                .map_err(|e| format!("Failed to format JSON: {}", e))?;
            Ok((formatted, "Entry duplicated".to_string()))
        } else {
            Err("Could not duplicate entry at cursor position".to_string())
        }
    }

    pub fn order_entries(json_input: &str) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut messages = Vec::new();

        if let Some(obj) = json_value.as_object_mut() {
            // Order outside entries by percentage (highest first), then by name (ascending)
            if let Some(outside_array) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside_array.sort_by(|a, b| {
                    let a_percent = a
                        .as_object()
                        .and_then(|o| o.get("percentage"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let b_percent = b
                        .as_object()
                        .and_then(|o| o.get("percentage"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let a_name = a
                        .as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_name = b
                        .as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    // First by percentage (descending), then by name (ascending)
                    b_percent.cmp(&a_percent).then_with(|| a_name.cmp(&b_name))
                });
                messages.push("Ordered outside entries");
            }

            // Order inside entries by date (newest first)
            if let Some(inside_array) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside_array.sort_by(|a, b| {
                    let a_date = a
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_date = b
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    b_date.cmp(&a_date) // Descending order (newest first)
                });
                messages.push("Ordered inside entries");
            }
        }

        let formatted = serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to format JSON: {}", e))?;

        let message = if messages.is_empty() {
            "No entries"
        } else {
            "Ordered"
        };

        Ok((formatted, message.to_string()))
    }

    pub fn order_by_percentage(json_input: &str) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut messages = Vec::new();

        if let Some(obj) = json_value.as_object_mut() {
            // Order outside entries by percentage only (highest first)
            if let Some(outside_array) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside_array.sort_by(|a, b| {
                    let a_percent = a
                        .as_object()
                        .and_then(|o| o.get("percentage"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let b_percent = b
                        .as_object()
                        .and_then(|o| o.get("percentage"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    // Order by percentage descending (highest first)
                    b_percent.cmp(&a_percent)
                });
                messages.push("Ordered outside entries by percentage");
            }

            // Order inside entries by date (newest first)
            if let Some(inside_array) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside_array.sort_by(|a, b| {
                    let a_date = a
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_date = b
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    b_date.cmp(&a_date) // Descending order (newest first)
                });
                messages.push("Ordered inside entries by date");
            }
        }

        let formatted = serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to format JSON: {}", e))?;

        let message = if messages.is_empty() {
            "No entries"
        } else {
            "Ordered by percentage"
        };

        Ok((formatted, message.to_string()))
    }

    pub fn order_by_name(json_input: &str) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut messages = Vec::new();

        if let Some(obj) = json_value.as_object_mut() {
            // Order outside entries by name only (ascending)
            if let Some(outside_array) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside_array.sort_by(|a, b| {
                    let a_name = a
                        .as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_name = b
                        .as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    // Order by name ascending
                    a_name.cmp(&b_name)
                });
                messages.push("Ordered outside entries by name");
            }

            // Order inside entries by date (newest first)
            if let Some(inside_array) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside_array.sort_by(|a, b| {
                    let a_date = a
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_date = b
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    b_date.cmp(&a_date) // Descending order (newest first)
                });
                messages.push("Ordered inside entries by date");
            }
        }

        let formatted = serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to format JSON: {}", e))?;

        let message = if messages.is_empty() {
            "No entries"
        } else {
            "Ordered by name"
        };

        Ok((formatted, message.to_string()))
    }

    pub fn order_random(json_input: &str) -> Result<(String, String), String> {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut messages = Vec::new();

        if let Some(obj) = json_value.as_object_mut() {
            // Shuffle outside entries randomly
            if let Some(outside_array) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside_array.shuffle(&mut rng);
                messages.push("Shuffled outside entries");
            }

            // Order inside entries by date (newest first) - same as other order commands
            if let Some(inside_array) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside_array.sort_by(|a, b| {
                    let a_date = a
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let b_date = b
                        .as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    b_date.cmp(&a_date) // Descending order (newest first)
                });
                messages.push("Ordered inside entries by date");
            }
        }

        let formatted = serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to format JSON: {}", e))?;

        let message = if messages.is_empty() {
            "No entries"
        } else {
            "Randomized outside entries"
        };

        Ok((formatted, message.to_string()))
    }
    /// Expand entries: produce one entry per match in `context`, each with that snippet.
    pub fn trim_context_around_match(json_value: &Value, pattern: &str, chars: usize) -> Value {
        if pattern.is_empty() {
            return json_value.clone();
        }

        let re = RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .or_else(|_| {
                RegexBuilder::new(&regex::escape(pattern))
                    .case_insensitive(true)
                    .build()
            });

        let snippets_for = |context: &str| -> Vec<String> {
            let total_chars = context.chars().count();
            let chars_vec: Vec<char> = context.chars().collect();
            let mut result = Vec::new();
            if let Ok(ref re) = re {
                for m in re.find_iter(context) {
                    let match_char = context[..m.start()].chars().count();
                    let match_end_char = match_char + m.as_str().chars().count();
                    let start = match_char.saturating_sub(chars);
                    let end = (match_end_char + chars).min(total_chars);
                    result.push(chars_vec[start..end].iter().collect());
                }
            }
            result
        };

        let expand_section = |arr: &[Value]| -> Vec<Value> {
            let mut out = Vec::new();
            for item in arr {
                if let Some(item_obj) = item.as_object() {
                    let ctx = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                    let snips = snippets_for(ctx);
                    if snips.is_empty() {
                        out.push(item.clone());
                    } else {
                        for snip in snips {
                            let mut entry = item_obj.clone();
                            entry.insert("context".to_string(), Value::String(snip));
                            out.push(Value::Object(entry));
                        }
                    }
                } else {
                    out.push(item.clone());
                }
            }
            out
        };

        let mut result = json_value.clone();
        if let Some(obj) = result.as_object_mut() {
            for section in ["outside", "inside"] {
                if let Some(arr) = obj.get(section).and_then(|v| v.as_array()) {
                    let expanded = expand_section(arr);
                    obj.insert(section.to_string(), Value::Array(expanded));
                }
            }
        }
        result
    }

    pub fn filter_entries(json_value: &Value, pattern: &str) -> Value {
        if pattern.is_empty() {
            return json_value.clone();
        }

        let re = RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap_or_else(|_| {
                RegexBuilder::new(&regex::escape(pattern))
                    .case_insensitive(true)
                    .build()
                    .expect("escaped pattern must compile")
            });

        let matches_re = |s: &str| re.is_match(s);

        let mut result = json_value.clone();

        if let Some(obj) = result.as_object_mut() {
            if let Some(outside) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.retain(|item| {
                    if let Some(item_obj) = item.as_object() {
                        let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                        let url = item_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        let percentage = item_obj.get("percentage")
                            .and_then(|v| v.as_i64())
                            .map(|p| format!("{}%", p))
                            .unwrap_or_default();

                        matches_re(name) || matches_re(context) || matches_re(url) || matches_re(&percentage)
                    } else {
                        false
                    }
                });
            }

            if let Some(inside) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.retain(|item| {
                    if let Some(item_obj) = item.as_object() {
                        let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                        matches_re(date) || matches_re(context)
                    } else {
                        false
                    }
                });
            }
        }

        result
    }

    fn build_re(pattern: &str) -> regex::Regex {
        RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap_or_else(|_| {
                RegexBuilder::new(&regex::escape(pattern))
                    .case_insensitive(true)
                    .build()
                    .expect("escaped pattern must compile")
            })
    }

    /// Delete outside entries where the `name` field matches pattern.
    pub fn delete_outside_by_name(json_value: &Value, pattern: &str) -> Value {
        let re = Self::build_re(pattern);
        let mut result = json_value.clone();
        if let Some(obj) = result.as_object_mut() {
            if let Some(outside) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.retain(|item| {
                    let name = item.as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    !re.is_match(name)
                });
            }
        }
        result
    }

    /// Delete outside entries where the `context` field matches pattern.
    pub fn delete_outside_by_context(json_value: &Value, pattern: &str) -> Value {
        let re = Self::build_re(pattern);
        let mut result = json_value.clone();
        if let Some(obj) = result.as_object_mut() {
            if let Some(outside) = obj.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.retain(|item| {
                    let context = item.as_object()
                        .and_then(|o| o.get("context"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    !re.is_match(context)
                });
            }
        }
        result
    }

    /// Delete inside entries where the `date` field matches pattern.
    pub fn delete_inside_by_date(json_value: &Value, pattern: &str) -> Value {
        let re = Self::build_re(pattern);
        let mut result = json_value.clone();
        if let Some(obj) = result.as_object_mut() {
            if let Some(inside) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.retain(|item| {
                    let date = item.as_object()
                        .and_then(|o| o.get("date"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    !re.is_match(date)
                });
            }
        }
        result
    }

    /// Delete inside entries where the `context` field matches pattern.
    pub fn delete_inside_by_context(json_value: &Value, pattern: &str) -> Value {
        let re = Self::build_re(pattern);
        let mut result = json_value.clone();
        if let Some(obj) = result.as_object_mut() {
            if let Some(inside) = obj.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.retain(|item| {
                    let context = item.as_object()
                        .and_then(|o| o.get("context"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    !re.is_match(context)
                });
            }
        }
        result
    }

    /// Append entries from new_json into current_json.
    /// inside_only/outside_only control which sections are merged.
    /// Inside entries are prepended (newest first); outside entries are appended.
    pub fn append_entries(current_json: &Value, new_json: &Value, inside_only: bool, outside_only: bool) -> Value {
        let mut result = current_json.clone();
        let both = !inside_only && !outside_only;

        if let Some(obj) = result.as_object_mut() {
            if inside_only || both {
                if let Some(new_inside) = new_json.get("inside").and_then(|v| v.as_array()) {
                    let inside_arr = obj.entry("inside".to_string()).or_insert(Value::Array(vec![]));
                    if let Some(arr) = inside_arr.as_array_mut() {
                        for (i, item) in new_inside.iter().enumerate() {
                            arr.insert(i, item.clone());
                        }
                    }
                }
            }

            if outside_only || both {
                if let Some(new_outside) = new_json.get("outside").and_then(|v| v.as_array()) {
                    let outside_arr = obj.entry("outside".to_string()).or_insert(Value::Array(vec![]));
                    if let Some(arr) = outside_arr.as_array_mut() {
                        for item in new_outside {
                            arr.push(item.clone());
                        }
                    }
                }
            }
        }

        result
    }
}

// Implement ContentOperations trait for JsonOperations
impl ContentOperations for JsonOperations {
    fn add_inside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        JsonOperations::add_inside_entry(content)
    }

    fn add_outside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        JsonOperations::add_outside_entry(content)
    }

    fn delete_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String> {
        JsonOperations::delete_entry_at_cursor(content, cursor_line, lines)
    }

    fn duplicate_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String> {
        JsonOperations::duplicate_entry_at_cursor(content, cursor_line, lines)
    }

    fn order_entries(&self, content: &str) -> Result<(String, String), String> {
        JsonOperations::order_entries(content)
    }

    fn order_by_percentage(&self, content: &str) -> Result<(String, String), String> {
        JsonOperations::order_by_percentage(content)
    }

    fn order_by_name(&self, content: &str) -> Result<(String, String), String> {
        JsonOperations::order_by_name(content)
    }

    fn order_random(&self, content: &str) -> Result<(String, String), String> {
        JsonOperations::order_random(content)
    }
}
