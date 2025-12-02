use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, FileOperation, FormatMode};

pub fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Handle file operation confirmation/prompt if active
    if let Some(ref op) = app.file_op_pending.clone() {
        return handle_file_operation(app, key, op);
    }

    // Handle substitute confirmation if active
    if !app.substitute_confirmations.is_empty() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('n') | KeyCode::Char('a') | KeyCode::Char('q') => {
                if let KeyCode::Char(c) = key.code {
                    app.handle_substitute_confirmation(c);
                }
                return Ok(false);
            }
            KeyCode::Esc => {
                app.handle_substitute_confirmation('q');
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    // Handle explorer navigation if explorer has focus
    if app.explorer_open && app.explorer_has_focus {
        return handle_explorer_navigation(app, key);
    }

    // Handle outline navigation if outline is open
    if app.outline_open {
        return handle_outline_navigation(app, key);
    }

    // Main normal mode keyboard handling
    match key.code {
        KeyCode::Char('u') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                app.undo();
            }
        }
        KeyCode::Char('v') => {
            // Enter Visual/Select mode in View mode
            if !app.showing_help && app.format_mode == FormatMode::View && !app.relf_entries.is_empty() {
                app.visual_mode = true;
                app.visual_start_index = app.selected_entry_index;
                app.visual_end_index = app.selected_entry_index;
                app.set_status("-- VISUAL --");
            }
        }
        KeyCode::Char('?') => {
            // Toggle help
            app.toggle_help();
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('[') => {
            // Check for Ctrl+[ to exit Visual mode
            if key.code == KeyCode::Char('[') && !key.modifiers.contains(KeyModifiers::CONTROL) {
                // Not Ctrl+[, ignore
            } else {
                // Exit Visual mode if active, otherwise quit
                if app.visual_mode {
                    app.visual_mode = false;
                    app.set_status("");
                } else {
                    return Ok(true);
                }
            }
        }
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
            // Ctrl+b: page up (vim-like)
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.page_up();
            } else {
                // Vim-like: move to start of previous word (Edit mode)
                // Or scroll card content up in View mode (like h)
                if !app.showing_help {
                    if app.format_mode == FormatMode::Edit {
                        app.move_to_previous_word_start();
                    } else {
                        // Scroll card content up (like h key)
                        app.hscroll = app.hscroll.saturating_sub(1);
                    }
                }
            }
        }
        KeyCode::Char('f') => {
            // Ctrl+f: page down (vim-like)
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.page_down();
            } else {
                // Scroll card content down in View mode (like l key)
                if !app.showing_help && app.format_mode == FormatMode::View {
                    app.hscroll += 1;
                }
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
                app.input_mode = crate::app::InputMode::Insert;
                app.ensure_cursor_visible();
                app.set_status("-- INSERT --");
            }
        }
        KeyCode::Char('a') if !app.substitute_confirmations.is_empty() => {
            // Handle substitute confirmation 'a' (replace all)
            // This case is handled elsewhere
        }
        KeyCode::Char('a') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                // Append: move cursor right then enter insert mode
                app.move_cursor_right();
                app.input_mode = crate::app::InputMode::Insert;
                app.ensure_cursor_visible();
                app.set_status("-- INSERT --");
            }
        }
        KeyCode::Char('o') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                // Open line below: insert new line and enter insert mode
                app.open_line_below();
                app.input_mode = crate::app::InputMode::Insert;
                app.set_status("-- INSERT --");
            }
        }
        KeyCode::Char('x') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                app.delete_char();
                app.is_modified = true;
            }
        }
        KeyCode::Char('X') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                app.backspace();
                app.is_modified = true;
            }
        }
        KeyCode::Char('d') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                // Handle dd (delete line)
                app.dd_count += 1;
                if app.dd_count == 2 {
                    app.delete_line();
                    app.dd_count = 0;
                }
            }
        }
        KeyCode::Char('y') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                // Handle yy (yank line)
                app.yy_count += 1;
                if app.yy_count == 2 {
                    app.yank_line();
                    app.yy_count = 0;
                }
            }
        }
        KeyCode::Char('p') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                // Paste yanked line
                app.paste_line();
            }
        }
        KeyCode::Char(':') => {
            // Allow command mode even when showing help (for :h to toggle)
            app.input_mode = crate::app::InputMode::Command;
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
                    // Reset horizontal scroll when changing cards
                    app.hscroll = 0;
                    // In Visual mode, extend selection
                    if app.visual_mode {
                        app.visual_end_index = app.selected_entry_index;
                    }
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
                    // Reset horizontal scroll when changing cards
                    app.hscroll = 0;
                    // In Visual mode, extend selection
                    if app.visual_mode {
                        app.visual_end_index = app.selected_entry_index;
                    }
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
                    // Check for Ctrl modifier for page scroll
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // Page scroll up (e.g., 5 lines at a time)
                        app.hscroll = app.hscroll.saturating_sub(5);
                    } else {
                        // Vertical scroll up in View mode (card context)
                        app.hscroll = app.hscroll.saturating_sub(1);
                    }
                }
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if !app.showing_help {
                if app.format_mode == FormatMode::Edit {
                    app.move_cursor_right();
                } else {
                    // Check for Ctrl modifier for page scroll
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // Page scroll down (e.g., 5 lines at a time)
                        app.hscroll += 5;
                    } else {
                        // Vertical scroll down in View mode (card context)
                        app.hscroll += 1;
                    }
                }
            }
        }
        KeyCode::Char('0') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                app.content_cursor_col = 0;
                app.ensure_cursor_visible();
            }
        }
        KeyCode::Char('$') => {
            if !app.showing_help && app.format_mode == FormatMode::Edit {
                let lines = app.get_content_lines();
                if app.content_cursor_line < lines.len() {
                    app.content_cursor_col =
                        lines[app.content_cursor_line].chars().count();
                    app.ensure_cursor_visible();
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
                let lines = app.get_content_lines();
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
            // Open edit overlay for selected card (only in View mode)
            if !app.showing_help && !app.relf_entries.is_empty() && app.format_mode == FormatMode::View {
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
            // Reset dd/yy count if any other key is pressed
            let should_clear = app.dd_count > 0 || app.yy_count > 0;
            app.dd_count = 0;
            app.yy_count = 0;
            if should_clear {
                app.vim_buffer.clear();
            }
        }
    }

    Ok(false)
}

fn handle_file_operation(app: &mut App, key: KeyEvent, op: &FileOperation) -> Result<bool> {
    match op {
        FileOperation::Delete(_) => {
            // Waiting for yes/no confirmation
            match key.code {
                KeyCode::Esc => {
                    app.cancel_file_operation();
                    return Ok(false);
                }
                KeyCode::Enter => {
                    let input = app.file_op_prompt_buffer.trim().to_lowercase();
                    if input == "yes" {
                        app.handle_file_op_confirmation('y');
                    } else if input == "no" {
                        app.handle_file_op_confirmation('n');
                    } else {
                        app.set_status("Invalid input. Type 'yes' or 'no'");
                        app.file_op_prompt_buffer.clear();
                    }
                    return Ok(false);
                }
                KeyCode::Char(c) => {
                    app.file_op_prompt_buffer.push(c);
                    let path_display = if let FileOperation::Delete(path) = op {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                        let item_type = if path.is_dir() { "directory" } else { "file" };
                        format!("Delete {} '{}'? (yes/no) {}", item_type, name, app.file_op_prompt_buffer)
                    } else {
                        String::new()
                    };
                    app.set_status(&path_display);
                    return Ok(false);
                }
                KeyCode::Backspace => {
                    if !app.file_op_prompt_buffer.is_empty() {
                        app.file_op_prompt_buffer.pop();
                        let path_display = if let FileOperation::Delete(path) = op {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                            let item_type = if path.is_dir() { "directory" } else { "file" };
                            format!("Delete {} '{}'? (yes/no) {}", item_type, name, app.file_op_prompt_buffer)
                        } else {
                            String::new()
                        };
                        app.set_status(&path_display);
                    } else {
                        app.cancel_file_operation();
                    }
                    return Ok(false);
                }
                _ => return Ok(false),
            }
        }
        FileOperation::Create | FileOperation::CreateDir | FileOperation::Copy(_) | FileOperation::Rename(_) => {
            // Waiting for filename input
            match key.code {
                KeyCode::Esc => {
                    app.cancel_file_operation();
                    return Ok(false);
                }
                KeyCode::Enter => {
                    app.execute_file_operation();
                    return Ok(false);
                }
                KeyCode::Char(c) => {
                    app.file_op_prompt_buffer.push(c);
                    let prompt_msg = match op {
                        FileOperation::Create => "New file name (must end with .json or .md):",
                        FileOperation::CreateDir => "New directory name:",
                        FileOperation::Copy(src) => {
                            let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                            &format!("Copy '{}' to (must end with .json or .md):", name)
                        }
                        FileOperation::Rename(path) => {
                            if path.is_dir() {
                                "Rename/Move directory to:"
                            } else {
                                "Rename/Move to (must end with .json or .md):"
                            }
                        }
                        _ => "",
                    };
                    app.set_status(&format!("{} {}", prompt_msg, app.file_op_prompt_buffer));
                    return Ok(false);
                }
                KeyCode::Backspace => {
                    if !app.file_op_prompt_buffer.is_empty() {
                        app.file_op_prompt_buffer.pop();
                        let prompt_msg = match op {
                            FileOperation::Create => "New file name (must end with .json or .md):",
                            FileOperation::CreateDir => "New directory name:",
                            FileOperation::Copy(src) => {
                                let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                                &format!("Copy '{}' to (must end with .json or .md):", name)
                            }
                            FileOperation::Rename(path) => {
                                if path.is_dir() {
                                    "Rename/Move directory to:"
                                } else {
                                    "Rename/Move to (must end with .json or .md):"
                                }
                            }
                            _ => "",
                        };
                        app.set_status(&format!("{} {}", prompt_msg, app.file_op_prompt_buffer));
                    } else {
                        app.cancel_file_operation();
                    }
                    return Ok(false);
                }
                _ => return Ok(false),
            }
        }
    }
}

fn handle_explorer_navigation(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char(':') => {
            // Allow command mode from explorer
            app.input_mode = crate::app::InputMode::Command;
            app.command_buffer = String::new();
            app.command_history_index = None;
            app.set_status(":");
            return Ok(false);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.explorer_move_down();
            return Ok(false);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.explorer_move_up();
            return Ok(false);
        }
        KeyCode::Char('h') | KeyCode::Left => {
            // Scroll left
            if app.explorer_horizontal_scroll > 0 {
                app.explorer_horizontal_scroll -= 1;
            }
            return Ok(false);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            // Scroll right (max 100)
            if app.explorer_horizontal_scroll < 100 {
                app.explorer_horizontal_scroll += 1;
            }
            return Ok(false);
        }
        KeyCode::Char('G') => {
            // Go to bottom
            if !app.explorer_entries.is_empty() {
                app.explorer_selected_index = app.explorer_entries.len() - 1;
                app.explorer_update_scroll();
            }
            return Ok(false);
        }
        KeyCode::Char('/') => {
            // Start search mode
            app.input_mode = crate::app::InputMode::Search;
            app.search_buffer = String::new();
            app.search_history_index = None;
            app.set_status("/");
            return Ok(false);
        }
        KeyCode::Char('n') => {
            // Next search match
            app.explorer_next_match();
            return Ok(false);
        }
        KeyCode::Char('N') => {
            // Previous search match
            app.explorer_prev_match();
            return Ok(false);
        }
        KeyCode::Enter => {
            // Open file and move focus to right
            app.explorer_select_entry();
            return Ok(false);
        }
        KeyCode::Char('o') => {
            // Check if this might be part of 'go'
            if app.vim_buffer == "g" {
                // Let handle_vim_input process 'go'
                app.handle_vim_input('o');
            } else {
                // Standalone 'o' - open file
                app.explorer_select_entry();
            }
            return Ok(false);
        }
        KeyCode::Char('q') => {
            // Quit program
            return Ok(true);
        }
        KeyCode::Char('g') => {
            // Start of potential 'go' or 'gg' or 'G'
            if app.vim_buffer == "g" {
                // Second 'g' - go to top
                app.explorer_selected_index = 0;
                app.explorer_update_scroll();
                app.vim_buffer.clear();
            } else {
                app.handle_vim_input('g');
            }
            return Ok(false);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_outline_navigation(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.outline_move_down();
            return Ok(false);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.outline_move_up();
            return Ok(false);
        }
        KeyCode::Char('f') => {
            // Ctrl+f: page down
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.outline_page_down();
                return Ok(false);
            }
        }
        KeyCode::Char('b') => {
            // Ctrl+b: page up
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.outline_page_up();
                return Ok(false);
            }
        }
        KeyCode::Char('G') => {
            // Go to bottom
            let max_index = if app.format_mode == FormatMode::View {
                app.relf_entries.len().saturating_sub(1)
            } else {
                app.get_entry_count_from_content().saturating_sub(1)
            };
            app.outline_selected_index = max_index;
            return Ok(false);
        }
        KeyCode::Char('g') => {
            if app.vim_buffer == "g" {
                // gg - go to top
                app.outline_selected_index = 0;
                app.vim_buffer.clear();
            } else {
                app.handle_vim_input('g');
            }
            return Ok(false);
        }
        KeyCode::Char('/') => {
            // Start search mode
            app.input_mode = crate::app::InputMode::Search;
            app.search_buffer = String::new();
            app.search_history_index = None;
            app.set_status("/");
            return Ok(false);
        }
        KeyCode::Char('n') => {
            // Next search match
            app.outline_next_match();
            return Ok(false);
        }
        KeyCode::Char('N') => {
            // Previous search match
            app.outline_prev_match();
            return Ok(false);
        }
        KeyCode::Enter => {
            // Jump to selected entry
            app.outline_jump_to_selected();
            return Ok(false);
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            // Close outline (use toggle_outline to properly restore focus)
            app.toggle_outline();
            return Ok(false);
        }
        _ => {}
    }
    Ok(false)
}
