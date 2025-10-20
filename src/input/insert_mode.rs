use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn handle_insert_mode(app: &mut App, key: KeyEvent) {
    // Check for Ctrl+[ to exit insert mode
    if key.modifiers == KeyModifiers::CONTROL
        && key.code == KeyCode::Char('[')
    {
        app.input_mode = crate::app::InputMode::Normal;
        app.set_status("");
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app.input_mode = crate::app::InputMode::Normal;
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
