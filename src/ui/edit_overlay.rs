use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render_edit_overlay(f: &mut Frame, app: &App) {
    // Create a centered popup area
    let area = f.area();

    let popup_width = area.width.min(80);
    // Use 70% of screen height
    let popup_height = ((area.height * 7) / 10).max(10).min(area.height - 4);

    // Align x to even column to prevent wide-char (CJK) rendering issues with borders
    let x_centered = (area.width.saturating_sub(popup_width)) / 2;
    let x_aligned = x_centered & !1; // Force to even number

    let popup_area = Rect {
        x: x_aligned,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Create a slightly wider clear area to avoid cutting wide characters at boundaries
    let clear_area = Rect {
        x: x_aligned.saturating_sub(1),
        y: popup_area.y,
        width: popup_width.saturating_add(2).min(area.width.saturating_sub(x_aligned.saturating_sub(1))),
        height: popup_height,
    };

    // Clear the wider area to fully erase any wide characters
    f.render_widget(Clear, clear_area);

    // Fill the clear area with background color using spaces
    let blank_lines: Vec<Line> = (0..clear_area.height)
        .map(|_| Line::from(" ".repeat(clear_area.width as usize)))
        .collect();
    let blank_paragraph = Paragraph::new(blank_lines)
        .style(Style::default().bg(app.colorscheme.background));
    f.render_widget(blank_paragraph, clear_area);

    // Determine if editing INSIDE or OUTSIDE entry
    // INSIDE: date, context (2 fields)
    // OUTSIDE: name, context, url, percentage (4 fields)
    let is_inside = app.edit_buffer.len() == 2;
    let title = if is_inside {
        " INSIDE "
    } else {
        " OUTSIDE "
    };

    // Render the popup border
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(app.colorscheme.background).fg(Color::White));

    f.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);

    // Render fields on borders and content
    if is_inside {
        render_inside_overlay(f, app, popup_area, inner_area);
    } else {
        render_outside_overlay(f, app, popup_area, inner_area);
    }
}

fn render_inside_overlay(f: &mut Frame, app: &App, card_area: Rect, inner_area: Rect) {
    // Field indices for INSIDE: 0=date, 1=context

    // Date on top-left border
    if !app.edit_buffer.is_empty() {
        let is_selected = app.edit_field_index == 0;
        let is_placeholder = app.edit_buffer_is_placeholder.get(0).copied().unwrap_or(false);

        let style = get_field_style(app, is_selected, is_placeholder);

        let mut date_text = format!(" {} ", app.edit_buffer[0].clone());

        // Add cursor if editing this field
        if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            date_text = add_cursor_to_text(&date_text, app.edit_cursor_pos, 1); // offset by 1 for leading space
        }

        let date_line = Line::styled(date_text, style);
        let date_area = Rect {
            x: card_area.x + 2,
            y: card_area.y,
            width: card_area.width.saturating_sub(4),
            height: 1
        };
        let date_para = Paragraph::new(date_line).alignment(Alignment::Left);
        f.render_widget(date_para, date_area);
    }

    // Context in the middle (always render with newlines)
    if app.edit_buffer.len() >= 2 {
        render_context_field(f, app, inner_area, 1);
    }
}

fn render_outside_overlay(f: &mut Frame, app: &App, card_area: Rect, inner_area: Rect) {
    // Field indices for OUTSIDE: 0=name, 1=context, 2=url, 3=percentage

    // Name on top-left border
    if !app.edit_buffer.is_empty() {
        let is_selected = app.edit_field_index == 0;
        let is_placeholder = app.edit_buffer_is_placeholder.get(0).copied().unwrap_or(false);

        let style = get_field_style(app, is_selected, is_placeholder);

        let name_area = Rect {
            x: card_area.x + 2,
            y: card_area.y,
            width: card_area.width.saturating_sub(4),
            height: 1
        };

        // Add cursor and handle horizontal scrolling if editing this field
        let name_text = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            render_scrollable_field(&app.edit_buffer[0], app.edit_cursor_pos, name_area.width as usize, 1)
        } else {
            format!(" {} ", app.edit_buffer[0].clone())
        };

        let name_line = Line::styled(name_text, style);
        let name_para = Paragraph::new(name_line).alignment(Alignment::Left);
        f.render_widget(name_para, name_area);
    }

    // URL on bottom-left border (render first)
    if app.edit_buffer.len() >= 3 {
        let is_selected = app.edit_field_index == 2;
        let is_placeholder = app.edit_buffer_is_placeholder.get(2).copied().unwrap_or(false);

        let style = get_field_style(app, is_selected, is_placeholder);

        let url_area = Rect {
            x: card_area.x + 2,
            y: card_area.y + card_area.height.saturating_sub(1),
            width: card_area.width.saturating_sub(4),
            height: 1
        };

        // Add cursor and handle horizontal scrolling if editing this field
        let url_text = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            render_scrollable_field(&app.edit_buffer[2], app.edit_cursor_pos, url_area.width as usize, 1)
        } else {
            format!(" {} ", app.edit_buffer[2].clone())
        };

        let url_line = Line::styled(url_text, style);
        let url_para = Paragraph::new(url_line).alignment(Alignment::Left);
        f.render_widget(url_para, url_area);
    }

    // Percentage on bottom-right border (render after URL to ensure visibility)
    if app.edit_buffer.len() >= 4 {
        let is_selected = app.edit_field_index == 3;
        let is_placeholder = app.edit_buffer_is_placeholder.get(3).copied().unwrap_or(false);

        let style = get_field_style(app, is_selected, is_placeholder);

        let mut pct_text = format!(" {} % ", app.edit_buffer[3].clone());

        // Add cursor if editing this field
        if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            pct_text = add_cursor_to_text(&pct_text, app.edit_cursor_pos, 1);
        }

        let pct_line = Line::styled(pct_text, style);
        let pct_area = Rect {
            x: card_area.x + 2,
            y: card_area.y + card_area.height.saturating_sub(1),
            width: card_area.width.saturating_sub(4),
            height: 1
        };
        let pct_para = Paragraph::new(pct_line).alignment(Alignment::Right);
        f.render_widget(pct_para, pct_area);
    }

    // Context in the middle (always render with newlines)
    if app.edit_buffer.len() >= 2 {
        render_context_field(f, app, inner_area, 1);
    }
}

fn render_context_field(f: &mut Frame, app: &App, inner_area: Rect, field_index: usize) {
    let is_selected = app.edit_field_index == field_index;
    let is_placeholder = app.edit_buffer_is_placeholder.get(field_index).copied().unwrap_or(false);

    let style = get_field_style(app, is_selected, is_placeholder);

    let field = &app.edit_buffer[field_index];

    // Render newlines when:
    // - NOT selected (user is editing other fields) → always render
    // - Selected AND in View Edit mode → render
    // - Selected AND in Field selection mode (not editing within field) → render
    // Show raw \n only when:
    // - Selected AND in Normal/Insert mode (editing within field, not View Edit)
    let should_render_newlines = !is_selected || app.view_edit_mode || !app.edit_field_editing_mode;

    if should_render_newlines {
        // View Edit mode: render with actual newlines
        let field_lines: Vec<&str> = field.lines().collect();
        let visible_height = inner_area.height as usize;
        let vscroll = app.edit_vscroll as usize;

        // Calculate cursor position if editing THIS field
        let (cursor_line, cursor_col) = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            let mut char_count = 0;
            let mut cursor_line_idx = 0;
            let mut cursor_col_in_line = 0;

            for (line_idx, line) in field_lines.iter().enumerate() {
                let line_len = line.chars().count();
                let separator_len = if line_idx < field_lines.len() - 1 { 1 } else { 0 }; // newline = 1 char

                if app.edit_cursor_pos <= char_count + line_len {
                    cursor_line_idx = line_idx;
                    cursor_col_in_line = app.edit_cursor_pos - char_count;
                    break;
                }

                char_count += line_len + separator_len;
            }

            (cursor_line_idx, cursor_col_in_line)
        } else {
            (0, 0)
        };

        // Render visible lines
        let visible_lines: Vec<&str> = field_lines
            .iter()
            .skip(vscroll)
            .take(visible_height)
            .copied()
            .collect();

        let mut content_lines: Vec<Line> = Vec::new();

        for (visible_idx, line_text) in visible_lines.iter().enumerate() {
            let actual_line_idx = vscroll + visible_idx;
            let mut display_line = line_text.to_string();

            // Add cursor if this is the line with the cursor AND this field is selected
            if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) && actual_line_idx == cursor_line {
                let char_count = display_line.chars().count();
                let cursor_char_pos = cursor_col.min(char_count);
                let byte_pos = if cursor_char_pos == 0 {
                    0
                } else if cursor_char_pos >= char_count {
                    display_line.len()
                } else {
                    display_line.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(display_line.len())
                };
                display_line.insert(byte_pos, '|');
            }

            content_lines.push(Line::styled(display_line, style));
        }

        // Pad with empty lines if needed
        for _ in content_lines.len()..visible_height {
            content_lines.push(Line::styled(String::new(), style));
        }

        let context_para = Paragraph::new(content_lines).wrap(Wrap { trim: false });
        f.render_widget(context_para, inner_area);
    } else {
        // Normal/Insert mode: show raw \n (replace with \\n for display)
        let mut display_text = field.replace('\n', "\\n");

        // Add cursor if editing this field
        if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            // Calculate cursor position in display_text
            // Each actual '\n' becomes "\\n" (2 chars), so we need to adjust the position
            let mut actual_pos = 0;
            let mut display_pos = 0;
            for ch in field.chars() {
                if actual_pos == app.edit_cursor_pos {
                    break;
                }
                if ch == '\n' {
                    display_pos += 2; // '\n' becomes "\\n" (2 characters)
                } else {
                    display_pos += 1;
                }
                actual_pos += 1;
            }

            // Insert cursor at the correct display position
            let byte_pos = if display_pos == 0 {
                0
            } else if display_pos >= display_text.chars().count() {
                display_text.len()
            } else {
                display_text.char_indices().nth(display_pos).map(|(i, _)| i).unwrap_or(display_text.len())
            };
            display_text.insert(byte_pos, '|');
        }

        // Render as single wrapped text
        let content_para = Paragraph::new(display_text)
            .style(style)
            .wrap(Wrap { trim: false });
        f.render_widget(content_para, inner_area);
    }
}

fn get_field_style(app: &App, is_selected: bool, is_placeholder: bool) -> Style {
    if is_selected {
        // Insert mode or View Edit mode: active color (yellow)
        // Field selection mode and Normal mode: selected color (blue)
        if app.edit_insert_mode || app.view_edit_mode {
            Style::default().fg(app.colorscheme.overlay_field_active).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.colorscheme.overlay_field_selected).add_modifier(Modifier::BOLD)
        }
    } else if is_placeholder {
        Style::default().fg(app.colorscheme.overlay_field_placeholder)
    } else {
        Style::default().fg(app.colorscheme.overlay_field_normal)
    }
}

fn add_cursor_to_text(text: &str, cursor_pos: usize, offset: usize) -> String {
    let mut result = text.to_string();
    let adjusted_pos = cursor_pos + offset;
    let char_count = result.chars().count();
    let cursor_char_pos = adjusted_pos.min(char_count);

    let byte_pos = if cursor_char_pos == 0 {
        0
    } else if cursor_char_pos >= char_count {
        result.len()
    } else {
        result.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(result.len())
    };

    result.insert(byte_pos, '|');
    result
}

// Render a field with horizontal scrolling to keep cursor visible
fn render_scrollable_field(field_content: &str, cursor_pos: usize, width: usize, padding: usize) -> String {
    // Account for leading/trailing spaces and cursor character
    let available_width = width.saturating_sub(padding * 2);

    if available_width == 0 {
        return format!(" {} ", field_content);
    }

    let field_chars: Vec<char> = field_content.chars().collect();
    let field_len = field_chars.len();

    // Calculate scroll offset to keep cursor visible
    let cursor_pos = cursor_pos.min(field_len);

    // Reserve space for cursor (1 char) with extra margin
    // Subtract 2 to ensure cursor is always visible even at the end
    let content_width = available_width.saturating_sub(2);

    // Calculate the scroll offset to keep cursor in view
    let scroll_offset = if cursor_pos < content_width {
        // Cursor is near the start, no scroll needed
        0
    } else {
        // Scroll so cursor is visible with margin
        // This ensures we can see the cursor even at the very end
        cursor_pos.saturating_sub(content_width)
    };

    // Extract visible portion
    let visible_start = scroll_offset;
    let visible_end = (scroll_offset + content_width).min(field_len);
    let visible_text: String = field_chars[visible_start..visible_end].iter().collect();

    // Add cursor
    let cursor_in_visible = cursor_pos.saturating_sub(scroll_offset);
    let mut display_text = format!(" {} ", visible_text);

    // Insert cursor at correct position (offset by 1 for leading space)
    let cursor_byte_pos = if cursor_in_visible == 0 {
        1 // After leading space
    } else {
        let prefix: String = field_chars[visible_start..(visible_start + cursor_in_visible).min(field_len)].iter().collect();
        1 + prefix.len()
    };

    if cursor_byte_pos <= display_text.len() {
        display_text.insert(cursor_byte_pos, '|');
    }

    display_text
}
