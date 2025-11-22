use ratatui::{
    style::Style,
    text::Span,
};

use crate::config::ColorScheme;
use crate::syntax_highlight::SyntaxHighlighter;

pub fn highlight_markdown_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Check for code block markers (```)
    if line.trim_start().starts_with("```") {
        spans.push(Span::styled(
            line.to_string(),
            Style::default().fg(colorscheme.md_url), // Use URL color for code fence
        ));
        return spans;
    }

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

    // Check for list markers (-, *, +)
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let marker_and_space = &trimmed[..2];
        let content = &trimmed[2..];

        // Render indent as normal text
        if !indent.is_empty() {
            spans.push(Span::styled(
                indent.to_string(),
                Style::default().fg(colorscheme.md_text),
            ));
        }

        // Render list marker in special color
        spans.push(Span::styled(
            marker_and_space.to_string(),
            Style::default().fg(colorscheme.md_url), // Use URL color for list markers
        ));

        // Process the rest of the line for bold text
        let content_spans = parse_bold_in_line(content, colorscheme);
        spans.extend(content_spans);
        return spans;
    }

    // Parse line for bold text (**text**)
    let content_spans = parse_bold_in_line(line, colorscheme);
    spans.extend(content_spans);

    spans
}

/// Parse bold text (**text**) in a line
fn parse_bold_in_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
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

/// Highlight markdown with code block support (for Edit mode)
pub fn highlight_markdown_with_code_blocks(
    lines: &[String],
    colorscheme: &ColorScheme,
    syntax_highlighter: Option<&SyntaxHighlighter>,
) -> Vec<Vec<Span<'static>>> {
    let mut result = Vec::new();
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut code_lines: Vec<String> = Vec::new();

    for line in lines {
        // Check for code block markers
        if line.trim_start().starts_with("```") {
            if in_code_block {
                // End of code block - highlight accumulated code
                if let Some(highlighter) = syntax_highlighter {
                    let code = code_lines.join("\n");
                    let highlighted = highlighter.highlight_code(&code, code_lang.as_deref());

                    // Add the highlighted lines to result
                    for highlighted_line in highlighted {
                        result.push(highlighted_line.spans);
                    }
                } else {
                    // No highlighter - just use plain text
                    for code_line in &code_lines {
                        result.push(vec![Span::styled(
                            code_line.to_string(),
                            Style::default().fg(colorscheme.md_text),
                        )]);
                    }
                }

                code_lines.clear();
                code_lang = None;
                in_code_block = false;

                // Add the closing ``` fence
                result.push(vec![Span::styled(
                    line.to_string(),
                    Style::default().fg(colorscheme.md_url),
                )]);
            } else {
                // Start of code block
                in_code_block = true;
                let lang_str = line.trim_start()[3..].trim();
                if !lang_str.is_empty() {
                    code_lang = Some(lang_str.to_string());
                }

                // Add the opening ``` fence
                result.push(vec![Span::styled(
                    line.to_string(),
                    Style::default().fg(colorscheme.md_url),
                )]);
            }
        } else if in_code_block {
            // Inside code block - accumulate lines
            code_lines.push(line.to_string());
        } else {
            // Regular markdown line
            result.push(highlight_markdown_line(line, colorscheme));
        }
    }

    // Handle unclosed code block
    if in_code_block && !code_lines.is_empty() {
        if let Some(highlighter) = syntax_highlighter {
            let code = code_lines.join("\n");
            let highlighted = highlighter.highlight_code(&code, code_lang.as_deref());
            for highlighted_line in highlighted {
                result.push(highlighted_line.spans);
            }
        } else {
            for code_line in &code_lines {
                result.push(vec![Span::styled(
                    code_line.to_string(),
                    Style::default().fg(colorscheme.md_text),
                )]);
            }
        }
    }

    result
}
