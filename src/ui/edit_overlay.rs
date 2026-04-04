use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::wrap::layout_wrapped_text;

pub fn overlay_layout(area: Rect) -> (Rect, Rect, Rect) {
    let popup_width = area.width.min(80);
    let popup_height = ((area.height * 7) / 10).max(10).min(area.height - 4);

    let x_centered = (area.width.saturating_sub(popup_width)) / 2;
    let x_aligned = x_centered & !1;

    let popup_area = Rect {
        x: x_aligned,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let clear_area = Rect {
        x: x_aligned.saturating_sub(1),
        y: popup_area.y,
        width: popup_width
            .saturating_add(2)
            .min(area.width.saturating_sub(x_aligned.saturating_sub(1))),
        height: popup_height,
    };

    let inner_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width.saturating_sub(2),
        height: popup_area.height.saturating_sub(2),
    };

    (popup_area, clear_area, inner_area)
}

pub fn render_edit_overlay(f: &mut Frame, app: &App) {
    // Create a centered popup area
    let area = f.area();
    let (popup_area, clear_area, inner_area) = overlay_layout(area);

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

    // Render the popup border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(app.border_style.to_border_type())
        .style(Style::default().bg(app.colorscheme.background).fg(Color::White));

    f.render_widget(block.clone(), popup_area);

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

        let date_text = format!(" {} ", app.edit_buffer[0].clone());
        let date_line = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            build_inline_block_cursor_line(&date_text, app.edit_cursor_pos, 1, style)
        } else {
            Line::styled(date_text, style)
        };
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
        let name_line = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            render_scrollable_field_line(&app.edit_buffer[0], app.edit_cursor_pos, name_area.width as usize, 1, style)
        } else {
            Line::styled(format!(" {} ", app.edit_buffer[0].clone()), style)
        };

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
        let url_line = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            render_scrollable_field_line(&app.edit_buffer[2], app.edit_cursor_pos, url_area.width as usize, 1, style)
        } else {
            Line::styled(format!(" {} ", app.edit_buffer[2].clone()), style)
        };

        let url_para = Paragraph::new(url_line).alignment(Alignment::Left);
        f.render_widget(url_para, url_area);
    }

    // Percentage on bottom-right border (render after URL to ensure visibility)
    if app.edit_buffer.len() >= 4 {
        let is_selected = app.edit_field_index == 3;
        let is_placeholder = app.edit_buffer_is_placeholder.get(3).copied().unwrap_or(false);

        let style = get_field_style(app, is_selected, is_placeholder);

        // Only show % when not a placeholder
        let pct_text = if is_placeholder {
            format!(" {} ", app.edit_buffer[3].clone())
        } else {
            format!(" {} % ", app.edit_buffer[3].clone())
        };

        let pct_line = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            build_inline_block_cursor_line(&pct_text, app.edit_cursor_pos, 1, style)
        } else {
            Line::styled(pct_text, style)
        };
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
        let cursor_pos = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            app.edit_cursor_pos
        } else {
            0
        };
        let layout = layout_wrapped_text(field, cursor_pos, inner_area.width as usize);
        let visible_height = inner_area.height as usize;
        let vscroll = app.edit_vscroll as usize;

        let visible_lines = layout
            .rows
            .iter()
            .skip(vscroll)
            .take(visible_height)
            .collect::<Vec<_>>();

        let mut content_lines: Vec<Line> = Vec::new();

        for (visible_idx, row) in visible_lines.iter().enumerate() {
            let actual_row_idx = vscroll + visible_idx;
            let display_line = row.text.clone();

            if is_selected
                && (app.edit_insert_mode || app.edit_field_editing_mode)
                && actual_row_idx == layout.cursor.visual_row
            {
                content_lines.push(build_context_line_with_cursor(
                    &display_line,
                    style,
                    layout.cursor.row_char_offset,
                ));
            } else {
                content_lines.push(Line::styled(display_line, style));
            }
        }

        // Pad with empty lines if needed
        for _ in content_lines.len()..visible_height {
            content_lines.push(Line::styled(String::new(), style));
        }

        let context_para = Paragraph::new(content_lines);
        f.render_widget(context_para, inner_area);
    } else {
        let mut display_text = String::new();
        let mut display_cursor_pos = 0usize;
        let actual_cursor = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
            Some(app.edit_cursor_pos)
        } else {
            None
        };

        for (idx, ch) in field.chars().enumerate() {
            if actual_cursor == Some(idx) {
                display_cursor_pos = display_text.chars().count();
            }
            if ch == '\n' {
                display_text.push('\\');
                display_text.push('n');
            } else {
                display_text.push(ch);
            }
        }
        if actual_cursor == Some(field.chars().count()) {
            display_cursor_pos = display_text.chars().count();
        }

        let layout = layout_wrapped_text(&display_text, display_cursor_pos, inner_area.width as usize);
        let visible_height = inner_area.height as usize;
        let vscroll = app.edit_vscroll as usize;
        let visible_rows = layout
            .rows
            .iter()
            .skip(vscroll)
            .take(visible_height)
            .collect::<Vec<_>>();

        let mut content_lines: Vec<Line> = Vec::new();
        for (visible_idx, row) in visible_rows.iter().enumerate() {
            let actual_row_idx = vscroll + visible_idx;
            let display_line = row.text.clone();

            if actual_cursor.is_some() && actual_row_idx == layout.cursor.visual_row {
                content_lines.push(build_context_line_with_cursor(
                    &display_line,
                    style,
                    layout.cursor.row_char_offset,
                ));
            } else {
                content_lines.push(Line::styled(display_line, style));
            }
        }

        for _ in content_lines.len()..visible_height {
            content_lines.push(Line::styled(String::new(), style));
        }

        let content_para = Paragraph::new(content_lines).style(style);
        f.render_widget(content_para, inner_area);
    }
}

fn get_field_style(app: &App, is_selected: bool, is_placeholder: bool) -> Style {
    if is_selected {
        // Insert mode: active color (yellow)
        // Normal mode (including View Edit mode): selected color (blue)
        if app.edit_insert_mode {
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

fn build_context_line_with_cursor(text: &str, base_style: Style, cursor_char_pos: usize) -> Line<'static> {
    let chars: Vec<char> = text.chars().collect();
    let char_count = chars.len();
    let cursor_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(110, 170, 255))
        .add_modifier(Modifier::BOLD);

    if cursor_char_pos >= char_count {
        let mut spans = Vec::with_capacity(2);
        if !text.is_empty() {
            spans.push(Span::styled(text.to_string(), base_style));
        }
        spans.push(Span::styled(" ".to_string(), cursor_style));
        return Line::from(spans);
    }

    let before: String = chars[..cursor_char_pos].iter().collect();
    let at_cursor = chars[cursor_char_pos].to_string();
    let after: String = chars[cursor_char_pos + 1..].iter().collect();

    let mut spans = Vec::with_capacity(3);
    if !before.is_empty() {
        spans.push(Span::styled(before, base_style));
    }
    spans.push(Span::styled(at_cursor, cursor_style));
    if !after.is_empty() {
        spans.push(Span::styled(after, base_style));
    }
    Line::from(spans)
}

fn build_inline_block_cursor_line(text: &str, cursor_pos: usize, offset: usize, base_style: Style) -> Line<'static> {
    let adjusted_pos = cursor_pos + offset;
    build_context_line_with_cursor(text, base_style, adjusted_pos)
}

// Render a field with horizontal scrolling to keep cursor visible
fn render_scrollable_field_line(
    field_content: &str,
    cursor_pos: usize,
    width: usize,
    padding: usize,
    base_style: Style,
) -> Line<'static> {
    // Account for leading/trailing spaces
    let available_width = width.saturating_sub(padding * 2);

    if available_width == 0 {
        return Line::styled(format!(" {} ", field_content), base_style);
    }

    let field_chars: Vec<char> = field_content.chars().collect();
    let field_len = field_chars.len();

    // Calculate scroll offset to keep cursor visible
    let cursor_pos = cursor_pos.min(field_len);

    let content_width = available_width.max(1);

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
    let cursor_in_visible = cursor_pos.saturating_sub(scroll_offset);
    let display_text = format!(" {} ", visible_text);

    build_inline_block_cursor_line(&display_text, cursor_in_visible, 1, base_style)
}
