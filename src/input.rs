use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseEventKind, MouseButton, KeyEventKind},
};
use std::time::Duration;

use crate::app::{App, InputMode, FormatMode};

pub fn run_app<B: ratatui::backend::Backend>(terminal: &mut ratatui::Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| crate::ui::ui(f, &mut app))?;
        app.update_status();

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

                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('v') => app.paste_from_clipboard(),
                            KeyCode::Char('c') => app.copy_to_clipboard(),
                            KeyCode::Char('e') => {
                                // Vim-like: move to end of next word (JSON mode)
                                if app.format_mode == FormatMode::Json {
                                    app.move_to_next_word_end();
                                }
                            }
                            KeyCode::Char('b') => {
                                // Vim-like: move to start of previous word (JSON mode)
                                if app.format_mode == FormatMode::Json {
                                    app.move_to_previous_word_start();
                                }
                            }
                            KeyCode::Char('d') => {
                                // Handle dd command for deleting data entries
                                if app.format_mode == FormatMode::Json {
                                    app.dd_count += 1;
                                    if app.dd_count == 2 {
                                        app.delete_current_entry();
                                        app.dd_count = 0;
                                    } else {
                                        // Start the dd sequence
                                        app.vim_buffer = "d".to_string();
                                        app.set_status("Press 'd' again to delete entry");
                                    }
                                }
                            }
                            KeyCode::Char('r') => {
                                app.format_mode = match app.format_mode {
                                    FormatMode::Relf => FormatMode::Json,
                                    FormatMode::Json => FormatMode::Relf,
                                };
                                let mode_name = match app.format_mode {
                                    FormatMode::Relf => "Relf",
                                    FormatMode::Json => "JSON",
                                };
                                if app.format_mode == FormatMode::Relf { app.hscroll = 0; }
                                app.set_status(&format!("{} mode", mode_name));
                                app.convert_json();
                            }
                            KeyCode::Char('i') => {
                                if app.format_mode == FormatMode::Json {
                                    app.input_mode = InputMode::Insert;
                                    app.ensure_cursor_visible();
                                    app.set_status("-- INSERT --");
                                }
                            }
                            KeyCode::Char(':') => {
                                app.input_mode = InputMode::Command;
                                app.command_buffer = String::new();
                                app.set_status(":");
                            }
                            KeyCode::Char('x') => {
                                app.clear_content();
                                app.set_status("");
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.format_mode == FormatMode::Json {
                                    app.move_cursor_up();
                                } else {
                                    app.relf_jump_up();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.format_mode == FormatMode::Json {
                                    app.move_cursor_down();
                                } else {
                                    app.relf_jump_down();
                                }
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                if app.format_mode == FormatMode::Json {
                                    app.move_cursor_left();
                                } else {
                                    // Faster horizontal pan in Relf
                                    app.relf_hscroll_by(-8);
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if app.format_mode == FormatMode::Json {
                                    app.move_cursor_right();
                                } else {
                                    app.relf_hscroll_by(8);
                                }
                            }
                            KeyCode::Char('H') => {
                                if app.format_mode == FormatMode::Relf {
                                    let step = (app.get_content_width() / 2) as i16;
                                    app.relf_hscroll_by(-step);
                                }
                            }
                            KeyCode::Char('L') => {
                                if app.format_mode == FormatMode::Relf {
                                    let step = (app.get_content_width() / 2) as i16;
                                    app.relf_hscroll_by(step);
                                }
                            }
                            KeyCode::Char('0') => {
                                if app.format_mode == FormatMode::Relf {
                                    app.hscroll = 0;
                                } else if app.format_mode == FormatMode::Json {
                                    app.content_cursor_col = 0;
                                    app.ensure_cursor_visible();
                                }
                            }
                            KeyCode::Char('$') => {
                                if app.format_mode == FormatMode::Relf {
                                    app.hscroll = app.relf_max_hscroll();
                                } else if app.format_mode == FormatMode::Json {
                                    let lines = app.get_json_lines();
                                    if app.content_cursor_line < lines.len() {
                                        app.content_cursor_col = lines[app.content_cursor_line].chars().count();
                                        app.ensure_cursor_visible();
                                    }
                                }
                            }
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Char('G') => {
                                app.scroll_to_bottom();
                                if app.format_mode == FormatMode::Json {
                                    let lines = app.get_json_lines();
                                    if !lines.is_empty() {
                                        app.content_cursor_line = lines.len() - 1;
                                        app.content_cursor_col = 0;
                                    }
                                }
                            }
                            KeyCode::Char('/') => {
                                app.start_search();
                            }
                            KeyCode::Char('n') => {
                                app.next_match();
                            }
                            KeyCode::Char('N') => {
                                app.prev_match();
                            }
                            KeyCode::Char(c) if c == 'g' || app.vim_buffer.starts_with('g') => {
                                app.handle_vim_input(c);
                            }
                            _ => {
                                // Reset dd count if any other key is pressed
                                if app.dd_count > 0 {
                                    app.dd_count = 0;
                                    app.vim_buffer.clear();
                                }
                            }
                        },
                        InputMode::Insert => match key.code {
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
                        },
                        InputMode::Command => match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.command_buffer.clear();
                                app.set_status("");
                            }
                            KeyCode::Enter => {
                                if app.execute_command() {
                                    return Ok(()); // Quit the application
                                }
                                app.input_mode = InputMode::Normal;
                                app.command_buffer.clear();
                            }
                            KeyCode::Char(c) => {
                                app.command_buffer.push(c);
                                app.set_status(&format!(":{}", app.command_buffer));
                            }
                            KeyCode::Backspace => {
                                if !app.command_buffer.is_empty() {
                                    app.command_buffer.pop();
                                    app.set_status(&format!(":{}", app.command_buffer));
                                } else {
                                    // Exit command mode when backspace on empty buffer
                                    app.input_mode = InputMode::Normal;
                                    app.set_status("");
                                }
                            }
                            _ => {}
                        },
                        InputMode::Search => match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.search_buffer.clear();
                                app.set_status("");
                            }
                            KeyCode::Enter => {
                                app.execute_search();
                            }
                            KeyCode::Char(c) => {
                                app.search_buffer.push(c);
                                app.set_status(&format!("/{}", app.search_buffer));
                            }
                            KeyCode::Backspace => {
                                if !app.search_buffer.is_empty() {
                                    app.search_buffer.pop();
                                    app.set_status(&format!("/{}", app.search_buffer));
                                }
                            }
                            _ => {}
                        },
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        if app.format_mode == FormatMode::Json {
                            // Move cursor up if it is not at the top of the visible area; otherwise scroll
                            let (cursor_visual_line, _) = app.calculate_cursor_visual_position();
                            let visible_top = app.scroll;
                            if cursor_visual_line > visible_top {
                                app.move_cursor_up();
                            } else {
                                app.scroll_up();
                                app.scroll_up();
                                app.scroll_up();
                            }
                        } else {
                            // Relf: clamp to content bounds
                            let dec = 3u16;
                            app.scroll = app.scroll.saturating_sub(dec);
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if app.format_mode == FormatMode::Json {
                            // Move cursor down while within the visible area; otherwise scroll
                            let (cursor_visual_line, _) = app.calculate_cursor_visual_position();
                            let visible_height = app.get_visible_height();
                            let visible_bottom = app.scroll.saturating_add(visible_height).saturating_sub(1);
                            // Estimate total visual lines to avoid overshooting content
                            let mut total_visual: u16 = 0;
                            for l in app.json_input.lines() {
                                total_visual = total_visual.saturating_add(app.calculate_visual_lines(l));
                            }
                            let last_visual = total_visual.saturating_sub(1);
                            let effective_bottom = std::cmp::min(visible_bottom, last_visual);
                            if cursor_visual_line < effective_bottom {
                                app.move_cursor_down();
                            } else {
                                app.scroll_down();
                                app.scroll_down();
                                app.scroll_down();
                            }
                        } else {
                            // Relf: clamp to last content page
                            let inc = 3u16;
                            let max_off = app.relf_content_max_scroll();
                            let new_val = app.scroll.saturating_add(inc);
                            app.scroll = std::cmp::min(new_val, max_off);
                        }
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Mouse click - no action
                    }
                    _ => {}
                },
                Event::Paste(_) => {
                    // Paste events not supported - use 'v' key instead
                }
                _ => {}
            }
        }
    }
}