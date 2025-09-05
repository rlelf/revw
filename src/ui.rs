use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::{App, FormatMode, InputMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());
    
    render_content(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    // Update the app's notion of the current content width for accurate wrapping
    // Use inner area width (inside borders and margins)
    app.content_width = inner_area.width;
    // Clamp horizontal scroll when width changes (Relf mode)
    if app.format_mode == FormatMode::Relf {
        let max_off = app.relf_max_hscroll();
        if app.hscroll > max_off { app.hscroll = max_off; }
    }
    // Remember actual visible height for correct scroll math elsewhere
    app.visible_height = inner_area.height;
    // Build visual (wrapped) lines and compute scroll bounds in visual rows
    let visual_lines = app.build_visual_lines();
    let lines_count = visual_lines.len() as u16;
    let visible_height = inner_area.height;
    let bottom_padding = if app.format_mode == FormatMode::Relf { 0 } else { 10u16 }; // No padding in Relf
    let padded_lines_count = lines_count + bottom_padding;
    app.max_scroll = padded_lines_count.saturating_sub(visible_height);

    let empty_line = String::new();
    let visible_content: Vec<_> = visual_lines
        .iter()
        .skip(app.scroll as usize)
        .chain(std::iter::repeat(&empty_line).take(bottom_padding as usize))
        .take(visible_height as usize)
        .collect();

    // Build content with cursor and horizontal viewport
    let content_text = {
        let w_cols = app.get_content_width() as usize;
        let off_cols = app.hscroll as usize;
        let mut lines_processed: Vec<String> = Vec::new();
        for (line_idx, s) in visible_content.iter().enumerate() {
            let actual_idx = line_idx + app.scroll as usize;
            let slice = app.slice_columns(s, off_cols, w_cols);
            if app.format_mode == FormatMode::Json && (app.input_mode == InputMode::Insert || app.input_mode == InputMode::Normal) && app.show_cursor {
                if actual_idx == app.content_cursor_line {
                    let cursor_char_pos = app.content_cursor_col;
                    let prefix_cols = app.prefix_display_width(s, cursor_char_pos);
                    if prefix_cols >= off_cols && prefix_cols < off_cols + w_cols {
                        let mut line_chars: Vec<char> = slice.chars().collect();
                        let insert_col_in_view = prefix_cols - off_cols;
                        let insert_idx = app.char_index_for_col(&slice, insert_col_in_view);
                        let cursor_ch = if app.input_mode == InputMode::Normal { '▮' } else { '│' };
                        let pos = insert_idx.min(line_chars.len());
                        line_chars.insert(pos, cursor_ch);
                        lines_processed.push(line_chars.into_iter().collect());
                        continue;
                    }
                }
            }
            lines_processed.push(slice);
        }
        lines_processed.join("\n")
    };

    let title = match &app.file_path {
        Some(path) => format!(" {} ", path.display()),
        None => String::new(),
    };

    let content = Paragraph::new(content_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(content, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    if !app.status_message.is_empty() {
        let status_text = format!(" {} ", app.status_message);
        
        let status_widget = Paragraph::new(Line::from(vec![
            Span::styled(status_text, Style::default().fg(Color::Cyan)),
        ]))
        .alignment(Alignment::Left);
        
        f.render_widget(status_widget, area);
    } else {
        // Empty status bar when no message
        let empty_widget = Paragraph::new("");
        f.render_widget(empty_widget, area);
    }
}