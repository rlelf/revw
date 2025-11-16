use ratatui::{
    style::Style,
    text::Span,
};

use crate::config::ColorScheme;

pub fn highlight_markdown_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Check for headers (##, ###, etc.)
    if line.starts_with('#') {
        let header_end = line.chars().take_while(|c| *c == '#').count();
        if header_end > 0 && line.chars().nth(header_end) == Some(' ') {
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(colorscheme.md_header),
            ));
            return spans;
        }
    }

    // Parse line for bold text (**text**) and URLs
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut current_text = String::new();

    while i < chars.len() {
        // Check for bold text **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            // Flush current text
            if !current_text.is_empty() {
                spans.push(Span::styled(
                    current_text.clone(),
                    Style::default().fg(colorscheme.md_text),
                ));
                current_text.clear();
            }

            // Find closing **
            let start = i + 2;
            let mut end = start;
            while end + 1 < chars.len() {
                if chars[end] == '*' && chars[end + 1] == '*' {
                    break;
                }
                end += 1;
            }

            if end + 1 < chars.len() && chars[end] == '*' && chars[end + 1] == '*' {
                // Found closing **, extract bold text
                let bold_text: String = chars[start..end].iter().collect();
                let full_bold = format!("**{}**", bold_text);

                // Check if this is URL: or Percentage: (special highlighting)
                if bold_text == "URL:" || bold_text == "Percentage:" {
                    spans.push(Span::styled(
                        full_bold,
                        Style::default().fg(colorscheme.md_url),
                    ));
                } else {
                    spans.push(Span::styled(
                        full_bold,
                        Style::default().fg(colorscheme.md_bold),
                    ));
                }
                i = end + 2;
            } else {
                // No closing **, treat as normal text
                current_text.push(chars[i]);
                i += 1;
            }
        } else {
            current_text.push(chars[i]);
            i += 1;
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        spans.push(Span::styled(
            current_text,
            Style::default().fg(colorscheme.md_text),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            line.to_string(),
            Style::default().fg(colorscheme.md_text),
        ));
    }

    spans
}
