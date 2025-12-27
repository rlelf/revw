use super::App;
use serde_json::{json, Value};

impl App {
    /// Parse Toon format and convert to JSON
    pub fn parse_toon(&self, toon_content: &str) -> Result<String, String> {
        let mut outside_entries = Vec::new();
        let mut inside_entries = Vec::new();

        let lines: Vec<&str> = toon_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check for header line: section[count]{fields}:
            if line.contains('[') && line.contains('{') && line.ends_with(':') {
                let (section, fields) = Self::parse_toon_header(line)?;
                i += 1;

                // Parse data lines until we hit another header or end of file
                while i < lines.len() {
                    let data_line = lines[i].trim();

                    // Stop if we hit another header or empty line before data
                    if data_line.is_empty() {
                        i += 1;
                        break;
                    }
                    if data_line.contains('[') && data_line.contains('{') && data_line.ends_with(':') {
                        break;
                    }

                    // Parse the data line
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

        // Build JSON
        let json_obj = json!({
            "outside": outside_entries,
            "inside": inside_entries
        });

        serde_json::to_string_pretty(&json_obj)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }

    /// Parse Toon header: section[count]{field1,field2,...}:
    fn parse_toon_header(line: &str) -> Result<(String, Vec<String>), String> {
        // Extract section name
        let section_end = line.find('[')
            .ok_or_else(|| "Invalid header: missing '['".to_string())?;
        let section = line[..section_end].trim().to_string();

        // Extract fields
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

    /// Parse a Toon data line: value1,value2,value3,...
    /// Handles quoted strings with commas inside
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
            // Try to parse as number for "percentage" field
            if field == "percentage" {
                if let Ok(num) = value.parse::<i64>() {
                    entry.insert(field.clone(), json!(num));
                    continue;
                }
            }

            // Otherwise, treat as string
            entry.insert(field.clone(), json!(value));
        }

        Ok(Value::Object(entry))
    }

    /// Parse CSV line with support for quoted strings
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

        // Add the last value
        if !current.is_empty() || !values.is_empty() {
            values.push(current.trim().to_string());
        }

        Ok(values)
    }

    /// Convert JSON to Toon format
    pub fn convert_to_toon(&self) -> Result<String, String> {
        let json_value: Value = serde_json::from_str(&self.json_input)
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

    /// Convert a JSON entry to a Toon data line
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
                    // Quote if contains comma or special characters
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

    /// Check if current file is a Toon file
    pub fn is_toon_file(&self) -> bool {
        self.file_mode == super::FileMode::Toon
    }

    /// Sync toon_input from json_input if this is a Toon file
    /// Returns true if sync occurred, false otherwise
    pub fn sync_toon_from_json(&mut self) -> bool {
        if self.is_toon_file() {
            if let Ok(toon_str) = self.convert_to_toon() {
                self.toon_input = toon_str;
                return true;
            }
        }
        false
    }
}
