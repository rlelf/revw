use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;

use crate::app::{App, FormatMode, ScrollbarType};

pub fn handle_mouse_event<B: ratatui::backend::Backend>(
    app: &mut App,
    mouse: MouseEvent,
    terminal: &mut ratatui::Terminal<B>,
) -> Result<()> {
    // Handle overlay mouse events
    if app.editing_entry {
        handle_overlay_mouse(app, mouse);
        return Ok(());
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
                // If outline has focus, scroll outline
                if app.outline_open && app.outline_has_focus {
                    app.outline_move_up();
                // If explorer has focus, scroll explorer
                } else if app.explorer_open && app.explorer_has_focus {
                    app.explorer_move_up();
                } else if app.format_mode == FormatMode::Edit {
                    // Scroll and move cursor together
                    for _ in 0..5 {
                        if app.content_cursor_line > 0 {
                            app.move_cursor_up();
                        } else {
                            app.scroll_up();
                        }
                    }
                } else if !app.relf_entries.is_empty() {
                    // Card view: move selection up
                    if app.selected_entry_index > 0 {
                        app.selected_entry_index -= 1;
                        // Reset vertical scroll when changing cards (hscroll is misused as vscroll for cards)
                        app.hscroll = 0;
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
                // If outline has focus, scroll outline
                if app.outline_open && app.outline_has_focus {
                    app.outline_move_down();
                // If explorer has focus, scroll explorer
                } else if app.explorer_open && app.explorer_has_focus {
                    app.explorer_move_down();
                } else if app.format_mode == FormatMode::Edit {
                    // Scroll and move cursor together
                    for _ in 0..5 {
                        app.move_cursor_down();
                    }
                } else if !app.relf_entries.is_empty() {
                    // Card view: move selection down
                    if app.selected_entry_index + 1 < app.relf_entries.len() {
                        app.selected_entry_index += 1;
                        // Reset vertical scroll when changing cards (hscroll is misused as vscroll for cards)
                        app.hscroll = 0;
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
            handle_left_mouse_down(app, mouse, terminal)?;
        }
        MouseEventKind::Up(MouseButton::Left) => {
            // Disable in Edit mode
            if app.format_mode == FormatMode::Edit {
                return Ok(());
            }
            // Release scrollbar drag
            app.dragging_scrollbar = None;
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            handle_left_mouse_drag(app, mouse, terminal)?;
        }
        _ => {}
    }

    Ok(())
}

fn handle_overlay_mouse(app: &mut App, mouse: MouseEvent) {
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
                        return;
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
        }
        // Allow scrolling in overlay (only in field selection mode)
        MouseEventKind::ScrollUp => {
            // Block if in field editing mode (normal/insert)
            if app.edit_field_editing_mode {
                return;
            }
            if app.edit_field_index > 0 {
                app.edit_field_index -= 1;
                app.edit_cursor_pos = 0;
                app.edit_hscroll = 0;
                app.edit_vscroll = 0;
            }
        }
        MouseEventKind::ScrollDown => {
            // Block if in field editing mode (normal/insert)
            if app.edit_field_editing_mode {
                return;
            }
            if app.edit_field_index + 1 < app.edit_buffer.len() {
                app.edit_field_index += 1;
                app.edit_cursor_pos = 0;
                app.edit_hscroll = 0;
                app.edit_vscroll = 0;
            }
        }
        _ => {
            // Other mouse events in overlay, ignore
        }
    }
}

fn handle_left_mouse_down<B: ratatui::backend::Backend>(
    app: &mut App,
    mouse: MouseEvent,
    terminal: &mut ratatui::Terminal<B>,
) -> Result<()> {
    // Disable scrollbar dragging in Edit mode
    if app.format_mode == FormatMode::Edit {
        return Ok(());
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

    if on_hscrollbar && app.format_mode == FormatMode::Edit {
        // Horizontal scrollbar clicked (Edit mode only)
        app.dragging_scrollbar = Some(ScrollbarType::Horizontal);
        let max_hscroll = app.relf_max_hscroll();

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
        // Not on any scrollbar - check for double-click
        // Check for double-click (clicks within 500ms)
        let now = Instant::now();
        let is_double_click = if let Some(last_time) = app.last_click_time {
            now.duration_since(last_time).as_millis() < 500
        } else {
            false
        };

        if is_double_click {
            // If explorer has focus, open file and move to file window
            if app.explorer_open && app.explorer_has_focus {
                app.explorer_select_entry();
            } else if app.format_mode == FormatMode::View && !app.relf_entries.is_empty() {
                // Double-click: open the overlay for the currently selected entry
                app.open_entry_overlay();
            }
            app.last_click_time = None; // Reset after double-click
        } else {
            // First click: just record the time
            app.last_click_time = Some(now);
        }
        app.dragging_scrollbar = None;
    }

    Ok(())
}

fn handle_left_mouse_drag<B: ratatui::backend::Backend>(
    app: &mut App,
    mouse: MouseEvent,
    terminal: &mut ratatui::Terminal<B>,
) -> Result<()> {
    // Disable in Edit mode
    if app.format_mode == FormatMode::Edit {
        return Ok(());
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
            // Continue horizontal scrollbar drag (Edit mode only)
            if app.format_mode == FormatMode::Edit {
                let click_x = mouse.column;
                let terminal_width =
                    terminal.size().map(|s| s.width).unwrap_or(80);

                if click_x > 0 && click_x < terminal_width - 1 {
                    let max_hscroll = app.relf_max_hscroll();

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
        }
        None => {
            // Not dragging any scrollbar, ignore
        }
    }

    Ok(())
}
