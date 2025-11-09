use super::App;
use serde_json::json;

impl App {
    /// Parse Markdown content and convert to JSON format
    pub fn parse_markdown(&self, content: &str) -> Result<String, String> {
        let mut outside_entries = Vec::new();
        let mut inside_entries = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_section = None; // "OUTSIDE" or "INSIDE"

        while i < lines.len() {
            let line = lines[i].trim();

            // Check for section headers
            if line == "## OUTSIDE" {
                current_section = Some("OUTSIDE");
                i += 1;
                continue;
            } else if line == "## INSIDE" {
                current_section = Some("INSIDE");
                i += 1;
                continue;
            }

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check for entry headers (### Title) or any non-empty line as implicit entry
            let (title, has_header) = if line.starts_with("### ") {
                (line[4..].trim().to_string(), true)
            } else if current_section.is_some() {
                // Treat first line as implicit title for entries without ###
                (line.to_string(), false)
            } else {
                i += 1;
                continue;
            };

            if has_header || current_section.is_some() {
                // Collect content until next header or end
                let mut content_lines = Vec::new();
                let mut url: Option<String> = None;
                let mut percentage: Option<i64> = None;

                // For entries without headers, the first line might contain content
                if !has_header {
                    // The title line itself is the content for headerless entries
                    // We'll move to next line
                    i += 1;
                } else {
                    i += 1;
                }

                while i < lines.len() {
                    let content_line = lines[i];
                    let trimmed = content_line.trim();

                    // Stop at next section or entry header
                    if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
                        break;
                    }

                    // Stop at blank lines followed by non-empty lines (potential new entry)
                    // This allows separation of entries without explicit headers
                    if trimmed.is_empty() && i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if !next_line.is_empty()
                            && !next_line.starts_with("**")
                            && !next_line.starts_with("## ")
                            && !next_line.starts_with("### ") {
                            // Next entry starts after this blank line
                            i += 1; // Skip the blank line
                            break;
                        }
                    }

                    // Check for URL
                    if trimmed.starts_with("**URL:**") {
                        url = Some(trimmed[8..].trim().to_string());
                        i += 1;
                        continue;
                    }

                    // Check for Percentage
                    if trimmed.starts_with("**Percentage:**") {
                        let pct_str = trimmed[15..].trim().trim_end_matches('%');
                        if let Ok(pct) = pct_str.parse::<i64>() {
                            percentage = Some(pct);
                        }
                        i += 1;
                        continue;
                    }

                    // Skip empty lines at the end
                    if !trimmed.is_empty() || !content_lines.is_empty() {
                        content_lines.push(content_line);
                    }

                    i += 1;
                }

                // Remove trailing empty lines
                while content_lines.last().map_or(false, |l| l.trim().is_empty()) {
                    content_lines.pop();
                }

                let context = content_lines.join("\n");

                match current_section {
                    Some("OUTSIDE") => {
                        outside_entries.push(json!({
                            "name": title,
                            "context": context,
                            "url": url.unwrap_or_default(),
                            "percentage": percentage
                        }));
                    }
                    Some("INSIDE") => {
                        inside_entries.push(json!({
                            "date": title,
                            "context": context
                        }));
                    }
                    Some(_) | None => {
                        // Entry outside of any section or unknown section, skip
                    }
                }
            } else {
                i += 1;
            }
        }

        let json_value = json!({
            "outside": outside_entries,
            "inside": inside_entries
        });

        serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Convert current JSON to Markdown format (for saving .md files)
    pub fn convert_to_markdown(&self) -> Result<String, String> {
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

                                output_lines.push(format!("### {}", name));

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

                                if !context.is_empty() {
                                    output_lines.push(context.to_string());
                                }

                                output_lines.push("".to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(output_lines.join("\n"))
    }
}
