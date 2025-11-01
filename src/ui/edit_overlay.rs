use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::app::App;

pub fn render_edit_overlay(f: &mut Frame, app: &App) {
    // Create a centered popup area
    let area = f.area();

    let popup_width = area.width.min(80);
    // Increase height to show more of the background: use 70% of screen height or calculated size
    let calculated_height = app.edit_buffer.len() as u16 + 4;
    let max_height = (area.height * 7) / 10; // 70% of screen height
    let popup_height = calculated_height.max(max_height.min(area.height - 4));

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
    // This ensures complete coverage, especially for wide characters
    let blank_lines: Vec<Line> = (0..clear_area.height)
        .map(|_| Line::from(" ".repeat(clear_area.width as usize)))
        .collect();
    let blank_paragraph = Paragraph::new(blank_lines)
        .style(Style::default().bg(app.colorscheme.background));
    f.render_widget(blank_paragraph, clear_area);

    // Determine if editing INSIDE or OUTSIDE entry
    // INSIDE: date, context, Exit (3 fields)
    // OUTSIDE: name, context, url, percentage, Exit (5 fields)
    let title = if app.edit_buffer.len() == 3 {
        " INSIDE "
    } else {
        " OUTSIDE "
    };

    // Render the popup as a single card with rounded borders on top
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(app.colorscheme.background).fg(Color::White));

    f.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);

    // Render each field with proper windowing and scrolling
    let mut lines = Vec::new();
    let window_width = inner_area.width as usize;

    for (i, field) in app.edit_buffer.iter().enumerate() {
        let is_selected = i == app.edit_field_index;

        // Check if this is a placeholder using the placeholder flag
        let is_placeholder = i < app.edit_buffer_is_placeholder.len()
                           && app.edit_buffer_is_placeholder[i];

        let style = if is_selected {
            // View Edit mode or Insert mode: active color (both are editing modes)
            // Normal mode: selected color
            if app.edit_insert_mode || app.view_edit_mode {
                Style::default().fg(app.colorscheme.overlay_field_active).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.colorscheme.overlay_field_selected).add_modifier(Modifier::BOLD)
            }
        } else if is_placeholder {
            // Show placeholders in dim color
            Style::default().fg(app.colorscheme.overlay_field_placeholder)
        } else {
            Style::default().fg(app.colorscheme.overlay_field_normal)
        };

        // Check if this is context field (index 1 in both INSIDE and OUTSIDE)
        let is_context_field = (app.edit_buffer.len() == 3 && i == 1) || // INSIDE context
                               (app.edit_buffer.len() == 5 && i == 1);   // OUTSIDE context

        // Render newlines for context field:
        // - Field selection mode (not editing): render \n as newlines (multi-line)
        // - View Edit mode: render \n as newlines (multi-line)
        // - Normal/Insert mode: show raw \n (single-line with wrapping)
        // - Other fields: never render \n as newlines
        let should_render_newlines = is_context_field && (!app.edit_field_editing_mode || app.view_edit_mode);

        if is_context_field && should_render_newlines {
            // Context field with newlines: dynamic window with scrolling
            // Context already contains actual newline characters
            let field_lines: Vec<&str> = field.lines().collect();

            // Context field window size: calculate based on available space
            let min_window_height = 1;

            // Calculate available height: total inner area minus other fields
            let num_other_fields = app.edit_buffer.len() - 1; // All fields except context
            let other_fields_height = num_other_fields * 2; // Each field + blank line
            let available_height = inner_area.height as usize;
            let max_window_height = if available_height > other_fields_height {
                (available_height - other_fields_height).max(min_window_height)
            } else {
                min_window_height
            };

            // Calculate actual display lines considering text wrapping with proper Unicode width
            let actual_display_lines: usize = field_lines.iter()
                .map(|line| {
                    let display_width = line.width(); // Accurate Unicode display width
                    if display_width == 0 {
                        1 // Empty lines still take 1 line
                    } else {
                        // Calculate how many lines this will take when wrapped
                        ((display_width + window_width - 1) / window_width).max(1)
                    }
                })
                .sum();

            // Determine window height based on actual display lines (with wrapping)
            let window_height = actual_display_lines.max(min_window_height).min(max_window_height);

            let vscroll = app.edit_vscroll as usize;

            // Apply vertical scroll
            let visible_lines: Vec<&str> = field_lines
                .iter()
                .skip(vscroll)
                .take(window_height)
                .copied()
                .collect();

            // Calculate cursor position if editing
            let (cursor_line, cursor_col) = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
                // Calculate which line and column the cursor is on
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

            // Render each visible line (no horizontal scrolling for context field)
            for (visible_idx, line_text) in visible_lines.iter().enumerate() {
                let actual_line_idx = vscroll + visible_idx;
                let mut display_line = line_text.to_string();

                // Add cursor if this is the line with the cursor
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

                // Context field doesn't use horizontal scrolling, just display as-is
                // Text will wrap naturally in the Paragraph widget
                lines.push(Line::styled(display_line, style));
            }

            // Pad with empty lines to reach window_height (for View Edit mode with few lines)
            for _ in visible_lines.len()..window_height {
                lines.push(Line::styled(String::new(), style));
            }
        } else if is_context_field {
            // Context field in Normal/Insert mode: show raw \n with wrapping
            // Replace actual newline characters with visible "\\n" text
            let mut display_text = field.replace('\n', "\\n");

            // Add cursor in insert mode or field editing mode
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

            // Calculate available height dynamically (similar to View Edit mode)
            let num_other_fields = app.edit_buffer.len() - 1; // All fields except context
            let other_fields_height = num_other_fields * 2; // Each field + blank line
            let available_height = inner_area.height as usize;
            let min_window_height = 1;
            let max_wrapped_lines = if available_height > other_fields_height {
                (available_height - other_fields_height).max(min_window_height)
            } else {
                min_window_height
            };

            // Split text into chunks that fit within window width (wrapping)
            let chars: Vec<char> = display_text.chars().collect();
            let mut line_start = 0;
            let mut wrapped_line_count = 0;

            while line_start < chars.len() && wrapped_line_count < max_wrapped_lines {
                let mut line_width = 0;
                let mut line_end = line_start;

                for (idx, ch) in chars[line_start..].iter().enumerate() {
                    let char_width = app.display_width_str(&ch.to_string());
                    if line_width + char_width > window_width && line_width > 0 {
                        break;
                    }
                    line_width += char_width;
                    line_end = line_start + idx + 1;
                }

                if line_end == line_start {
                    // Edge case: single character wider than window
                    line_end = line_start + 1;
                }

                let line_text: String = chars[line_start..line_end].iter().collect();
                lines.push(Line::styled(line_text, style));

                line_start = line_end;
                wrapped_line_count += 1;
            }

            // If empty, add at least one line
            if chars.is_empty() {
                lines.push(Line::styled(String::new(), style));
            }
        } else {
            // Single-line field: apply horizontal scrolling
            let mut display_text = field.clone();

            // Add cursor in insert mode or field editing mode
            if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
                let char_count = field.chars().count();
                let cursor_char_pos = app.edit_cursor_pos.min(char_count);
                let byte_pos = if cursor_char_pos == 0 {
                    0
                } else if cursor_char_pos >= char_count {
                    field.len()
                } else {
                    field.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(field.len())
                };
                display_text.insert(byte_pos, '|');
            }

            // Apply horizontal scroll if this is the selected field and in editing mode
            let scrolled_text = if is_selected && app.edit_field_editing_mode {
                let hscroll = app.edit_hscroll as usize;
                // Use display width slicing
                app.slice_columns(&display_text, hscroll, window_width)
            } else {
                // No scrolling for non-editing fields, just truncate if too long
                if app.display_width_str(&display_text) > window_width {
                    app.slice_columns(&display_text, 0, window_width)
                } else {
                    display_text
                }
            };

            lines.push(Line::styled(scrolled_text, style));
        }

        // Add blank line between fields
        if i < app.edit_buffer.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(content, inner_area);
}
