use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

pub fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = crate::app::InputMode::Normal;
            app.command_buffer.clear();
            app.command_history_index = None;
            app.set_status("");
        }
        KeyCode::Tab => {
            // Tab completion for commands
            app.complete_command();
        }
        KeyCode::Enter => {
            // Add to history before executing
            app.add_to_command_history(app.command_buffer.clone());

            if app.execute_command() {
                return Ok(true); // Quit the application
            }
            app.input_mode = crate::app::InputMode::Normal;
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
            app.reset_completion(); // Reset completion on manual input
            app.set_status(&format!(":{}", app.command_buffer));
        }
        KeyCode::Backspace => {
            if !app.command_buffer.is_empty() {
                app.command_buffer.pop();
                app.command_history_index = None;
                app.reset_completion(); // Reset completion on backspace
                app.set_status(&format!(":{}", app.command_buffer));
            } else {
                // Exit command mode when backspace on empty buffer
                app.input_mode = crate::app::InputMode::Normal;
                app.command_history_index = None;
                app.set_status("");
            }
        }
        _ => {}
    }
    Ok(false)
}
