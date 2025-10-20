use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn handle_overlay_keyboard(app: &mut App, key: KeyEvent) {
    if app.edit_insert_mode {
        // Insert mode: typing edits current field
        match key.code {
            KeyCode::Esc | KeyCode::Char('[') if key.code == KeyCode::Esc || key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Exit insert mode
                app.edit_insert_mode = false;
                // Exit View Edit mode if active and reset scroll
                if app.view_edit_mode {
                    app.view_edit_mode = false;
                    app.edit_vscroll = 0; // Reset to first line
                }
                // If entered insert mode directly with 'i' or 'v', skip normal mode and go back to field selection
                if app.edit_skip_normal_mode {
                    app.edit_field_editing_mode = false;
                    app.edit_skip_normal_mode = false;
                    // Restore placeholder if field is empty (for :ai/:ao flow)
                    if app.edit_field_index < app.edit_buffer.len() {
                        let field = &app.edit_buffer[app.edit_field_index];
                        if field.is_empty() {
                            let placeholder = if app.edit_buffer.len() == 3 {
                                match app.edit_field_index {
                                    0 => "date",
                                    1 => "context",
                                    _ => "",
                                }
                            } else {
                                match app.edit_field_index {
                                    0 => "name",
                                    1 => "context",
                                    2 => "url",
                                    3 => "percentage",
                                    _ => "",
                                }
                            };
                            if !placeholder.is_empty() {
                                app.edit_buffer[app.edit_field_index] = placeholder.to_string();
                                if app.edit_field_index < app.edit_buffer_is_placeholder.len() {
                                    app.edit_buffer_is_placeholder[app.edit_field_index] = true;
                                }
                            }
                        }
                    }
                }
                // Otherwise stay in field editing mode (normal mode)
                // Keep field empty to reflect actual buffer content
            }
            KeyCode::Backspace => {
                if app.edit_field_index < app.edit_buffer.len() && app.edit_cursor_pos > 0 {
                    let field = &mut app.edit_buffer[app.edit_field_index];

                    // Delete single character (including newline character)
                    let char_indices: Vec<_> = field.char_indices().collect();
                    if app.edit_cursor_pos > 0 && app.edit_cursor_pos <= char_indices.len() {
                        let byte_pos = char_indices[app.edit_cursor_pos - 1].0;
                        field.remove(byte_pos);
                        app.edit_cursor_pos -= 1;
                    }
                }
                app.ensure_overlay_cursor_visible();
            }
            KeyCode::Left => {
                if app.view_edit_mode {
                    // In View Edit mode, move cursor like a normal text editor
                    if app.edit_cursor_pos > 0 && app.edit_field_index < app.edit_buffer.len() {
                        let field = &app.edit_buffer[app.edit_field_index];
                        let lines: Vec<&str> = field.split('\n').collect();

                        // Find current line and column
                        let mut char_count = 0;
                        let mut current_line = 0;
                        let mut col_in_line = 0;

                        for (line_idx, line) in lines.iter().enumerate() {
                            let line_len = line.chars().count();
                            let separator_len = if line_idx < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char

                            if app.edit_cursor_pos <= char_count + line_len {
                                current_line = line_idx;
                                col_in_line = app.edit_cursor_pos - char_count;
                                break;
                            }

                            char_count += line_len + separator_len;
                        }

                        if col_in_line > 0 {
                            // Move left within current line
                            app.edit_cursor_pos -= 1;
                        } else if current_line > 0 {
                            // Move to end of previous line
                            let mut new_pos = 0;
                            for (i, _line) in lines.iter().enumerate().take(current_line - 1) {
                                let line_len = lines[i].chars().count();
                                let separator_len = if i < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char
                                new_pos += line_len + separator_len;
                            }
                            new_pos += lines[current_line - 1].chars().count();
                            app.edit_cursor_pos = new_pos;
                        }
                    }
                } else if app.edit_cursor_pos > 0 {
                    app.edit_cursor_pos -= 1;
                }
                app.ensure_overlay_cursor_visible();
            }
            KeyCode::Right => {
                if app.edit_field_index < app.edit_buffer.len() {
                    let field = &app.edit_buffer[app.edit_field_index];
                    let field_len = field.chars().count();

                    if app.edit_cursor_pos < field_len {
                        if app.view_edit_mode {
                            // In View Edit mode, move cursor like a normal text editor
                            let lines: Vec<&str> = field.split('\n').collect();

                            // Find current line and column
                            let mut char_count = 0;
                            let mut current_line = 0;
                            let mut col_in_line = 0;

                            for (line_idx, line) in lines.iter().enumerate() {
                                let line_len = line.chars().count();
                                let separator_len = if line_idx < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char

                                if app.edit_cursor_pos <= char_count + line_len {
                                    current_line = line_idx;
                                    col_in_line = app.edit_cursor_pos - char_count;
                                    break;
                                }

                                char_count += line_len + separator_len;
                            }

                            let current_line_len = lines[current_line].chars().count();

                            if col_in_line < current_line_len {
                                // Move right within current line
                                app.edit_cursor_pos += 1;
                            } else if current_line + 1 < lines.len() {
                                // Move to start of next line (skip over newline)
                                app.edit_cursor_pos += 1; // Skip newline (1 character)
                            }
                        } else {
                            app.edit_cursor_pos += 1;
                        }
                    }
                }
                app.ensure_overlay_cursor_visible();
            }
            KeyCode::Enter => {
                // In View Edit mode, insert newline character
                if app.view_edit_mode && app.edit_field_index < app.edit_buffer.len() {
                    let field = &mut app.edit_buffer[app.edit_field_index];
                    // Find byte index for character position
                    let byte_pos = if app.edit_cursor_pos == 0 {
                        0
                    } else if app.edit_cursor_pos >= field.chars().count() {
                        field.len()
                    } else {
                        field.char_indices().nth(app.edit_cursor_pos).map(|(i, _)| i).unwrap_or(field.len())
                    };
                    // Insert actual newline character
                    field.insert(byte_pos, '\n');
                    app.edit_cursor_pos += 1; // Move cursor past newline (1 character)
                }
            }
            KeyCode::Up => {
                // In View Edit mode, move up one line
                if app.view_edit_mode && app.edit_field_index < app.edit_buffer.len() {
                    let field = &app.edit_buffer[app.edit_field_index];
                    let lines: Vec<&str> = field.split('\n').collect();

                    // Find current line and column
                    let mut current_pos = 0;
                    let mut current_line = 0;
                    let mut col_in_line = 0;

                    for (line_idx, line) in lines.iter().enumerate() {
                        let line_len = line.chars().count();
                        let separator_len = if line_idx < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char

                        if app.edit_cursor_pos <= current_pos + line_len {
                            current_line = line_idx;
                            col_in_line = app.edit_cursor_pos - current_pos;
                            break;
                        }

                        current_pos += line_len + separator_len;
                    }

                    // Move to previous line if possible
                    if current_line > 0 {
                        let prev_line = lines[current_line - 1];
                        let prev_line_len = prev_line.chars().count();

                        // Calculate position in previous line
                        let mut new_pos = 0;
                        for (i, _line) in lines.iter().enumerate().take(current_line - 1) {
                            let line_len = lines[i].chars().count();
                            let separator_len = if i < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char
                            new_pos += line_len + separator_len;
                        }

                        // Keep same column or go to end of line
                        new_pos += col_in_line.min(prev_line_len);
                        app.edit_cursor_pos = new_pos;
                    }
                }
                app.ensure_overlay_cursor_visible();
            }
            KeyCode::Down => {
                // In View Edit mode, move down one line
                if app.view_edit_mode && app.edit_field_index < app.edit_buffer.len() {
                    let field = &app.edit_buffer[app.edit_field_index];
                    let lines: Vec<&str> = field.split('\n').collect();

                    // Find current line and column
                    let mut current_pos = 0;
                    let mut current_line = 0;
                    let mut col_in_line = 0;

                    for (line_idx, line) in lines.iter().enumerate() {
                        let line_len = line.chars().count();
                        let separator_len = if line_idx < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char

                        if app.edit_cursor_pos <= current_pos + line_len {
                            current_line = line_idx;
                            col_in_line = app.edit_cursor_pos - current_pos;
                            break;
                        }

                        current_pos += line_len + separator_len;
                    }

                    // Move to next line if possible
                    if current_line + 1 < lines.len() {
                        let next_line = lines[current_line + 1];
                        let next_line_len = next_line.chars().count();

                        // Calculate position in next line
                        let mut new_pos = 0;
                        for (i, _line) in lines.iter().enumerate().take(current_line + 1) {
                            let line_len = lines[i].chars().count();
                            let separator_len = if i < lines.len() - 1 { 1 } else { 0 }; // newline = 1 char
                            new_pos += line_len + separator_len;
                        }

                        // Keep same column or go to end of line
                        new_pos += col_in_line.min(next_line_len);
                        app.edit_cursor_pos = new_pos;
                    }
                }
                app.ensure_overlay_cursor_visible();
            }
            KeyCode::Char(c) => {
                if app.edit_field_index < app.edit_buffer.len() {
                    let field = &mut app.edit_buffer[app.edit_field_index];
                    // Find byte index for character position
                    let byte_pos = if app.edit_cursor_pos == 0 {
                        0
                    } else if app.edit_cursor_pos >= field.chars().count() {
                        field.len()
                    } else {
                        field.char_indices().nth(app.edit_cursor_pos).map(|(i, _)| i).unwrap_or(field.len())
                    };
                    field.insert(byte_pos, c);
                    app.edit_cursor_pos += 1;
                }
                app.ensure_overlay_cursor_visible();
            }
            _ => {}
        }
    } else if app.edit_field_editing_mode {
        // Field editing normal mode: cursor navigation within field
        handle_field_editing_mode(app, key);
    } else {
        // Field selection mode: navigate between fields
        handle_field_selection_mode(app, key);
    }
}

fn handle_field_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('[') if key.code == KeyCode::Esc || key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Exit field editing mode, go back to field selection
            app.edit_field_editing_mode = false;
            app.edit_cursor_pos = 0;
            app.edit_hscroll = 0;
            // Exit View Edit mode if active and reset scroll
            if app.view_edit_mode {
                app.view_edit_mode = false;
                app.edit_vscroll = 0; // Reset to first line
            }
            // Restore placeholder if field is empty
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                if field.is_empty() {
                    // Determine placeholder based on edit_buffer length
                    let placeholder = if app.edit_buffer.len() == 3 {
                        // INSIDE entry: date, context, Exit
                        match app.edit_field_index {
                            0 => "date",
                            1 => "context",
                            _ => "",
                        }
                    } else {
                        // OUTSIDE entry: name, context, url, percentage, Exit
                        match app.edit_field_index {
                            0 => "name",
                            1 => "context",
                            2 => "url",
                            3 => "percentage",
                            _ => "",
                        }
                    };
                    if !placeholder.is_empty() {
                        app.edit_buffer[app.edit_field_index] = placeholder.to_string();
                        if app.edit_field_index < app.edit_buffer_is_placeholder.len() {
                            app.edit_buffer_is_placeholder[app.edit_field_index] = true;
                        }
                    }
                }
            }
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if app.edit_cursor_pos > 0 {
                app.edit_cursor_pos -= 1;
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.edit_field_index < app.edit_buffer.len() {
                let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                if app.edit_cursor_pos < field_len {
                    app.edit_cursor_pos += 1;
                }
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('0') => {
            app.edit_cursor_pos = 0;
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('$') => {
            if app.edit_field_index < app.edit_buffer.len() {
                let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                app.edit_cursor_pos = field_len;
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('w') => {
            // Move to next word (simplified: skip to next space)
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                let chars: Vec<char> = field.chars().collect();
                let mut pos = app.edit_cursor_pos;
                // Skip current word
                while pos < chars.len() && !chars[pos].is_whitespace() {
                    pos += 1;
                }
                // Skip whitespace
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                app.edit_cursor_pos = pos;
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('b') => {
            // Move to previous word
            if app.edit_cursor_pos > 0 {
                let field = &app.edit_buffer[app.edit_field_index];
                let chars: Vec<char> = field.chars().collect();
                let mut pos = app.edit_cursor_pos.saturating_sub(1);
                // Skip whitespace
                while pos > 0 && chars[pos].is_whitespace() {
                    pos -= 1;
                }
                // Skip to start of word
                while pos > 0 && !chars[pos - 1].is_whitespace() {
                    pos -= 1;
                }
                app.edit_cursor_pos = pos;
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('e') => {
            // Move to end of current or next word
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                let chars: Vec<char> = field.chars().collect();
                if !chars.is_empty() && app.edit_cursor_pos < chars.len() {
                    let mut pos = app.edit_cursor_pos;

                    // Skip whitespace if we're on it
                    while pos < chars.len() && chars[pos].is_whitespace() {
                        pos += 1;
                    }

                    // Move to end of current word
                    while pos < chars.len() && !chars[pos].is_whitespace() {
                        pos += 1;
                    }

                    // Position on last character of word (not the space after)
                    if pos > 0 {
                        app.edit_cursor_pos = pos - 1;
                    }
                }
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('g') => {
            // Handle gg (go to start)
            app.edit_cursor_pos = 0;
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('G') => {
            // Go to end
            if app.edit_field_index < app.edit_buffer.len() {
                let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                app.edit_cursor_pos = field_len;
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('x') => {
            // Delete character at cursor
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &mut app.edit_buffer[app.edit_field_index];
                let mut chars: Vec<char> = field.chars().collect();
                if app.edit_cursor_pos < chars.len() {
                    chars.remove(app.edit_cursor_pos);
                    *field = chars.into_iter().collect();
                    // Mark as no longer a placeholder if it was
                    if app.edit_field_index < app.edit_buffer_is_placeholder.len() {
                        app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                    }
                }
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('X') => {
            // Delete character before cursor
            if app.edit_field_index < app.edit_buffer.len() && app.edit_cursor_pos > 0 {
                let field = &mut app.edit_buffer[app.edit_field_index];
                let mut chars: Vec<char> = field.chars().collect();
                if app.edit_cursor_pos > 0 && app.edit_cursor_pos <= chars.len() {
                    chars.remove(app.edit_cursor_pos - 1);
                    *field = chars.into_iter().collect();
                    app.edit_cursor_pos -= 1;
                    // Mark as no longer a placeholder if it was
                    if app.edit_field_index < app.edit_buffer_is_placeholder.len() {
                        app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                    }
                }
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('i') => {
            // Enter insert mode (from normal mode within field)
            app.edit_insert_mode = true;
            // edit_skip_normal_mode stays false because we're already in normal mode
            // Clear placeholder text when entering insert mode
            if app.edit_field_index < app.edit_buffer_is_placeholder.len()
                && app.edit_buffer_is_placeholder[app.edit_field_index] {
                app.edit_buffer[app.edit_field_index] = String::new();
                app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                app.edit_cursor_pos = 0;
            }
        }
        _ => {}
    }
}

fn handle_field_selection_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.cancel_editing_entry();
        }
        KeyCode::Char('w') => {
            app.save_edited_entry();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.edit_field_index > 0 {
                app.edit_field_index -= 1;
                app.edit_cursor_pos = 0;
                app.edit_hscroll = 0;
                app.edit_vscroll = 0;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.edit_field_index + 1 < app.edit_buffer.len() {
                app.edit_field_index += 1;
                app.edit_cursor_pos = 0;
                app.edit_hscroll = 0;
                app.edit_vscroll = 0;
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Check if this is context field (index 1)
            let is_context_field = (app.edit_buffer.len() == 3 && app.edit_field_index == 1) ||
                                   (app.edit_buffer.len() == 5 && app.edit_field_index == 1);

            if is_context_field {
                // Vertical scroll up for context field
                app.edit_vscroll = app.edit_vscroll.saturating_sub(1);
            } else {
                // Horizontal scroll left for other fields
                app.edit_hscroll = app.edit_hscroll.saturating_sub(4);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            // Check if this is context field (index 1)
            let is_context_field = (app.edit_buffer.len() == 3 && app.edit_field_index == 1) ||
                                   (app.edit_buffer.len() == 5 && app.edit_field_index == 1);

            if is_context_field {
                // Vertical scroll down for context field
                if app.edit_field_index < app.edit_buffer.len() {
                    let field = &app.edit_buffer[app.edit_field_index];
                    let lines: Vec<&str> = field.split('\n').collect();
                    // Fixed window size for field selection mode (minimum 5 lines)
                    let window_height = 5;
                    let total_lines = lines.len();
                    // Calculate max scroll: lines - window_height (but at least 0)
                    let max_scroll = total_lines.saturating_sub(window_height);
                    // Only scroll if we haven't reached the limit
                    if (app.edit_vscroll as usize) < max_scroll {
                        app.edit_vscroll += 1;
                    }
                }
            } else {
                // Horizontal scroll right for other fields
                if app.edit_field_index < app.edit_buffer.len() {
                    let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                    // Allow scrolling up to field length
                    if (app.edit_hscroll as usize) < field_len {
                        app.edit_hscroll += 4;
                    }
                }
            }
        }
        KeyCode::Enter => {
            // Check if Exit field is selected
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                if field == "Exit" {
                    // Close overlay without saving
                    app.cancel_editing_entry();
                    return;
                }
                // Clear placeholder text when entering field editing mode
                if app.edit_field_index < app.edit_buffer_is_placeholder.len()
                    && app.edit_buffer_is_placeholder[app.edit_field_index] {
                    app.edit_buffer[app.edit_field_index] = String::new();
                    app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                }
            }
            // Enter field editing mode
            app.edit_field_editing_mode = true;
            app.edit_cursor_pos = 0;
            app.edit_hscroll = 0;
        }
        KeyCode::Char('i') => {
            // Skip field editing mode, go straight to insert mode with cursor at end
            app.edit_field_editing_mode = true;
            app.edit_insert_mode = true;
            app.edit_skip_normal_mode = true; // Mark that we skipped normal mode
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                // Clear placeholder text when entering insert mode
                if app.edit_field_index < app.edit_buffer_is_placeholder.len()
                    && app.edit_buffer_is_placeholder[app.edit_field_index] {
                    app.edit_buffer[app.edit_field_index] = String::new();
                    app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                    app.edit_cursor_pos = 0;
                } else {
                    // Move cursor to end of text
                    app.edit_cursor_pos = field.chars().count();
                }
            }
            app.ensure_overlay_cursor_visible();
        }
        KeyCode::Char('v') => {
            // Enter View Edit mode: render \n as newlines
            // ONLY allow View Edit mode for context field (index 1)
            if app.edit_field_index != 1 {
                // Not on context field, ignore 'v' key
                return;
            }

            // Check if Exit field is selected
            if app.edit_field_index < app.edit_buffer.len() {
                let field = &app.edit_buffer[app.edit_field_index];
                if field == "Exit" {
                    // Don't enter View Edit mode on Exit field
                    return;
                }
                // Clear placeholder text when entering View Edit mode
                if app.edit_field_index < app.edit_buffer_is_placeholder.len()
                    && app.edit_buffer_is_placeholder[app.edit_field_index] {
                    app.edit_buffer[app.edit_field_index] = String::new();
                    app.edit_buffer_is_placeholder[app.edit_field_index] = false;
                    app.edit_cursor_pos = 0;
                } else {
                    // Move cursor to start of field
                    app.edit_cursor_pos = 0;
                }
            }
            // Enter View Edit mode directly in insert mode (skip normal mode)
            app.view_edit_mode = true;
            app.edit_field_editing_mode = true;
            app.edit_insert_mode = true;
            app.edit_skip_normal_mode = true;
            // Ensure cursor is visible in the window
            app.ensure_overlay_cursor_visible();
        }
        _ => {}
    }
}
