use super::{App, UndoState};

impl App {
    pub fn save_undo_state(&mut self) {
        let state = UndoState {
            json_input: self.json_input.clone(),
            markdown_input: self.markdown_input.clone(),
            content_cursor_line: self.content_cursor_line,
            content_cursor_col: self.content_cursor_col,
            scroll: self.scroll,
        };

        self.undo_stack.push(state);

        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }

        // Clear redo stack when new change is made
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current_state = UndoState {
                json_input: self.json_input.clone(),
                markdown_input: self.markdown_input.clone(),
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.redo_stack.push(current_state);

            // Restore previous state
            self.json_input = state.json_input;
            self.markdown_input = state.markdown_input;
            self.content_cursor_line = state.content_cursor_line;
            self.content_cursor_col = state.content_cursor_col;
            self.scroll = state.scroll;

            self.convert_json();
            self.set_status("Undo");
        } else {
            self.set_status("Nothing to undo");
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current_state = UndoState {
                json_input: self.json_input.clone(),
                markdown_input: self.markdown_input.clone(),
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.undo_stack.push(current_state);

            // Restore next state
            self.json_input = state.json_input;
            self.markdown_input = state.markdown_input;
            self.content_cursor_line = state.content_cursor_line;
            self.content_cursor_col = state.content_cursor_col;
            self.scroll = state.scroll;

            self.convert_json();
            self.set_status("Redo");
        } else {
            self.set_status("Nothing to redo");
        }
    }
}
