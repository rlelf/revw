use ratatui::{
    style::{Color, Style},
    text::Span,
};

use crate::config::ColorScheme;

// JSON syntax highlighting
pub fn highlight_json_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Push accumulated text
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(colorscheme.text),
                    ));
                    current.clear();
                }

                // Start collecting string
                let mut string_content = String::from("\"");
                let mut escaped = false;

                while let Some(next_ch) = chars.next() {
                    string_content.push(next_ch);
                    if next_ch == '\\' && !escaped {
                        escaped = true;
                    } else if next_ch == '"' && !escaped {
                        break;
                    } else {
                        escaped = false;
                    }
                }

                // Determine if this is a key (followed by ':')
                let mut temp_chars = chars.clone();
                let mut is_key = false;
                while let Some(peek_ch) = temp_chars.next() {
                    if peek_ch == ':' {
                        is_key = true;
                        break;
                    } else if !peek_ch.is_whitespace() {
                        break;
                    }
                }

                let color = if is_key {
                    colorscheme.key // Keys in light blue
                } else {
                    colorscheme.string // String values in orange/peach
                };

                spans.push(Span::styled(
                    string_content,
                    Style::default().fg(color),
                ));
            }
            '{' | '}' | '[' | ']' => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(colorscheme.text),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(colorscheme.bracket), // Yellow/gold
                ));
            }
            ':' | ',' => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(colorscheme.text),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
            't' | 'f' | 'n' => {
                // Check for true, false, null
                let peek_str: String = std::iter::once(ch)
                    .chain(chars.clone().take(4))
                    .collect();

                if peek_str.starts_with("true") || peek_str.starts_with("false") || peek_str.starts_with("null") {
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(colorscheme.text),
                        ));
                        current.clear();
                    }

                    let keyword = if peek_str.starts_with("true") {
                        chars.nth(2); // skip 'r', 'u', 'e'
                        "true"
                    } else if peek_str.starts_with("false") {
                        chars.nth(3); // skip 'a', 'l', 's', 'e'
                        "false"
                    } else {
                        chars.nth(2); // skip 'u', 'l', 'l'
                        "null"
                    };

                    spans.push(Span::styled(
                        keyword.to_string(),
                        Style::default().fg(colorscheme.boolean), // Purple/blue
                    ));
                } else {
                    current.push(ch);
                }
            }
            '0'..='9' | '-' => {
                // Numbers
                let mut num = String::from(ch);
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' || next_ch == 'e' || next_ch == 'E' || next_ch == '-' || next_ch == '+' {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(colorscheme.text),
                    ));
                    current.clear();
                }

                spans.push(Span::styled(
                    num,
                    Style::default().fg(colorscheme.number), // Light green
                ));
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(
            current,
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
