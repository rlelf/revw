use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, FormatMode};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    // Left side: status message
    if !app.status_message.is_empty() {
        let status_text = format!(" {} ", app.status_message);
        spans.push(Span::styled(
            status_text,
            Style::default().fg(app.colorscheme.status_bar),
        ));
    }

    // Right side: cursor position in Edit mode
    if app.format_mode == FormatMode::Edit {
        let current_line = app.content_cursor_line + 1;
        let current_col = app.content_cursor_col + 1;
        let position_text = format!("{}:{} ", current_line, current_col);

        // Calculate padding to right-align
        let status_width = if !app.status_message.is_empty() {
            app.status_message.len() + 2
        } else {
            0
        };
        let position_width = position_text.len();
        let available_width = area.width as usize;

        if available_width > status_width + position_width {
            let padding_width = available_width - status_width - position_width;
            spans.push(Span::raw(" ".repeat(padding_width)));
        }

        spans.push(Span::styled(
            position_text,
            Style::default().fg(Color::DarkGray),
        ));
    }

    let status_widget = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left);

    f.render_widget(status_widget, area);
}
