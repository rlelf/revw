use ratatui::style::Color;
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Debug, Default)]
pub struct RelfLineStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
}

#[derive(Clone, Debug)]
pub struct RelfEntry {
    pub lines: Vec<String>,
    pub bg_color: Color,
}

#[derive(Clone, Debug, Default)]
pub struct RelfRenderResult {
    pub lines: Vec<String>,
    pub styles: Vec<RelfLineStyle>,
    pub entries: Vec<RelfEntry>,
}

pub struct Renderer;

impl Renderer {
    pub fn display_width_str(s: &str) -> usize {
        s.chars()
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
    }

    pub fn prefix_display_width(s: &str, char_pos: usize) -> usize {
        s.chars()
            .take(char_pos)
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
    }

    pub fn slice_columns(s: &str, start_cols: usize, width_cols: usize) -> String {
        if width_cols == 0 {
            return String::new();
        }
        let mut sum = 0usize;
        let mut start_idx = 0usize;
        for (i, c) in s.chars().enumerate() {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if sum >= start_cols {
                start_idx = i;
                break;
            }
            sum += w;
            start_idx = i + 1;
        }
        let mut out = String::new();
        let mut used = 0usize;
        for c in s.chars().skip(start_idx) {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if used + w > width_cols {
                break;
            }
            out.push(c);
            used += w;
        }
        out
    }

    pub fn render_relf(json_input: &str, filter_pattern: &str) -> RelfRenderResult {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_input) {
            let mut result = RelfRenderResult::default();

            if let Some(obj) = json_value.as_object() {
                let card_bg = Color::Rgb(26, 28, 34);

                for (section_key, section_value) in obj {
                    if section_key == "outside" || section_key == "inside" {
                        if let Some(section_array) = section_value.as_array() {
                            for item in section_array {
                                if let Some(item_obj) = item.as_object() {
                                    if section_key == "outside" {

                                        let mut entry_lines = Vec::new();

                                        let name = item_obj
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let context = item_obj
                                            .get("context")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let url = item_obj
                                            .get("url")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let percentage = item_obj
                                            .get("percentage")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);

                                        entry_lines.push(name.to_string());
                                        if !context.is_empty() {
                                            entry_lines.push(context.to_string());
                                        }
                                        if !url.is_empty() {
                                            entry_lines.push(url.to_string());
                                        }
                                        entry_lines.push(format!("{}%", percentage));

                                        // Apply filter if pattern is provided
                                        if !filter_pattern.is_empty() {
                                            let matches = entry_lines.iter().any(|line| {
                                                line.to_lowercase().contains(&filter_pattern.to_lowercase())
                                            });
                                            if !matches {
                                                continue; // Skip this entry
                                            }
                                        }

                                        result.entries.push(RelfEntry {
                                            lines: entry_lines,
                                            bg_color: card_bg,
                                        });
                                    } else if section_key == "inside" {

                                        let mut entry_lines = Vec::new();
                                        for (_key, value) in item_obj {
                                            let value_str = match value {
                                                serde_json::Value::String(s) => s.clone(),
                                                serde_json::Value::Number(n) => n.to_string(),
                                                serde_json::Value::Bool(b) => b.to_string(),
                                                _ => value.to_string(),
                                            };

                                            if !value_str.is_empty() {
                                                entry_lines.push(value_str);
                                            }
                                        }

                                        // Apply filter if pattern is provided
                                        if !filter_pattern.is_empty() {
                                            let matches = entry_lines.iter().any(|line| {
                                                line.to_lowercase().contains(&filter_pattern.to_lowercase())
                                            });
                                            if !matches {
                                                continue; // Skip this entry
                                            }
                                        }

                                        result.entries.push(RelfEntry {
                                            lines: entry_lines,
                                            bg_color: card_bg,
                                        });
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

        let mut result = RelfRenderResult::default();
        result
            .lines
            .push("⚠ Not valid JSON - showing raw text file content".to_string());
        result.styles.push(RelfLineStyle {
            fg: Some(Color::Yellow),
            bg: None,
            bold: true,
        });
        result.lines.push("".to_string());
        result.styles.push(RelfLineStyle::default());
        for line in lines {
            result.lines.push(line);
            result.styles.push(RelfLineStyle::default());
        }
        result
    }

    pub fn render_json(json_input: &str) -> Vec<String> {
        json_input.lines().map(|line| line.to_string()).collect()
    }
}
