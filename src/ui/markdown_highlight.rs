use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

use crate::config::ColorScheme;

// Markdown syntax highlighting - only highlight **URL:** and **Percentage:**
pub fn highlight_markdown_line(line: &str, colorscheme: &ColorScheme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Just return plain text to avoid any byte index issues with multibyte characters
    // Highlighting **URL:** and **Percentage:** can be added later if needed
    spans.push(Span::styled(
        line.to_string(),
        Style::default().fg(colorscheme.text),
    ));

    spans
}
