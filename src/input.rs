use anyhow::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crate::app::{App, FormatMode, InputMode, ScrollbarType};

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    mut app: App,
) -> Result<()> {
    // Setup file watcher
    let (tx, rx): (std::sync::mpsc::Sender<NotifyEvent>, Receiver<NotifyEvent>) = mpsc::channel();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<NotifyEvent, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

    // Watch the file if it exists
    if let Some(ref path) = app.file_path {
        let _ = watcher.watch(path, RecursiveMode::NonRecursive);
    }

    loop {
        terminal.draw(|f| crate::ui::ui(f, &mut app))?;
        app.update_status();

        // Check for file changes
        if app.auto_reload && app.file_path.is_some() {
            match rx.try_recv() {
                Ok(event) => {
                    // Check if it's a modify event
                    if matches!(event.kind, notify::EventKind::Modify(_)) {
                        // Ignore file changes within 1 second after saving (to avoid reloading our own save)
                        let should_reload = if let Some(last_save) = app.last_save_time {
                            last_save.elapsed() > Duration::from_millis(1000)
                        } else {
                            true
                        };

                        // Only reload if not modified by user and not recently saved
                        if !app.is_modified && should_reload {
                            app.reload_file();
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {}
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Filter out key repeat events on Windows to prevent duplicate input
                    #[cfg(target_os = "windows")]
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                        return Ok(());
                    }
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('r') {
                        app.redo();
                        continue;
                    }

                    // Handle editing overlay input separately
                    if app.editing_entry {
                        if app.edit_insert_mode {
                            // Insert mode: typing edits current field
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('[') if key.code == KeyCode::Esc || key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Exit insert mode
                                    app.edit_insert_mode = false;
                                    // If entered insert mode directly with 'i', skip normal mode and go back to field selection
                                    if app.edit_skip_normal_mode {
                                        app.edit_field_editing_mode = false;
                                        app.edit_skip_normal_mode = false;
                                    }
                                    // Otherwise stay in field editing mode (normal mode)
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
                                KeyCode::Backspace => {
                                    if app.edit_field_index < app.edit_buffer.len() && app.edit_cursor_pos > 0 {
                                        let field = &mut app.edit_buffer[app.edit_field_index];
                                        // Find byte index for character position
                                        let char_indices: Vec<_> = field.char_indices().collect();
                                        if app.edit_cursor_pos > 0 && app.edit_cursor_pos <= char_indices.len() {
                                            let byte_pos = char_indices[app.edit_cursor_pos - 1].0;
                                            field.remove(byte_pos);
                                            app.edit_cursor_pos -= 1;
                                        }
                                    }
                                }
                                KeyCode::Left => {
                                    if app.edit_cursor_pos > 0 {
                                        app.edit_cursor_pos -= 1;
                                    }
                                }
                                KeyCode::Right => {
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                                        if app.edit_cursor_pos < field_len {
                                            app.edit_cursor_pos += 1;
                                        }
                                    }
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
                                }
                                _ => {}
                            }
                        } else if app.edit_field_editing_mode {
                            // Field editing normal mode: cursor navigation within field
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('[') if key.code == KeyCode::Esc || key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Exit field editing mode, go back to field selection
                                    app.edit_field_editing_mode = false;
                                    app.edit_cursor_pos = 0;
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
                                }
                                KeyCode::Char('l') | KeyCode::Right => {
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                                        if app.edit_cursor_pos < field_len {
                                            app.edit_cursor_pos += 1;
                                        }
                                    }
                                }
                                KeyCode::Char('0') => {
                                    app.edit_cursor_pos = 0;
                                }
                                KeyCode::Char('$') => {
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                                        app.edit_cursor_pos = field_len;
                                    }
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
                                }
                                KeyCode::Char('e') => {
                                    // Move to end of current or next word
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field = &app.edit_buffer[app.edit_field_index];
                                        let chars: Vec<char> = field.chars().collect();
                                        if chars.is_empty() {
                                            return Ok(());
                                        }
                                        let mut pos = app.edit_cursor_pos;

                                        // If we're at the end, don't move
                                        if pos >= chars.len() {
                                            return Ok(());
                                        }

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
                                KeyCode::Char('g') => {
                                    // Handle gg (go to start)
                                    app.edit_cursor_pos = 0;
                                }
                                KeyCode::Char('G') => {
                                    // Go to end
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field_len = app.edit_buffer[app.edit_field_index].chars().count();
                                        app.edit_cursor_pos = field_len;
                                    }
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
                        } else {
                            // Field selection mode: navigate between fields
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
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.edit_field_index + 1 < app.edit_buffer.len() {
                                        app.edit_field_index += 1;
                                        app.edit_cursor_pos = 0;
                                    }
                                }
                                KeyCode::Enter => {
                                    // Check if Exit field is selected
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field = &app.edit_buffer[app.edit_field_index];
                                        if field == "Exit" {
                                            // Close overlay without saving
                                            app.cancel_editing_entry();
                                            continue;
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
                                }
                                _ => {}
                            }
                        }
                        continue;
                    }

                    match app.input_mode {
                        InputMode::Normal => {
                            // Handle substitute confirmation if active
                            if !app.substitute_confirmations.is_empty() {
                                match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('n') | KeyCode::Char('a') | KeyCode::Char('q') => {
                                        if let KeyCode::Char(c) = key.code {
                                            app.handle_substitute_confirmation(c);
                                        }
                                        continue;
                                    }
                                    KeyCode::Esc => {
                                        app.handle_substitute_confirmation('q');
                                        continue;
                                    }
                                    _ => continue,
                                }
                            }

                            match key.code {
                            KeyCode::Char('u') => {
                                if !app.showing_help && app.format_mode == FormatMode::Edit {
                                    app.undo();
                                }
                            }
                            KeyCode::Char('?') => {
                                // Toggle help
                                app.toggle_help();
                            }
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('w') => {
                                // Vim-like: move to start of next word (Edit mode)
                                if !app.showing_help && app.format_mode == FormatMode::Edit {
                                    app.move_to_next_word_start();
                                }
                            }
                            KeyCode::Char('e') => {
                                // Vim-like: move to end of next word (Edit mode)
                                if !app.showing_help && app.format_mode == FormatMode::Edit {
                                    app.move_to_next_word_end();
                                }
                            }
                            KeyCode::Char('b') => {
                                // Vim-like: move to start of previous word (Edit mode)
                                if !app.showing_help && app.format_mode == FormatMode::Edit {
                                    app.move_to_previous_word_start();
                                }
                            }
                            KeyCode::Char('r') => {
                                if !app.showing_help {
                                    // Clear filter when toggling modes
                                    if !app.filter_pattern.is_empty() {
                                        app.filter_pattern.clear();
                                    }

                                    // Toggle between View and Edit only (not Help)
                                    app.format_mode = match app.format_mode {
                                        FormatMode::View => FormatMode::Edit,
                                        FormatMode::Edit => FormatMode::View,
                                        FormatMode::Help => FormatMode::View, // If somehow in Help, go to View
                                    };
                                    let mode_name = match app.format_mode {
                                        FormatMode::View => "View",
                                        FormatMode::Edit => "Edit",
                                        FormatMode::Help => "Help",
                                    };
                                    if app.format_mode == FormatMode::View {
                                        app.hscroll = 0;
                                    }
                                    app.convert_json();
                                    app.set_status(&format!("{} mode", mode_name));
                                }
                            }
                            KeyCode::Char('i') => {
                                if !app.showing_help && app.format_mode == FormatMode::Edit {
                                    app.input_mode = InputMode::Insert;
                                    app.ensure_cursor_visible();
                                    app.set_status("-- INSERT --");
                                }
                            }
                            KeyCode::Char(':') => {
                                // Allow command mode even when showing help (for :h to toggle)
                                app.input_mode = InputMode::Command;
                                app.command_buffer = String::new();
                                app.command_history_index = None;
                                app.set_status(":");
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.showing_help {
                                    // Allow scrolling in help mode (takes priority)
                                    app.scroll_up();
                                } else if app.format_mode == FormatMode::Edit {
                                    app.move_cursor_up();
                                } else if !app.relf_entries.is_empty() {
                                    // Move selection up in card view
                                    if app.selected_entry_index > 0 {
                                        app.selected_entry_index -= 1;
                                    }
                                } else {
                                    app.relf_jump_up();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.showing_help {
                                    // Allow scrolling in help mode (takes priority)
                                    app.scroll_down();
                                } else if app.format_mode == FormatMode::Edit {
                                    app.move_cursor_down();
                                } else if !app.relf_entries.is_empty() {
                                    // Move selection down in card view
                                    if app.selected_entry_index + 1 < app.relf_entries.len() {
                                        app.selected_entry_index += 1;
                                    }
                                } else {
                                    app.relf_jump_down();
                                }
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                if !app.showing_help {
                                    if app.format_mode == FormatMode::Edit {
                                        app.move_cursor_left();
                                    } else {
                                        // Faster horizontal pan in Relf
                                        app.relf_hscroll_by(-8);
                                    }
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if !app.showing_help {
                                    if app.format_mode == FormatMode::Edit {
                                        app.move_cursor_right();
                                    } else {
                                        app.relf_hscroll_by(8);
                                    }
                                }
                            }
                            KeyCode::Char('H') => {
                                if !app.showing_help && app.format_mode == FormatMode::View {
                                    let step = (app.get_content_width() / 2) as i16;
                                    app.relf_hscroll_by(-step);
                                }
                            }
                            KeyCode::Char('L') => {
                                if !app.showing_help && app.format_mode == FormatMode::View {
                                    let step = (app.get_content_width() / 2) as i16;
                                    app.relf_hscroll_by(step);
                                }
                            }
                            KeyCode::Char('0') => {
                                if !app.showing_help {
                                    if app.format_mode == FormatMode::View {
                                        app.hscroll = 0;
                                    } else if app.format_mode == FormatMode::Edit {
                                        app.content_cursor_col = 0;
                                        app.ensure_cursor_visible();
                                    }
                                }
                            }
                            KeyCode::Char('$') => {
                                if !app.showing_help {
                                    if app.format_mode == FormatMode::View {
                                        app.hscroll = app.relf_max_hscroll();
                                    } else if app.format_mode == FormatMode::Edit {
                                        let lines = app.get_json_lines();
                                        if app.content_cursor_line < lines.len() {
                                            app.content_cursor_col =
                                                lines[app.content_cursor_line].chars().count();
                                            app.ensure_cursor_visible();
                                        }
                                    }
                                }
                            }
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Char('G') => {
                                if app.showing_help {
                                    // Allow scrolling to bottom in help mode (takes priority)
                                    app.scroll_to_bottom();
                                } else if app.format_mode == FormatMode::Edit {
                                    app.scroll_to_bottom();
                                    let lines = app.get_json_lines();
                                    if !lines.is_empty() {
                                        app.content_cursor_line = lines.len() - 1;
                                        app.content_cursor_col = 0;
                                    }
                                } else if !app.relf_entries.is_empty() {
                                    // Jump to last card
                                    app.selected_entry_index = app.relf_entries.len() - 1;
                                } else {
                                    app.scroll_to_bottom();
                                }
                            }
                            KeyCode::Char('/') => {
                                if !app.showing_help {
                                    app.start_search();
                                }
                            }
                            KeyCode::Char('n') => {
                                if !app.showing_help {
                                    app.next_match();
                                }
                            }
                            KeyCode::Char('N') => {
                                if !app.showing_help {
                                    app.prev_match();
                                }
                            }
                            KeyCode::Enter => {
                                // Open edit overlay for selected card
                                if !app.showing_help && !app.relf_entries.is_empty() {
                                    app.start_editing_entry();
                                }
                            }
                            KeyCode::Char(c)
                                if c == 'g'
                                    || c == '-'
                                    || c == '+'
                                    || app.vim_buffer.starts_with('g') =>
                            {
                                // Allow gg in help mode for scrolling to top
                                app.handle_vim_input(c);
                            }
                            _ => {
                                // Reset dd count if any other key is pressed
                                if app.dd_count > 0 {
                                    app.dd_count = 0;
                                    app.vim_buffer.clear();
                                }
                            }
                        }
                        }
                        InputMode::Insert => {
                            // Check for Ctrl+[ to exit insert mode
                            if key.modifiers == KeyModifiers::CONTROL
                                && key.code == KeyCode::Char('[')
                            {
                                app.input_mode = InputMode::Normal;
                                app.set_status("");
                                continue;
                            }

                            match key.code {
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                    app.set_status("");
                                }
                                KeyCode::Enter => {
                                    app.insert_newline();
                                    app.is_modified = true;
                                }
                                KeyCode::Char(c) => {
                                    app.insert_char(c);
                                    app.is_modified = true;
                                }
                                KeyCode::Backspace => {
                                    app.backspace();
                                    app.is_modified = true;
                                }
                                KeyCode::Left => {
                                    app.move_cursor_left();
                                }
                                KeyCode::Right => {
                                    app.move_cursor_right();
                                }
                                KeyCode::Up => {
                                    app.move_cursor_up();
                                }
                                KeyCode::Down => {
                                    app.move_cursor_down();
                                }
                                KeyCode::Delete => {
                                    app.delete_char();
                                    app.is_modified = true;
                                }
                                _ => {}
                            }
                        }
                        InputMode::Command => match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.command_buffer.clear();
                                app.command_history_index = None;
                                app.set_status("");
                            }
                            KeyCode::Enter => {
                                // Add to history before executing
                                app.add_to_command_history(app.command_buffer.clone());

                                if app.execute_command() {
                                    return Ok(()); // Quit the application
                                }
                                app.input_mode = InputMode::Normal;
                                app.command_buffer.clear();
                            }
                            KeyCode::Up => {
                                if let Some(cmd) = app.get_previous_command() {
                                    app.command_buffer = cmd;
                                    app.set_status(&format!(":{}", app.command_buffer));
                                }
                            }
                            KeyCode::Down => {
                                if let Some(cmd) = app.get_next_command() {
                                    app.command_buffer = cmd;
                                    app.set_status(&format!(":{}", app.command_buffer));
                                }
                            }
                            KeyCode::Char(c) => {
                                app.command_buffer.push(c);
                                app.command_history_index = None;
                                app.set_status(&format!(":{}", app.command_buffer));
                            }
                            KeyCode::Backspace => {
                                if !app.command_buffer.is_empty() {
                                    app.command_buffer.pop();
                                    app.command_history_index = None;
                                    app.set_status(&format!(":{}", app.command_buffer));
                                } else {
                                    // Exit command mode when backspace on empty buffer
                                    app.input_mode = InputMode::Normal;
                                    app.command_history_index = None;
                                    app.set_status("");
                                }
                            }
                            _ => {}
                        },
                        InputMode::Search => match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.search_buffer.clear();
                                app.search_history_index = None;
                                app.set_status("");
                            }
                            KeyCode::Enter => {
                                // Add to history before executing
                                app.add_to_search_history(app.search_buffer.clone());
                                app.execute_search();
                            }
                            KeyCode::Up => {
                                if let Some(search) = app.get_previous_search() {
                                    app.search_buffer = search;
                                    app.set_status(&format!("/{}", app.search_buffer));
                                }
                            }
                            KeyCode::Down => {
                                if let Some(search) = app.get_next_search() {
                                    app.search_buffer = search;
                                    app.set_status(&format!("/{}", app.search_buffer));
                                }
                            }
                            KeyCode::Char(c) => {
                                app.search_buffer.push(c);
                                app.search_history_index = None;
                                app.set_status(&format!("/{}", app.search_buffer));
                            }
                            KeyCode::Backspace => {
                                if !app.search_buffer.is_empty() {
                                    app.search_buffer.pop();
                                    app.search_history_index = None;
                                    app.set_status(&format!("/{}", app.search_buffer));
                                } else {
                                    // Exit search mode when backspace on empty buffer
                                    app.input_mode = InputMode::Normal;
                                    app.search_history_index = None;
                                    app.set_status("");
                                }
                            }
                            _ => {}
                        },
                    }
                }
                Event::Mouse(mouse) => {
                    // Handle overlay mouse events
                    if app.editing_entry {
                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) if mouse.modifiers.is_empty() => {
                                // Check for double-click (clicks within 500ms)
                                let now = Instant::now();
                                let is_double_click = if let Some(last_time) = app.last_click_time {
                                    now.duration_since(last_time).as_millis() < 500
                                } else {
                                    false
                                };

                                if is_double_click {
                                    // Check if Exit field is selected
                                    if app.edit_field_index < app.edit_buffer.len() {
                                        let field = &app.edit_buffer[app.edit_field_index];
                                        if field == "Exit" {
                                            // Close overlay without saving
                                            app.cancel_editing_entry();
                                            app.last_click_time = None;
                                            continue;
                                        }
                                    }

                                    // Double-click: enter insert mode for currently selected field
                                    if !app.edit_insert_mode {
                                        app.edit_field_editing_mode = true;
                                        app.edit_insert_mode = true;
                                        app.edit_skip_normal_mode = true; // Mark that we skipped normal mode

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
                                    app.last_click_time = None; // Reset after double-click
                                } else {
                                    // First click: just record the time
                                    app.last_click_time = Some(now);
                                }
                                continue;
                            }
                            // Allow scrolling in overlay
                            MouseEventKind::ScrollUp => {
                                if app.edit_field_index > 0 {
                                    app.edit_field_index -= 1;
                                    app.edit_cursor_pos = 0;
                                }
                                continue;
                            }
                            MouseEventKind::ScrollDown => {
                                if app.edit_field_index + 1 < app.edit_buffer.len() {
                                    app.edit_field_index += 1;
                                    app.edit_cursor_pos = 0;
                                }
                                continue;
                            }
                            _ => {
                                // Other mouse events in overlay, ignore
                            }
                        }
                    }

                    match mouse.kind {
                        MouseEventKind::ScrollLeft => {
                            // Horizontal scroll left
                            if app.format_mode == FormatMode::View {
                                app.relf_hscroll_by(-8);
                            } else if app.format_mode == FormatMode::Edit {
                                app.relf_hscroll_by(-8);
                            }
                        }
                        MouseEventKind::ScrollRight => {
                            // Horizontal scroll right
                            if app.format_mode == FormatMode::View {
                                app.relf_hscroll_by(8);
                            } else if app.format_mode == FormatMode::Edit {
                                app.relf_hscroll_by(8);
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            // Don't scroll vertically if horizontal scrollbar is being dragged
                            if app.dragging_scrollbar != Some(ScrollbarType::Horizontal) {
                                if app.format_mode == FormatMode::Edit {
                                    // Move cursor up if it is not at the top of the visible area; otherwise scroll
                                    let (cursor_visual_line, _) =
                                        app.calculate_cursor_visual_position();
                                    let visible_top = app.scroll;
                                    if cursor_visual_line > visible_top {
                                        app.move_cursor_up();
                                    } else {
                                        // Faster scrolling for vim-like feel
                                        for _ in 0..5 {
                                            app.scroll_up();
                                        }
                                    }
                                } else if !app.relf_entries.is_empty() {
                                    // Card view: move selection up
                                    if app.selected_entry_index > 0 {
                                        app.selected_entry_index -= 1;
                                    }
                                } else {
                                    // Relf: clamp to content bounds
                                    let dec = 5u16;
                                    app.scroll = app.scroll.saturating_sub(dec);
                                }
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            // Don't scroll vertically if horizontal scrollbar is being dragged
                            if app.dragging_scrollbar != Some(ScrollbarType::Horizontal) {
                                if app.format_mode == FormatMode::Edit {
                                    // Move cursor down while within the visible area; otherwise scroll
                                    let (cursor_visual_line, _) =
                                        app.calculate_cursor_visual_position();
                                    let visible_height = app.get_visible_height();
                                    let visible_bottom =
                                        app.scroll.saturating_add(visible_height).saturating_sub(1);
                                    // Estimate total visual lines to avoid overshooting content
                                    let mut total_visual: u16 = 0;
                                    for l in app.json_input.lines() {
                                        total_visual = total_visual
                                            .saturating_add(app.calculate_visual_lines(l));
                                    }
                                    let last_visual = total_visual.saturating_sub(1);
                                    let effective_bottom =
                                        std::cmp::min(visible_bottom, last_visual);
                                    if cursor_visual_line < effective_bottom {
                                        app.move_cursor_down();
                                    } else {
                                        // Faster scrolling for vim-like feel
                                        for _ in 0..5 {
                                            app.scroll_down();
                                        }
                                    }
                                } else if !app.relf_entries.is_empty() {
                                    // Card view: move selection down
                                    if app.selected_entry_index + 1 < app.relf_entries.len() {
                                        app.selected_entry_index += 1;
                                    }
                                } else {
                                    // Relf: clamp to last content page
                                    let inc = 5u16;
                                    let max_off = app.relf_content_max_scroll();
                                    let new_val = app.scroll.saturating_add(inc);
                                    app.scroll = std::cmp::min(new_val, max_off);
                                }
                            }
                        }
                        MouseEventKind::Down(MouseButton::Left) => {
                            // Disable scrollbar dragging in Edit mode
                            if app.format_mode == FormatMode::Edit {
                                continue;
                            }

                            // Handle mouse click on scrollbars
                            let click_x = mouse.column;
                            let click_y = mouse.row;

                            // Check if click is on vertical scrollbar
                            let terminal_width = terminal.size().map(|s| s.width).unwrap_or(80);
                            let terminal_height = terminal.size().map(|s| s.height).unwrap_or(24);

                            // Check horizontal scrollbar first
                            let on_hscrollbar = click_y >= terminal_height.saturating_sub(2)
                                && click_x > 0
                                && click_x < terminal_width - 1;
                            let on_vscrollbar = click_x == terminal_width - 1
                                && click_y > 0
                                && click_y < terminal_height - 1;

                            if on_hscrollbar {
                                // Horizontal scrollbar clicked
                                app.dragging_scrollbar = Some(ScrollbarType::Horizontal);
                                let max_hscroll = if app.format_mode == FormatMode::View {
                                    app.relf_max_hscroll()
                                } else {
                                    app.relf_max_hscroll()
                                };

                                if max_hscroll > 0 {
                                    let scrollbar_width = (terminal_width - 2) as f32;
                                    let click_ratio = (click_x - 1) as f32 / scrollbar_width;
                                    let new_hscroll = (max_hscroll as f32 * click_ratio) as u16;
                                    app.hscroll = new_hscroll.min(max_hscroll);
                                }
                            } else if on_vscrollbar {
                                // Vertical scrollbar clicked
                                app.dragging_scrollbar = Some(ScrollbarType::Vertical);
                                let scrollbar_height = (terminal_height - 2) as f32;
                                let click_ratio = (click_y - 1) as f32 / scrollbar_height;
                                let new_scroll = (app.max_scroll as f32 * click_ratio) as u16;
                                app.scroll = new_scroll.min(app.max_scroll);
                            } else {
                                // Not on any scrollbar - check for double-click in View mode
                                if app.format_mode == FormatMode::View && !app.relf_entries.is_empty() {
                                    // Check for double-click (clicks within 500ms)
                                    let now = Instant::now();
                                    let is_double_click = if let Some(last_time) = app.last_click_time {
                                        now.duration_since(last_time).as_millis() < 500
                                    } else {
                                        false
                                    };

                                    if is_double_click {
                                        // Double-click: open the overlay for the currently selected entry
                                        app.open_entry_overlay();
                                        app.last_click_time = None; // Reset after double-click
                                    } else {
                                        // First click: just record the time
                                        app.last_click_time = Some(now);
                                    }
                                }
                                app.dragging_scrollbar = None;
                            }
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            // Disable in Edit mode
                            if app.format_mode == FormatMode::Edit {
                                continue;
                            }
                            // Release scrollbar drag
                            app.dragging_scrollbar = None;
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            // Disable in Edit mode
                            if app.format_mode == FormatMode::Edit {
                                continue;
                            }
                            // Only handle drag if we're already dragging a scrollbar
                            match app.dragging_scrollbar {
                                Some(ScrollbarType::Vertical) => {
                                    // Continue vertical scrollbar drag
                                    let click_y = mouse.row;
                                    let terminal_height =
                                        terminal.size().map(|s| s.height).unwrap_or(24);

                                    if click_y > 0 && click_y < terminal_height - 1 {
                                        let scrollbar_height = (terminal_height - 2) as f32;
                                        let click_ratio = (click_y - 1) as f32 / scrollbar_height;
                                        let new_scroll =
                                            (app.max_scroll as f32 * click_ratio) as u16;
                                        app.scroll = new_scroll.min(app.max_scroll);
                                    }
                                }
                                Some(ScrollbarType::Horizontal) => {
                                    // Continue horizontal scrollbar drag
                                    let click_x = mouse.column;
                                    let terminal_width =
                                        terminal.size().map(|s| s.width).unwrap_or(80);

                                    if click_x > 0 && click_x < terminal_width - 1 {
                                        let max_hscroll = if app.format_mode == FormatMode::View {
                                            app.relf_max_hscroll()
                                        } else {
                                            app.relf_max_hscroll()
                                        };

                                        if max_hscroll > 0 {
                                            let scrollbar_width = (terminal_width - 2) as f32;
                                            let click_ratio =
                                                (click_x - 1) as f32 / scrollbar_width;
                                            let new_hscroll =
                                                (max_hscroll as f32 * click_ratio) as u16;
                                            app.hscroll = new_hscroll.min(max_hscroll);
                                        }
                                    }
                                }
                                None => {
                                    // Not dragging any scrollbar, ignore
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Paste(_) => {
                    // Paste events not supported - use 'v' key instead
                }
                _ => {}
            }
        }
    }
}
