use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    symbols::scrollbar,
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
        let mut lines_vec: Vec<Line> = Vec::new();

        for (line_idx, s) in visible_content.iter().enumerate() {
            let actual_idx = line_idx + app.scroll as usize;
            let slice = app.slice_columns(s, off_cols, w_cols);

            // Build spans for the line with search highlighting
            let mut spans: Vec<Span> = Vec::new();

            if !app.search_query.is_empty() {
                // Highlight search matches
                let query_lower = app.search_query.to_lowercase();
                let line_lower = slice.to_lowercase();
                let mut last_pos = 0;

                while let Some(match_pos) = line_lower[last_pos..].find(&query_lower) {
                    let actual_pos = last_pos + match_pos;

                    // Add text before match
                    if actual_pos > last_pos {
                        spans.push(Span::styled(
                            slice[last_pos..actual_pos].to_string(),
                            Style::default().fg(Color::Gray)
                        ));
                    }

                    // Check if this is the current match
                    let is_current_match = app.current_match_index
                        .and_then(|idx| app.search_matches.get(idx))
                        .map(|(line, col)| *line == actual_idx && *col == actual_pos + off_cols)
                        .unwrap_or(false);

                    // Add highlighted match
                    let match_end = actual_pos + app.search_query.len();
                    let highlight_style = if is_current_match {
                        Style::default().fg(Color::Black).bg(Color::Yellow) // Current match
                    } else {
                        Style::default().fg(Color::Black).bg(Color::Cyan) // Other matches
                    };

                    spans.push(Span::styled(
                        slice[actual_pos..match_end.min(slice.len())].to_string(),
                        highlight_style
                    ));

                    last_pos = match_end;
                }

                // Add remaining text after last match
                if last_pos < slice.len() {
                    spans.push(Span::styled(
                        slice[last_pos..].to_string(),
                        Style::default().fg(Color::Gray)
                    ));
                }
            } else {
                // No search highlighting, just use plain text
                spans.push(Span::styled(slice.clone(), Style::default().fg(Color::Gray)));
            }

            // Add cursor if needed
            if app.format_mode == FormatMode::Json && (app.input_mode == InputMode::Insert || app.input_mode == InputMode::Normal) && app.show_cursor {
                if actual_idx == app.content_cursor_line {
                    let cursor_char_pos = app.content_cursor_col;
                    let prefix_cols = app.prefix_display_width(s, cursor_char_pos);
                    if prefix_cols >= off_cols && prefix_cols < off_cols + w_cols {
                        // Always show cursor as vertical bar
                        let line_chars: Vec<char> = slice.chars().collect();
                        let insert_col_in_view = prefix_cols - off_cols;
                        let insert_idx = app.char_index_for_col(&slice, insert_col_in_view);

                        let mut new_chars = line_chars.clone();
                        let pos = insert_idx.min(new_chars.len());
                        new_chars.insert(pos, '│');
                        spans = vec![Span::styled(
                            new_chars.into_iter().collect::<String>(),
                            Style::default().fg(Color::Gray)
                        )];
                    }
                }
            }

            lines_vec.push(Line::from(spans));
        }

        lines_vec
    };

    let title = match &app.file_path {
        Some(path) => format!(" {} ", path.display()),
        None => String::new(),
    };

    let content = Paragraph::new(content_text)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(content, area);

    // Only show scrollbars in Relf mode (not in JSON edit mode)
    if app.format_mode == FormatMode::Relf {
        // Render vertical scrollbar
        let scrollbar_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });

        let mut scrollbar_state = ScrollbarState::new(app.max_scroll as usize)
            .position(app.scroll as usize);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::DarkGray))
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .end_symbol(None);

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

        // Render horizontal scrollbar
        let max_hscroll = app.relf_max_hscroll();

        if max_hscroll > 0 {
            // Use custom rendering for thinner horizontal scrollbar
            let hscroll_y = area.y + area.height - 1;
            let hscroll_start_x = area.x + 1;
            let hscroll_width = area.width.saturating_sub(2) as usize;

            if hscroll_width > 0 && max_hscroll > 0 {
                // Calculate thumb position and size
                let thumb_size = ((hscroll_width as f32 / (hscroll_width as f32 + max_hscroll as f32)) * hscroll_width as f32).max(1.0) as usize;
                let thumb_pos = ((app.hscroll as f32 / max_hscroll as f32) * (hscroll_width - thumb_size) as f32) as usize;

                // Build the scrollbar line
                let mut scrollbar_line = String::new();
                for i in 0..hscroll_width {
                    if i >= thumb_pos && i < thumb_pos + thumb_size {
                        scrollbar_line.push('█'); // Thumb
                    } else {
                        scrollbar_line.push('─'); // Track
                    }
                }

                // Render the custom horizontal scrollbar
                let hscrollbar_widget = Paragraph::new(scrollbar_line)
                    .style(Style::default().fg(Color::DarkGray));

                let hscrollbar_rect = Rect {
                    x: hscroll_start_x,
                    y: hscroll_y,
                    width: hscroll_width as u16,
                    height: 1,
                };

                f.render_widget(hscrollbar_widget, hscrollbar_rect);
            }
        }
    }
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