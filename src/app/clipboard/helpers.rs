use super::super::App;
use serde_json::Value;

impl App {
    /// Convert JSON value to Markdown string format
    pub(crate) fn json_to_markdown_string(json_value: &Value) -> Result<String, String> {
        let mut output_lines = Vec::new();

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

                            if !context.is_empty() {
                                output_lines.push(context.to_string());
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

                            if !context.is_empty() {
                                output_lines.push(context.to_string());
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

        Ok(output_lines.join("\n"))
    }

    /// Parse clipboard text and convert to JSON value
    /// Supports JSON and Markdown formats
    pub(super) fn clipboard_text_to_json_value(&self, clipboard_text: &str) -> Result<Value, String> {
        if let Ok(json_value) = serde_json::from_str::<Value>(clipboard_text) {
            return Ok(json_value);
        }

        if clipboard_text.contains("## OUTSIDE") || clipboard_text.contains("## INSIDE") {
            let json_str = self
                .parse_markdown(clipboard_text)
                .map_err(|e| format!("Clipboard is not valid Markdown: {}", e))?;
            return serde_json::from_str::<Value>(&json_str)
                .map_err(|e| format!("Clipboard is not valid JSON: {}", e));
        }

        Err("Clipboard is not valid JSON or Markdown".to_string())
    }
}

