use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use crate::app::App;

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    mut app: App,
) -> Result<()> {
    // Setup file watcher
    let (tx, mut rx): (std::sync::mpsc::Sender<NotifyEvent>, Receiver<NotifyEvent>) = mpsc::channel();
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

    // Watch explorer directory if open
    if app.explorer_open {
        let _ = watcher.watch(&app.explorer_current_dir, RecursiveMode::NonRecursive);
    }

    loop {
        terminal.draw(|f| crate::ui::ui(f, &mut app))?;
        app.update_status();

        // Update watcher if file path or explorer directory changed
        if app.file_path_changed || app.explorer_dir_changed {
            // Unwatch all (recreate watcher to avoid keeping old watches)
            drop(watcher);
            let (new_tx, new_rx): (std::sync::mpsc::Sender<NotifyEvent>, Receiver<NotifyEvent>) = mpsc::channel();
            watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, notify::Error>| {
                if let Ok(event) = res {
                    let _ = new_tx.send(event);
                }
            })?;

            // Watch the new file
            if let Some(ref path) = app.file_path {
                let _ = watcher.watch(path, RecursiveMode::NonRecursive);
            }

            // Watch explorer directory if open
            if app.explorer_open {
                let _ = watcher.watch(&app.explorer_current_dir, RecursiveMode::NonRecursive);
            }

            // Update the receiver to use the new channel
            rx = new_rx;
            app.file_path_changed = false;
            app.explorer_dir_changed = false;
        }

        // Check for file changes
        if app.auto_reload {
            match rx.try_recv() {
                Ok(event) => {
                    // Check if it's a modify event for files
                    if matches!(event.kind, notify::EventKind::Modify(_)) {
                        // Ignore file changes within 1 second after saving (to avoid reloading our own save)
                        let should_reload = if let Some(last_save) = app.last_save_time {
                            last_save.elapsed() > Duration::from_millis(1000)
                        } else {
                            true
                        };

                        // Only reload if not modified by user and not recently saved
                        if !app.is_modified && should_reload && app.file_path.is_some() {
                            app.reload_file();
                        }
                    }
                    // Check for create/delete/modify events in explorer directory
                    if app.explorer_open && (matches!(event.kind, notify::EventKind::Create(_)) || matches!(event.kind, notify::EventKind::Remove(_)) || matches!(event.kind, notify::EventKind::Modify(_))) {
                        // Reload explorer entries (without resetting cursor position)
                        app.reload_explorer_entries();
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

                    // Handle Ctrl+w window commands
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
                        // Wait for next key (1000ms timeout)
                        loop {
                            if let Ok(true) = event::poll(Duration::from_millis(1000)) {
                                if let Ok(Event::Key(next_key)) = event::read() {
                                    #[cfg(target_os = "windows")]
                                    {
                                        // Skip release events on Windows
                                        if next_key.kind != KeyEventKind::Press {
                                            continue;
                                        }
                                    }

                                    match next_key.code {
                                        KeyCode::Char('w') => {
                                            // Ctrl+w w: cycle between windows (accept with or without Ctrl)
                                            app.switch_window_focus();
                                            let focus_msg = if app.explorer_has_focus {
                                                "Focused explorer"
                                            } else if app.outline_has_focus {
                                                "Focused outline"
                                            } else {
                                                "Focused file window"
                                            };
                                            app.set_status(focus_msg);
                                            break;
                                        }
                                        KeyCode::Char('h') => {
                                            // Ctrl+w h: move to left window (explorer)
                                            app.focus_explorer();
                                            app.set_status("Focused explorer");
                                            break;
                                        }
                                        KeyCode::Char('l') => {
                                            // Ctrl+w l: move to right window (outline or file)
                                            if app.outline_open {
                                                app.focus_outline();
                                                app.set_status("Focused outline");
                                            } else {
                                                app.focus_file();
                                                app.set_status("Focused file window");
                                            }
                                            break;
                                        }
                                        KeyCode::Char('j') | KeyCode::Char('k') => {
                                            // Ctrl+w j/k: move to center window (file content)
                                            app.focus_file();
                                            app.set_status("Focused file window");
                                            break;
                                        }
                                        _ => {
                                            // Any other key - cancel
                                            break;
                                        }
                                    }
                                }
                            } else {
                                // Timeout
                                break;
                            }
                        }
                        continue;
                    }

                    // Delegate to mode-specific handlers
                    use crate::app::InputMode;

                    // Handle Search mode globally (including in overlay)
                    if app.input_mode == InputMode::Search {
                        super::search_mode::handle_search_mode(&mut app, key);
                        continue;
                    }

                    // Handle editing overlay input separately
                    if app.editing_entry {
                        super::overlay_mode::handle_overlay_keyboard(&mut app, key);
                        continue;
                    }

                    match app.input_mode {
                        InputMode::Normal => {
                            if super::normal_mode::handle_normal_mode(&mut app, key)? {
                                return Ok(());
                            }
                        }
                        InputMode::Insert => {
                            super::insert_mode::handle_insert_mode(&mut app, key);
                        }
                        InputMode::Command => {
                            if super::command_mode::handle_command_mode(&mut app, key)? {
                                return Ok(());
                            }
                        }
                        InputMode::Search => {
                            super::search_mode::handle_search_mode(&mut app, key);
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    super::mouse::handle_mouse_event(&mut app, mouse, terminal)?;
                }
                Event::Paste(_) => {
                    // Paste events not supported - use 'v' key instead
                }
                _ => {}
            }
        }
    }
}
