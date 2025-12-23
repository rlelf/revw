use chrono::Local;
use crate::content_ops::ContentOperations;
use serde_json::{json, Value};

pub struct ToonOperations;

impl ToonOperations {
    /// Parse Toon content and convert to JSON, then perform operation, then convert back
    fn modify_via_json<F>(content: &str, f: F) -> Result<(String, String), String>
    where
        F: FnOnce(Value) -> Result<Value, String>,
    {
        // Parse Toon to JSON
        let json_str = Self::toon_to_json(content)?;
        let mut json: Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        // Apply modification
        json = f(json)?;

        // Convert back to Toon
        let new_json_str = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("JSON serialize error: {}", e))?;
        let new_toon = Self::json_to_toon(&new_json_str)?;

        Ok((new_toon, new_json_str))
    }

    /// Parse Toon format and convert to JSON string
    fn toon_to_json(toon_content: &str) -> Result<String, String> {
        let mut outside_entries = Vec::new();
        let mut inside_entries = Vec::new();

        let lines: Vec<&str> = toon_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() {
                i += 1;
                continue;
            }

            if line.contains('[') && line.contains('{') && line.ends_with(':') {
                let (section, fields) = Self::parse_toon_header(line)?;
                i += 1;

                while i < lines.len() {
                    let data_line = lines[i].trim();

                    if data_line.is_empty() {
                        i += 1;
                        break;
                    }
                    if data_line.contains('[') && data_line.contains('{') && data_line.ends_with(':') {
                        break;
                    }

                    let entry = Self::parse_toon_data_line(data_line, &fields)?;

                    if section == "outside" {
                        outside_entries.push(entry);
                    } else if section == "inside" {
                        inside_entries.push(entry);
                    }

                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        let json_obj = json!({
            "outside": outside_entries,
            "inside": inside_entries
        });

        serde_json::to_string_pretty(&json_obj)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }

    fn parse_toon_header(line: &str) -> Result<(String, Vec<String>), String> {
        let section_end = line.find('[')
            .ok_or_else(|| "Invalid header: missing '['".to_string())?;
        let section = line[..section_end].trim().to_string();

        let fields_start = line.find('{')
            .ok_or_else(|| "Invalid header: missing '{'".to_string())?;
        let fields_end = line.find('}')
            .ok_or_else(|| "Invalid header: missing '}'".to_string())?;

        let fields_str = &line[fields_start + 1..fields_end];
        let fields: Vec<String> = fields_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok((section, fields))
    }

    fn parse_toon_data_line(line: &str, fields: &[String]) -> Result<Value, String> {
        let values = Self::parse_csv_line(line)?;

        if values.len() != fields.len() {
            return Err(format!(
                "Field count mismatch: expected {}, got {}",
                fields.len(),
                values.len()
            ));
        }

        let mut entry = serde_json::Map::new();
        for (field, value) in fields.iter().zip(values.iter()) {
            if field == "percentage" {
                // Empty string means null for percentage
                if value.is_empty() {
                    entry.insert(field.clone(), Value::Null);
                    continue;
                }
                if let Ok(num) = value.parse::<i64>() {
                    entry.insert(field.clone(), json!(num));
                    continue;
                }
            }

            entry.insert(field.clone(), json!(value));
        }

        Ok(Value::Object(entry))
    }

    fn parse_csv_line(line: &str) -> Result<Vec<String>, String> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ',' if !in_quotes => {
                    values.push(current.trim().to_string());
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() || !values.is_empty() {
            values.push(current.trim().to_string());
        }

        Ok(values)
    }

    /// Convert JSON string to Toon format
    pub fn json_to_toon(json_str: &str) -> Result<String, String> {
        let json_value: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut toon_output = String::new();

        // Convert outside section
        if let Some(outside) = json_value.get("outside").and_then(|v| v.as_array()) {
            if !outside.is_empty() {
                let fields = vec!["name", "context", "url", "percentage"];
                toon_output.push_str(&format!("outside[{}]{{{}}}:\n",
                    outside.len(),
                    fields.join(",")));

                for entry in outside {
                    let line = Self::entry_to_toon_line(entry, &fields)?;
                    toon_output.push_str("  ");
                    toon_output.push_str(&line);
                    toon_output.push('\n');
                }
                toon_output.push('\n');
            }
        }

        // Convert inside section
        if let Some(inside) = json_value.get("inside").and_then(|v| v.as_array()) {
            if !inside.is_empty() {
                let fields = vec!["date", "context"];
                toon_output.push_str(&format!("inside[{}]{{{}}}:\n",
                    inside.len(),
                    fields.join(",")));

                for entry in inside {
                    let line = Self::entry_to_toon_line(entry, &fields)?;
                    toon_output.push_str("  ");
                    toon_output.push_str(&line);
                    toon_output.push('\n');
                }
            }
        }

        Ok(toon_output)
    }

    fn entry_to_toon_line(entry: &Value, fields: &[&str]) -> Result<String, String> {
        let mut values = Vec::new();

        for field in fields {
            let value = match entry.get(field) {
                Some(value) => value,
                None if *field == "percentage" => {
                    values.push("null".to_string());
                    continue;
                }
                None => return Err(format!("Missing field: {}", field)),
            };

            let value_str = match value {
                Value::Null => "null".to_string(),
                Value::String(s) => {
                    if s.contains(',') || s.contains('"') || s.contains('\n') {
                        format!("\"{}\"", s.replace('"', "\\\""))
                    } else {
                        s.clone()
                    }
                }
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };

            values.push(value_str);
        }

        Ok(values.join(","))
    }
}

impl ContentOperations for ToonOperations {
    fn add_inside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        let result = Self::modify_via_json(content, |mut json| {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let new_entry = json!({
                "date": timestamp,
                "context": ""
            });

            if let Some(inside) = json.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.insert(0, new_entry);
            }

            Ok(json)
        })?;

        // For Toon format, cursor position is always at the top of the file
        Ok((result.0, 0, 0, "Added inside".to_string()))
    }

    fn add_outside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        let result = Self::modify_via_json(content, |mut json| {
            let new_entry = json!({
                "name": "",
                "context": "",
                "url": "",
                "percentage": null
            });

            if let Some(outside) = json.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.push(new_entry);
            }

            Ok(json)
        })?;

        let mut outside_line = 0;
        let mut in_outside = false;
        for (idx, line) in result.0.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("outside[") && trimmed.ends_with(':') {
                in_outside = true;
                continue;
            }
            if in_outside {
                if trimmed.is_empty() {
                    break;
                }
                outside_line = idx;
            }
        }

        Ok((result.0, outside_line, 0, "Added outside".to_string()))
    }

    fn delete_entry_at_cursor(
        &self,
        _content: &str,
        _cursor_line: usize,
        _lines: &[String],
    ) -> Result<(String, String), String> {
        // For Toon format in edit mode, we don't support entry-based deletion
        // This would require more complex parsing
        Err("Entry deletion in Toon format is not yet supported in Edit mode".to_string())
    }

    fn duplicate_entry_at_cursor(
        &self,
        _content: &str,
        _cursor_line: usize,
        _lines: &[String],
    ) -> Result<(String, String), String> {
        // For Toon format in edit mode, we don't support entry-based duplication
        Err("Entry duplication in Toon format is not yet supported in Edit mode".to_string())
    }

    fn order_entries(&self, content: &str) -> Result<(String, String), String> {
        let (toon_content, _) = Self::modify_via_json(content, |mut json| {
            // Order outside by percentage desc, then name asc
            if let Some(outside) = json.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.sort_by(|a, b| {
                    let a_pct = a.get("percentage").and_then(|v| v.as_i64()).unwrap_or(0);
                    let b_pct = b.get("percentage").and_then(|v| v.as_i64()).unwrap_or(0);
                    let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");

                    b_pct.cmp(&a_pct).then(a_name.cmp(b_name))
                });
            }

            // Order inside by date desc
            if let Some(inside) = json.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.sort_by(|a, b| {
                    let a_date = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let b_date = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    b_date.cmp(a_date)
                });
            }

            Ok(json)
        })?;

        Ok((toon_content, "Ordered".to_string()))
    }

    fn order_by_percentage(&self, content: &str) -> Result<(String, String), String> {
        let (toon_content, _) = Self::modify_via_json(content, |mut json| {
            if let Some(outside) = json.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.sort_by(|a, b| {
                    let a_pct = a.get("percentage").and_then(|v| v.as_i64()).unwrap_or(0);
                    let b_pct = b.get("percentage").and_then(|v| v.as_i64()).unwrap_or(0);
                    b_pct.cmp(&a_pct)
                });
            }

            if let Some(inside) = json.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.sort_by(|a, b| {
                    let a_date = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let b_date = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    b_date.cmp(a_date)
                });
            }

            Ok(json)
        })?;

        Ok((toon_content, "Ordered by percentage".to_string()))
    }

    fn order_by_name(&self, content: &str) -> Result<(String, String), String> {
        let (toon_content, _) = Self::modify_via_json(content, |mut json| {
            if let Some(outside) = json.get_mut("outside").and_then(|v| v.as_array_mut()) {
                outside.sort_by(|a, b| {
                    let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    a_name.cmp(b_name)
                });
            }

            if let Some(inside) = json.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.sort_by(|a, b| {
                    let a_date = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let b_date = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    b_date.cmp(a_date)
                });
            }

            Ok(json)
        })?;

        Ok((toon_content, "Ordered by name".to_string()))
    }

    fn order_random(&self, content: &str) -> Result<(String, String), String> {
        use rand::seq::SliceRandom;
        use rand::rng;

        let (toon_content, _) = Self::modify_via_json(content, |mut json| {
            if let Some(outside) = json.get_mut("outside").and_then(|v| v.as_array_mut()) {
                let mut rng = rng();
                outside.shuffle(&mut rng);
            }

            if let Some(inside) = json.get_mut("inside").and_then(|v| v.as_array_mut()) {
                inside.sort_by(|a, b| {
                    let a_date = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let b_date = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    b_date.cmp(a_date)
                });
            }

            Ok(json)
        })?;

        Ok((toon_content, "Randomized outside entries".to_string()))
    }
}
