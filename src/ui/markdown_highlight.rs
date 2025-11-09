use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use crate::config::ColorScheme;

// Markdown syntax highlighting
pub fn highlight_markdown_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Check for headers (##, ###, etc.)
    if line.trim_start().starts_with('#') {
        let trimmed = line.trim_start();
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

        if hash_count > 0 && (hash_count >= trimmed.len() || trimmed.chars().nth(hash_count).map(|c| c.is_whitespace()).unwrap_or(false)) {
            // Valid header
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(colorscheme.key).add_modifier(Modifier::BOLD),
            ));
            return spans;
        }
    }

    // Check for bold markdown (**text**)
    if line.contains("**") {
        let mut chars = line.chars().collect::<Vec<_>>();
        let mut current = String::new();
        let mut i = 0;

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                // Found potential bold marker
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(colorscheme.text),
                    ));
                    current.clear();
                }

                // Find closing **
                let mut bold_text = String::from("**");
                i += 2;
                let mut found_closing = false;

                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '*' {
                        bold_text.push_str("**");
                        i += 2;
                        found_closing = true;
                        break;
                    }
                    bold_text.push(chars[i]);
                    i += 1;
                }

                if found_closing {
                    spans.push(Span::styled(
                        bold_text,
                        Style::default().fg(colorscheme.key).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // No closing **, treat as normal text
                    current.push_str(&bold_text);
                }
            } else {
                current.push(chars[i]);
                i += 1;
            }
        }

        if !current.is_empty() {
            spans.push(Span::styled(
                current,
                Style::default().fg(colorscheme.text),
            ));
        }
    } else {
        // No markdown formatting, just plain text
        spans.push(Span::styled(
            line.to_string(),
            Style::default().fg(colorscheme.text),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            String::new(),
            Style::default().fg(colorscheme.text),
        ));
    }

    spans
}
