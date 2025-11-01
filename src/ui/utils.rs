use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::app::App;
use crate::rendering::RelfLineStyle;

// Slice spans by display width (for horizontal scrolling)
pub fn slice_spans_by_width(app: &App, spans: Vec<Span>, start_col: usize, width: usize) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let mut current_col = 0;
    let end_col = start_col + width;

    for span in spans {
        let text = span.content.to_string();
        let span_width = app.display_width_str(&text);
        let span_start = current_col;
        let span_end = current_col + span_width;

        if span_end <= start_col {
            // This span is entirely before the visible range
            current_col = span_end;
            continue;
        }

        if span_start >= end_col {
            // This span is entirely after the visible range
            break;
        }

        // This span overlaps with visible range - need to slice it
        let visible_start = if span_start < start_col {
            start_col - span_start
        } else {
            0
        };

        let visible_end = if span_end > end_col {
            span_width - (span_end - end_col)
        } else {
            span_width
        };

        // Slice the text by display width
        let sliced_text = app.slice_columns(&text, visible_start, visible_end - visible_start);

        if !sliced_text.is_empty() {
            result.push(Span::styled(sliced_text, span.style));
        }

        current_col = span_end;
    }

    result
}

pub fn highlight_search_in_line(line: &str, query: &str, base_style: Style) -> Line<'static> {
    let query_lower = query.to_lowercase();
    let line_lower = line.to_lowercase();
    let mut spans = Vec::new();
    let mut byte_pos = 0;

    while byte_pos < line_lower.len() {
        if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
            let actual_byte_pos = byte_pos + match_pos;

            // Add text before match (ensuring char boundaries)
            if actual_byte_pos > byte_pos && line.is_char_boundary(byte_pos) && line.is_char_boundary(actual_byte_pos) {
                spans.push(Span::styled(
                    line[byte_pos..actual_byte_pos].to_string(),
                    base_style,
                ));
            }

            // Add highlighted match (ensuring char boundaries)
            let match_end_byte = actual_byte_pos + query_lower.len();
            if line.is_char_boundary(actual_byte_pos) && match_end_byte <= line.len() {
                let safe_end = if line.is_char_boundary(match_end_byte) {
                    match_end_byte
                } else {
                    // Find next char boundary
                    (match_end_byte..=line.len())
                        .find(|&i| line.is_char_boundary(i))
                        .unwrap_or(line.len())
                };

                spans.push(Span::styled(
                    line[actual_byte_pos..safe_end].to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ));
                byte_pos = safe_end;
            } else {
                byte_pos = match_end_byte;
            }

            // Ensure we're on a char boundary
            while byte_pos < line.len() && !line.is_char_boundary(byte_pos) {
                byte_pos += 1;
            }
        } else {
            break;
        }
    }

    // Add remaining text after last match
    if byte_pos < line.len() && line.is_char_boundary(byte_pos) {
        spans.push(Span::styled(line[byte_pos..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(line.to_string(), base_style));
    }

    Line::from(spans)
}

pub fn apply_relf_style(mut style: Style, line_style: Option<&RelfLineStyle>) -> Style {
    if let Some(ls) = line_style {
        if let Some(fg) = ls.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = ls.bg {
            style = style.bg(bg);
        }
        if ls.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
    }
    style
}
