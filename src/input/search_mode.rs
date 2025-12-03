use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

pub fn handle_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = crate::app::InputMode::Normal;
            app.search_buffer.clear();
            app.search_history_index = None;
            app.set_status("");
        }
        KeyCode::Enter => {
            // Add to history before executing
            app.add_to_search_history(app.search_buffer.clone());

            if app.editing_entry {
                // In overlay: just save to history and jump to first match
                app.input_mode = crate::app::InputMode::Normal;
                app.overlay_next_match();
                app.set_status("");
            } else {
                app.execute_search();
            }
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
                app.input_mode = crate::app::InputMode::Normal;
                app.search_history_index = None;
                app.set_status("");
            }
        }
        _ => {}
    }
}
