use chrono::Local;
use crate::content_ops::ContentOperations;

pub struct MarkdownOperations;

impl MarkdownOperations {
    /// Parse Markdown content to find entry boundaries
    fn parse_entries(content: &str) -> Vec<Entry> {
        let mut entries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_section = None;

        while i < lines.len() {
            let line = lines[i].trim();

            if line == "## OUTSIDE" {
                current_section = Some(Section::Outside);
                i += 1;
                continue;
            } else if line == "## INSIDE" {
                current_section = Some(Section::Inside);
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
                let start_line = i;

                let mut content_lines = Vec::new();
                let mut url = String::new();
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
                    // This only applies to entries WITHOUT ### headers
                    if !has_header && trimmed.is_empty() && i + 1 < lines.len() {
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

                    if trimmed.starts_with("**URL:**") {
                        url = trimmed[8..].trim().to_string();
                        i += 1;
                        continue;
                    }

                    if trimmed.starts_with("**Percentage:**") {
                        let pct_str = trimmed[15..].trim().trim_end_matches('%');
                        if let Ok(pct) = pct_str.parse::<i64>() {
                            percentage = Some(pct);
                        }
                        i += 1;
                        continue;
                    }

                    content_lines.push(content_line);
                    i += 1;
                }

                while content_lines.last().map_or(false, |l| l.trim().is_empty()) {
                    content_lines.pop();
                }

                let context = content_lines.join("\n");
                let end_line = i - 1;

                if let Some(section) = current_section {
                    entries.push(Entry {
                        section,
                        title,
                        context,
                        url,
                        percentage,
                        start_line,
                        end_line,
                    });
                }
            } else {
                i += 1;
            }
        }

        entries
    }

    /// Delete an entry at the cursor position
    pub fn delete_entry_at_cursor(
        markdown_input: &str,
        cursor_line: usize,
    ) -> Result<(String, String), String> {
        let entries = Self::parse_entries(markdown_input);

        // Find the entry that contains the cursor
        let entry_to_delete = entries.iter().position(|entry| {
            cursor_line >= entry.start_line && cursor_line <= entry.end_line
        });

        if let Some(idx) = entry_to_delete {
            let lines: Vec<&str> = markdown_input.lines().collect();
            let mut result_lines = Vec::new();

            let entry = &entries[idx];

            // Add all lines except the deleted entry
            for (i, line) in lines.iter().enumerate() {
                if i < entry.start_line || i > entry.end_line {
                    result_lines.push(line.to_string());
                } else if i == entry.end_line {
                    // Skip the trailing blank line after the entry
                    continue;
                }
            }

            // Remove duplicate blank lines
            let mut final_lines = Vec::new();
            let mut prev_blank = false;
            for line in result_lines {
                let is_blank = line.trim().is_empty();
                if !(is_blank && prev_blank) {
                    final_lines.push(line);
                }
                prev_blank = is_blank;
            }

            Ok((final_lines.join("\n"), "Entry deleted".to_string()))
        } else {
            Err("Could not delete entry at cursor position".to_string())
        }
    }

    /// Add a new inside entry
    pub fn add_inside_entry(markdown_input: &str) -> Result<(String, usize, usize, String), String> {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let new_entry = format!("### {}\n", date_str);

        let lines: Vec<&str> = markdown_input.lines().collect();
        let mut result_lines = Vec::new();
        let mut inside_section_start = None;
        let mut inserted = false;
        let mut insert_line = 0;

        for (i, line) in lines.iter().enumerate() {
            result_lines.push(line.to_string());

            if line.trim() == "## INSIDE" {
                inside_section_start = Some(i);
            }

            // Insert after "## INSIDE" header
            if let Some(section_start) = inside_section_start {
                if i == section_start && !inserted {
                    result_lines.push("".to_string());
                    result_lines.push(new_entry.trim().to_string());
                    insert_line = result_lines.len() - 1;
                    result_lines.push("".to_string());
                    inserted = true;
                }
            }
        }

        // If no INSIDE section exists, create it
        if inside_section_start.is_none() {
            if !result_lines.is_empty() {
                result_lines.push("".to_string());
            }
            result_lines.push("## INSIDE".to_string());
            result_lines.push("".to_string());
            result_lines.push(new_entry.trim().to_string());
            insert_line = result_lines.len() - 1;
            result_lines.push("".to_string());
        }

        let formatted = result_lines.join("\n");

        // Calculate cursor position (after the title, ready to type context)
        let _col = new_entry.len();

        Ok((formatted, insert_line + 1, 0, "Added inside".to_string()))
    }

    /// Add a new outside entry
    pub fn add_outside_entry(markdown_input: &str) -> Result<(String, usize, usize, String), String> {
        let new_entry = "### ";

        let lines: Vec<&str> = markdown_input.lines().collect();
        let mut result_lines = Vec::new();
        let mut outside_section_start = None;
        let mut inserted = false;
        let mut insert_line = 0;

        // If input is empty, create the structure
        if markdown_input.trim().is_empty() {
            result_lines.push("## OUTSIDE".to_string());
            result_lines.push("".to_string());
            result_lines.push(new_entry.to_string());
            insert_line = 2;
            let col = new_entry.len();
            return Ok((result_lines.join("\n"), insert_line, col, "Added outside".to_string()));
        }

        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "## OUTSIDE" {
                outside_section_start = Some(i);
                result_lines.push(line.to_string());
                continue;
            }

            // Insert before INSIDE section if we're in OUTSIDE section
            if line.trim() == "## INSIDE" && outside_section_start.is_some() && !inserted {
                result_lines.push(new_entry.to_string());
                insert_line = result_lines.len() - 1;
                result_lines.push("".to_string());
                inserted = true;
            }

            result_lines.push(line.to_string());
        }

        // If OUTSIDE section exists but no INSIDE section, append at the end
        if outside_section_start.is_some() && !inserted {
            result_lines.push("".to_string());
            result_lines.push(new_entry.to_string());
            insert_line = result_lines.len() - 1;
            result_lines.push("".to_string());
        }

        // If no OUTSIDE section exists, create it at the beginning
        if outside_section_start.is_none() {
            let mut new_result = vec!["## OUTSIDE".to_string(), "".to_string(), new_entry.to_string(), "".to_string()];
            insert_line = 2;
            new_result.extend(result_lines);
            result_lines = new_result;
        }

        let formatted = result_lines.join("\n");
        let col = new_entry.len();

        Ok((formatted, insert_line, col, "Added outside".to_string()))
    }

    /// Duplicate an entry at the cursor position
    pub fn duplicate_entry_at_cursor(
        markdown_input: &str,
        cursor_line: usize,
    ) -> Result<(String, String), String> {
        let entries = Self::parse_entries(markdown_input);

        let entry_to_duplicate = entries.iter().position(|entry| {
            cursor_line >= entry.start_line && cursor_line <= entry.end_line
        });

        if let Some(idx) = entry_to_duplicate {
            let lines: Vec<&str> = markdown_input.lines().collect();
            let mut result_lines = Vec::new();

            let entry = &entries[idx];

            for (i, line) in lines.iter().enumerate() {
                result_lines.push(line.to_string());

                // After the entry ends, insert the duplicate
                if i == entry.end_line {
                    result_lines.push("".to_string());
                    // Reconstruct the entry
                    result_lines.push(format!("### {}", entry.title));
                    if !entry.context.is_empty() {
                        result_lines.push(entry.context.clone());
                    }
                    if !entry.url.is_empty() {
                        result_lines.push(format!("**URL:** {}", entry.url));
                    }
                    if let Some(pct) = entry.percentage {
                        result_lines.push(format!("**Percentage:** {}%", pct));
                    }
                }
            }

            Ok((result_lines.join("\n"), "Entry duplicated".to_string()))
        } else {
            Err("Could not duplicate entry at cursor position".to_string())
        }
    }

    /// Order entries (outside by percentage desc, then name asc; inside by date desc)
    pub fn order_entries(markdown_input: &str) -> Result<(String, String), String> {
        let entries = Self::parse_entries(markdown_input);

        let mut outside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Outside))
            .cloned()
            .collect();

        let mut inside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Inside))
            .cloned()
            .collect();

        // Sort outside by percentage desc, then name asc
        outside_entries.sort_by(|a, b| {
            b.percentage.unwrap_or(0)
                .cmp(&a.percentage.unwrap_or(0))
                .then_with(|| a.title.cmp(&b.title))
        });

        // Sort inside by date desc (newest first)
        inside_entries.sort_by(|a, b| b.title.cmp(&a.title));

        Ok((Self::reconstruct_markdown(&outside_entries, &inside_entries), "Ordered".to_string()))
    }

    /// Order entries by percentage only
    pub fn order_by_percentage(markdown_input: &str) -> Result<(String, String), String> {
        let entries = Self::parse_entries(markdown_input);

        let mut outside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Outside))
            .cloned()
            .collect();

        let mut inside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Inside))
            .cloned()
            .collect();

        // Sort outside by percentage desc only
        outside_entries.sort_by(|a, b| {
            b.percentage.unwrap_or(0).cmp(&a.percentage.unwrap_or(0))
        });

        // Sort inside by date desc
        inside_entries.sort_by(|a, b| b.title.cmp(&a.title));

        Ok((Self::reconstruct_markdown(&outside_entries, &inside_entries), "Ordered by percentage".to_string()))
    }

    /// Order entries by name only
    pub fn order_by_name(markdown_input: &str) -> Result<(String, String), String> {
        let entries = Self::parse_entries(markdown_input);

        let mut outside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Outside))
            .cloned()
            .collect();

        let mut inside_entries: Vec<_> = entries.iter()
            .filter(|e| matches!(e.section, Section::Inside))
            .cloned()
            .collect();

        // Sort outside by name asc only
        outside_entries.sort_by(|a, b| a.title.cmp(&b.title));

        // Sort inside by date desc
        inside_entries.sort_by(|a, b| b.title.cmp(&a.title));

        Ok((Self::reconstruct_markdown(&outside_entries, &inside_entries), "Ordered by name".to_string()))
    }

    /// Reconstruct markdown from sorted entries
    fn reconstruct_markdown(outside_entries: &[Entry], inside_entries: &[Entry]) -> String {
        let mut lines = Vec::new();

        if !outside_entries.is_empty() {
            lines.push("## OUTSIDE".to_string());
            lines.push("".to_string());

            for entry in outside_entries {
                lines.push(format!("### {}", entry.title));
                if !entry.context.is_empty() {
                    lines.push(entry.context.clone());
                }
                if !entry.url.is_empty() {
                    lines.push("".to_string());
                    lines.push(format!("**URL:** {}", entry.url));
                }
                if let Some(pct) = entry.percentage {
                    lines.push("".to_string());
                    lines.push(format!("**Percentage:** {}%", pct));
                }
                lines.push("".to_string());
            }
        }

        if !inside_entries.is_empty() {
            lines.push("## INSIDE".to_string());
            lines.push("".to_string());

            for entry in inside_entries {
                lines.push(format!("### {}", entry.title));
                if !entry.context.is_empty() {
                    lines.push(entry.context.clone());
                }
                lines.push("".to_string());
            }
        }

        lines.join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    Outside,
    Inside,
}

#[derive(Debug, Clone)]
struct Entry {
    section: Section,
    title: String,
    context: String,
    url: String,
    percentage: Option<i64>,
    start_line: usize,
    end_line: usize,
}

// Implement ContentOperations trait for MarkdownOperations
impl ContentOperations for MarkdownOperations {
    fn add_inside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        MarkdownOperations::add_inside_entry(content)
    }

    fn add_outside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String> {
        MarkdownOperations::add_outside_entry(content)
    }

    fn delete_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        _lines: &[String],
    ) -> Result<(String, String), String> {
        MarkdownOperations::delete_entry_at_cursor(content, cursor_line)
    }

    fn duplicate_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        _lines: &[String],
    ) -> Result<(String, String), String> {
        MarkdownOperations::duplicate_entry_at_cursor(content, cursor_line)
    }

    fn order_entries(&self, content: &str) -> Result<(String, String), String> {
        MarkdownOperations::order_entries(content)
    }

    fn order_by_percentage(&self, content: &str) -> Result<(String, String), String> {
        MarkdownOperations::order_by_percentage(content)
    }

    fn order_by_name(&self, content: &str) -> Result<(String, String), String> {
        MarkdownOperations::order_by_name(content)
    }
}
