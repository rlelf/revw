use super::super::navigation::Navigator;
use super::super::rendering::{RelfEntry, RelfLineStyle, RelfRenderResult, Renderer};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Command, // For vim-style commands like :w, :wq
    Search,  // For vim-style search like /pattern
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FormatMode {
    View,
    Edit,
}

#[derive(Clone)]
pub struct SubstituteMatch {
    pub line: usize,
    pub col: usize,
    pub pattern: String,
    pub replacement: String,
}

pub struct App {
    pub input_mode: InputMode,
    pub json_input: String,
    pub rendered_content: Vec<String>,
    pub relf_line_styles: Vec<RelfLineStyle>,
    pub relf_visual_styles: Vec<RelfLineStyle>,
    pub relf_entries: Vec<RelfEntry>,
    pub selected_entry_index: usize, // Currently selected entry in View mode
    pub editing_entry: bool, // Whether we're editing entry in overlay
    pub edit_buffer: Vec<String>, // Buffer for editing entry fields
    pub edit_field_index: usize, // Which field is being edited
    pub edit_field_editing_mode: bool, // Whether editing within a field (Enter pressed)
    pub edit_insert_mode: bool, // Whether in insert mode within overlay
    pub edit_cursor_pos: usize, // Cursor position within current field
    pub previous_content: Vec<String>, // Store content before showing help
    pub previous_relf_styles: Vec<RelfLineStyle>,
    pub previous_relf_visual_styles: Vec<RelfLineStyle>,
    pub showing_help: bool, // Track if help is being shown
    pub scroll: u16,
    pub max_scroll: u16,
    pub status_message: String,
    pub status_time: Option<Instant>,
    pub file_path: Option<PathBuf>,
    pub vim_buffer: String,
    pub format_mode: FormatMode,
    pub command_buffer: String,     // For vim commands like :w, :wq
    pub is_modified: bool,          // Track if content has been modified
    pub content_cursor_line: usize, // Current line in content
    pub content_cursor_col: usize,  // Current column in content line
    pub show_cursor: bool,          // Show/hide cursor in Normal mode
    pub dd_count: usize,            // Count consecutive 'd' presses for dd command
    // Current renderable content width (inner area). Used for accurate wrapping.
    pub content_width: u16,
    // Horizontal scroll offset (used mainly in View mode without wrapping)
    pub hscroll: u16,
    // Last measured visible height of content area
    pub visible_height: u16,
    // Search functionality
    pub search_query: String,
    pub search_buffer: String,
    pub search_matches: Vec<(usize, usize)>, // (line, col) positions
    pub current_match_index: Option<usize>,
    // Undo/Redo functionality
    pub undo_stack: Vec<UndoState>,
    pub redo_stack: Vec<UndoState>,
    // Auto-reload functionality
    pub auto_reload: bool,
    pub last_save_time: Option<Instant>,
    // Scrollbar interaction state
    pub dragging_scrollbar: Option<ScrollbarType>,
    // Substitute confirmation state
    pub substitute_confirmations: Vec<SubstituteMatch>,
    pub current_substitute_index: usize,
    // Double-click detection
    pub last_click_time: Option<Instant>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ScrollbarType {
    Vertical,
    Horizontal,
}

#[derive(Clone)]
pub struct UndoState {
    pub json_input: String,
    pub content_cursor_line: usize,
    pub content_cursor_col: usize,
    pub scroll: u16,
}

impl App {
    pub fn new(format_mode: FormatMode) -> Self {
        let app = Self {
            input_mode: InputMode::Normal,
            json_input: String::new(),
            rendered_content: vec![],
            relf_line_styles: Vec::new(),
            relf_visual_styles: Vec::new(),
            relf_entries: Vec::new(),
            selected_entry_index: 0,
            editing_entry: false,
            edit_buffer: Vec::new(),
            edit_field_index: 0,
            edit_field_editing_mode: false,
            edit_insert_mode: false,
            edit_cursor_pos: 0,
            previous_content: vec![],
            previous_relf_styles: Vec::new(),
            previous_relf_visual_styles: Vec::new(),
            showing_help: false,
            scroll: 0,
            max_scroll: 0,
            status_message: "".to_string(),
            status_time: Some(Instant::now()),
            file_path: None,
            vim_buffer: String::new(),
            format_mode,
            command_buffer: String::new(),
            is_modified: false,
            content_cursor_line: 0,
            content_cursor_col: 0,
            show_cursor: true,
            dd_count: 0,
            content_width: 80,
            hscroll: 0,
            visible_height: 20,
            search_query: String::new(),
            search_buffer: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            auto_reload: true,
            last_save_time: None,
            dragging_scrollbar: None,
            substitute_confirmations: Vec::new(),
            current_substitute_index: 0,
            last_click_time: None,
        };

        app
    }

    // --- Display width helpers (unicode-aware) ---
    pub fn display_width_str(&self, s: &str) -> usize {
        Renderer::display_width_str(s)
    }

    pub fn prefix_display_width(&self, s: &str, char_pos: usize) -> usize {
        Renderer::prefix_display_width(s, char_pos)
    }

    pub fn slice_columns(&self, s: &str, start_cols: usize, width_cols: usize) -> String {
        Renderer::slice_columns(s, start_cols, width_cols)
    }

    pub fn convert_json(&mut self) {
        if self.json_input.is_empty() {
            self.rendered_content = vec![];
            self.relf_line_styles.clear();
            self.relf_visual_styles.clear();
            return;
        }

        // Reset help state when loading new content
        self.showing_help = false;

        match self.format_mode {
            FormatMode::Edit => {
                // In Edit mode, always show raw content without any processing
                self.rendered_content = self.render_json();
                self.relf_line_styles.clear();
                self.relf_visual_styles.clear();
                self.scroll = 0;
                self.set_status("");
            }
            FormatMode::View => {
                // In View mode, try to parse JSON directly
                let relf = self.render_relf();
                self.rendered_content = relf.lines;
                self.relf_line_styles = relf.styles;
                self.relf_entries = relf.entries;
                self.relf_visual_styles.clear();
                self.scroll = 0;
                // Keep selected_entry_index, but ensure it's within bounds
                if self.selected_entry_index >= self.relf_entries.len() && !self.relf_entries.is_empty() {
                    self.selected_entry_index = self.relf_entries.len() - 1;
                } else if self.relf_entries.is_empty() {
                    self.selected_entry_index = 0;
                }

                // Check if we have valid entries (new card-based rendering)
                if !self.relf_entries.is_empty() {
                    self.set_status("");
                } else if self.rendered_content.is_empty()
                    || (self.rendered_content.len() >= 2
                        && self.rendered_content[0].contains("Not valid JSON"))
                {
                    // Only set status if not already showing this message
                    if !self.status_message.contains("Not a JSON file") {
                        self.set_status("Not a JSON file - showing as text");
                    }
                } else {
                    self.set_status("");
                }
            }
        }
    }

    fn render_relf(&self) -> RelfRenderResult {
        Renderer::render_relf(&self.json_input)
    }

    fn render_json(&self) -> Vec<String> {
        Renderer::render_json(&self.json_input)
    }

    pub fn set_status(&mut self, message: &str) {
        if message.is_empty() {
            self.status_message = String::new();
            self.status_time = None;
        } else {
            self.status_message = message.to_string();
            self.status_time = Some(Instant::now());
        }
    }

    pub fn update_status(&mut self) {
        // Clear status message after 3 seconds
        if let Some(time) = self.status_time {
            if time.elapsed() > Duration::from_secs(3) {
                self.status_message = String::new();
                self.status_time = None;
            }
        }
    }

    pub fn get_json_lines(&self) -> Vec<String> {
        self.json_input.lines().map(|s| s.to_string()).collect()
    }

    pub fn set_json_from_lines(&mut self, lines: Vec<String>) {
        self.json_input = lines.join("\n");
        // Ensure trailing newline is preserved
        if !self.json_input.is_empty() && !self.json_input.ends_with('\n') {
            self.json_input.push('\n');
        }
        self.convert_json();
    }

    pub fn get_visible_height(&self) -> u16 {
        // Return the last measured visible height
        // (updated by UI code before rendering)
        self.visible_height.max(1)
    }

    pub fn get_content_width(&self) -> u16 {
        // Return the current content width
        // (updated by UI code before rendering based on current viewport)
        self.content_width.max(1)
    }

    pub fn calculate_visual_lines(&self, text_line: &str) -> u16 {
        Navigator::calculate_visual_lines(text_line, self.get_content_width() as usize)
    }

    pub fn build_visual_lines(&mut self) -> Vec<String> {
        // Simply return content as-is for now, wrapping will be handled by UI
        if self.showing_help {
            return self.previous_content.clone();
        }
        self.rendered_content.clone()
    }

    pub fn calculate_cursor_visual_position(&self) -> (u16, u16) {
        // Calculate the visual position (row, col) of the cursor based on:
        // - content_cursor_line: the logical line index
        // - content_cursor_col: the character position within that line
        // This function considers text wrapping when width is limited.

        let _width = self.get_content_width() as usize;
        let lines = self.get_json_lines();

        let mut visual_row = 0u16;

        // Sum up all visual lines before the cursor line
        for i in 0..self.content_cursor_line.min(lines.len()) {
            visual_row += self.calculate_visual_lines(&lines[i]);
        }

        // Now handle the cursor line itself
        if self.content_cursor_line < lines.len() {
            let line = &lines[self.content_cursor_line];
            let col_in_chars = self.content_cursor_col.min(line.chars().count());
            let prefix = line.chars().take(col_in_chars).collect::<String>();
            let prefix_width = Renderer::display_width_str(&prefix);

            // Simplified: assume no wrapping for now
            let visual_col = prefix_width as u16;
            return (visual_row, visual_col);
        }

        (visual_row, 0)
    }

    pub fn show_help(&mut self) {
        // Store current content before showing help
        self.previous_content = self.rendered_content.clone();
        self.previous_relf_styles = self.relf_line_styles.clone();
        self.previous_relf_visual_styles = self.relf_visual_styles.clone();
        self.showing_help = true;

        // Create help text
        self.rendered_content = vec![
            "╭─────────────────────────────────────────────────────────────────╮".to_string(),
            "│                         NAVIGATION                              │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ j/k       : Scroll down/up                                      │".to_string(),
            "│ h/l       : Scroll left/right (View mode)                       │".to_string(),
            "│ Ctrl+d/u  : Page down/up                                        │".to_string(),
            "│ g/G       : Jump to top/bottom                                  │".to_string(),
            "│ J/K       : Jump to next/prev entry boundary (View mode)        │".to_string(),
            "│ n/N       : Next/previous search match                          │".to_string(),
            "│ C-o/C-i   : Jump to first OUTSIDE/INSIDE entry (View mode)      │".to_string(),
            "│                                                                 │".to_string(),
            "│                         EDITING                                 │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ i         : Enter insert mode                                   │".to_string(),
            "│ a         : Append after cursor                                 │".to_string(),
            "│ o/O       : Insert new line below/above                         │".to_string(),
            "│ dd        : Delete current line                                 │".to_string(),
            "│ x         : Delete current entry (View mode)                    │".to_string(),
            "│ u         : Undo                                                │".to_string(),
            "│ Ctrl+r    : Redo                                                │".to_string(),
            "│ ESC       : Return to normal mode                               │".to_string(),
            "│                                                                 │".to_string(),
            "│                         SEARCH                                  │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ /         : Start search                                        │".to_string(),
            "│ n/N       : Next/previous match                                 │".to_string(),
            "│ ESC       : Clear search highlight                              │".to_string(),
            "│                                                                 │".to_string(),
            "│                         COMMANDS                                │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ :w        : Save file                                           │".to_string(),
            "│ :w <file> : Save as new file                                    │".to_string(),
            "│ :q        : Quit (warns if unsaved)                             │".to_string(),
            "│ :q!       : Force quit without saving                           │".to_string(),
            "│ :wq       : Save and quit                                       │".to_string(),
            "│ :e        : Reload file                                         │".to_string(),
            "│ :s/<pat>/<rep>/[g]  : Substitute pattern (g for all on line)    │".to_string(),
            "│ :%s/<pat>/<rep>/[g] : Substitute in entire file                 │".to_string(),
            "│ :s/<pat>/<rep>/gc   : Substitute with confirmation              │".to_string(),
            "│                                                                 │".to_string(),
            "│                         CLIPBOARD                               │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ y         : Copy all content to clipboard                       │".to_string(),
            "│ yi        : Copy INSIDE section                                 │".to_string(),
            "│ yo        : Copy OUTSIDE section                                │".to_string(),
            "│ yu        : Copy selected URL (View mode)                       │".to_string(),
            "│ p         : Paste from clipboard                                │".to_string(),
            "│ pi        : Paste INSIDE section (overwrite)                    │".to_string(),
            "│ po        : Paste OUTSIDE section (overwrite)                   │".to_string(),
            "│ Pi        : Paste INSIDE section (append)                       │".to_string(),
            "│ Po        : Paste OUTSIDE section (append)                      │".to_string(),
            "│ P         : Paste both sections (append)                        │".to_string(),
            "│ pu        : Paste URL to selected entry (View mode)             │".to_string(),
            "│ ci        : Clear INSIDE section                                │".to_string(),
            "│ co        : Clear OUTSIDE section                               │".to_string(),
            "│ cc        : Clear all content                                   │".to_string(),
            "│                                                                 │".to_string(),
            "│                         VIEW MODE                               │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ Enter     : Open entry editor overlay                           │".to_string(),
            "│ ai        : Append new entry to INSIDE                          │".to_string(),
            "│ ao        : Append new entry to OUTSIDE                         │".to_string(),
            "│ s         : Sort entries by percentage                          │".to_string(),
            "│                                                                 │".to_string(),
            "│                         OTHER                                   │".to_string(),
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
            "│ ?         : Toggle this help                                    │".to_string(),
            "│ Tab       : Toggle View/Edit mode                               │".to_string(),
            "╰─────────────────────────────────────────────────────────────────╯".to_string(),
        ];

        self.relf_line_styles.clear();
        self.relf_visual_styles.clear();
        self.scroll = 0;
    }

    pub fn save_undo_state(&mut self) {
        let state = UndoState {
            json_input: self.json_input.clone(),
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
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.redo_stack.push(current_state);

            // Restore previous state
            self.json_input = state.json_input;
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
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.undo_stack.push(current_state);

            // Restore next state
            self.json_input = state.json_input;
            self.content_cursor_line = state.content_cursor_line;
            self.content_cursor_col = state.content_cursor_col;
            self.scroll = state.scroll;

            self.convert_json();
            self.set_status("Redo");
        } else {
            self.set_status("Nothing to redo");
        }
    }

    pub fn clear_content(&mut self) {
        self.save_undo_state();
        self.json_input = String::new();
        self.content_cursor_line = 0;
        self.content_cursor_col = 0;
        self.scroll = 0;
        self.is_modified = true;
        self.convert_json();
        self.set_status("Content cleared");
    }
}
