use unicode_width::UnicodeWidthChar;

pub struct Renderer;

impl Renderer {
    pub fn display_width_str(s: &str) -> usize {
        s.chars().map(|c| UnicodeWidthChar::width(c).unwrap_or(0)).sum()
    }

    pub fn prefix_display_width(s: &str, char_pos: usize) -> usize {
        s.chars().take(char_pos).map(|c| UnicodeWidthChar::width(c).unwrap_or(0)).sum()
    }

    pub fn char_index_for_col(s: &str, target_cols: usize) -> usize {
        let mut sum = 0usize;
        for (i, c) in s.chars().enumerate() {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if sum >= target_cols { return i; }
            sum += w.max(0);
        }
        s.chars().count()
    }

    pub fn slice_columns(s: &str, start_cols: usize, width_cols: usize) -> String {
        if width_cols == 0 { return String::new(); }
        let mut sum = 0usize;
        let mut start_idx = 0usize;
        for (i, c) in s.chars().enumerate() {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if sum >= start_cols { start_idx = i; break; }
            sum += w;
            start_idx = i + 1;
        }
        let mut out = String::new();
        let mut used = 0usize;
        for c in s.chars().skip(start_idx) {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if used + w > width_cols { break; }
            out.push(c);
            used += w;
        }
        out
    }

    pub fn render_relf(json_input: &str) -> Vec<String> {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_input) {
            let mut result = Vec::new();

            if let Some(obj) = json_value.as_object() {
                let mut is_first_section = true;

                for (section_key, section_value) in obj {
                    if section_key == "outside" || section_key == "inside" {
                        if let Some(section_array) = section_value.as_array() {
                            if !is_first_section {
                                result.push("".to_string());
                            }
                            is_first_section = false;

                            result.push(section_key.to_uppercase());
                            result.push("".to_string());

                            for item in section_array {
                                if let Some(item_obj) = item.as_object() {
                                    if section_key == "outside" {
                                        let name = item_obj.get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("").replace("\n", "\\n");
                                        let context = item_obj.get("context")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("").replace("\n", "\\n");
                                        let url = item_obj.get("url")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("").replace("\n", "\\n");
                                        let percentage = item_obj.get("percentage")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);
                                        
                                        result.push(format!("  {}", name));
                                        if !context.is_empty() {
                                            result.push(format!("    {}", context));
                                        }
                                        if !url.is_empty() {
                                            result.push(format!("    {}", url));
                                        }
                                        result.push(format!("    {}%", percentage));
                                        result.push("".to_string());
                                    } else if section_key == "inside" {
                                        let mut first_field = true;
                                        for (_key, value) in item_obj {
                                            let value_str = match value {
                                                serde_json::Value::String(s) => s.replace("\n", "\\n"),
                                                serde_json::Value::Number(n) => n.to_string(),
                                                serde_json::Value::Bool(b) => b.to_string(),
                                                _ => value.to_string().replace("\n", "\\n"),
                                            };
                                            
                                            if !value_str.is_empty() {
                                                if first_field {
                                                    result.push(format!("  {}", value_str));
                                                    first_field = false;
                                                } else {
                                                    result.push(format!("    {}", value_str));
                                                }
                                            }
                                        }
                                        result.push("".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return result;
        }

        let lines: Vec<String> = json_input
            .lines()
            .enumerate()
            .take(50)
            .map(|(i, line)| format!("{:4}: {}", i + 1, line))
            .collect();
        
        vec![
            "⚠ Not valid JSON - showing raw text file content".to_string(),
            "".to_string(),
        ]
        .into_iter()
        .chain(lines)
        .collect()
    }

    pub fn render_json(json_input: &str) -> Vec<String> {
        json_input
            .lines()
            .map(|line| line.to_string())
            .collect()
    }
}