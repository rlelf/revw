use chrono::Local;
use serde_json::Value;

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
        let mut in_array = false;
        let mut _array_index = 0;

        for i in 0..=cursor_line {
            if i >= lines.len() {
                break;
            }
            let line = &lines[i];

            if line.contains('[') {
                in_array = true;
                _array_index = 0;
            }

            if in_array && line.trim().starts_with('{') && i < cursor_line {
                _array_index += 1;
            }

            if line.contains(']') {
                in_array = false;
            }
        }

        if let Some(obj) = json_value.as_object_mut() {
            for (key, value) in obj.iter_mut() {
                if let Some(arr) = value.as_array_mut() {
                    let key_pattern = format!("\"{}\"", key);
                    let mut found_key = false;
                    let mut current_item = 0;

                    for i in 0..cursor_line {
                        if i >= lines.len() {
                            break;
                        }
                        if lines[i].contains(&key_pattern) {
                            found_key = true;
                        }
                        if found_key && lines[i].trim().starts_with('{') {
                            if i < cursor_line {
                                let mut depth = 1;
                                for j in (i + 1)..=cursor_line {
                                    if j >= lines.len() {
                                        break;
                                    }
                                    if lines[j].contains('{') {
                                        depth += 1;
                                    }
                                    if lines[j].contains('}') {
                                        depth -= 1;
                                        if depth == 0 {
                                            if j >= cursor_line {
                                                if current_item < arr.len() {
                                                    arr.remove(current_item);
                                                    deleted = true;
                                                }
                                                break;
                                            } else {
                                                current_item += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if deleted {
                        break;
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
                    "percentage": 0
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

    pub fn order_entries(json_input: &str) -> Result<(String, String), String> {
        let mut json_value: Value =
            serde_json::from_str(json_input).map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut messages = Vec::new();

        if let Some(obj) = json_value.as_object_mut() {
            // Order outside entries by percentage (highest first)
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
                    b_percent.cmp(&a_percent) // Descending order
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
}
