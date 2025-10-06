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
    Help,
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
    pub showing_help: bool, // Track if help is being shown
    pub scroll: u16,
    pub max_scroll: u16,
    pub status_message: String,
    pub status_time: Option<Instant>,
    pub file_path: Option<PathBuf>,
    pub vim_buffer: String,
    pub format_mode: FormatMode,
    pub previous_format_mode: FormatMode, // Store mode before entering Help
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
    // Filter functionality (View mode only)
    pub filter_pattern: String,
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
            showing_help: false,
            scroll: 0,
            max_scroll: 0,
            status_message: "".to_string(),
            status_time: Some(Instant::now()),
            file_path: None,
            vim_buffer: String::new(),
            format_mode,
            previous_format_mode: format_mode, // Initialize with same mode
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
            filter_pattern: String::new(),
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
            self.relf_entries.clear();
            self.selected_entry_index = 0;
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
            FormatMode::Help => {
                // In Help mode, don't process JSON - help content is set separately
                // This branch should not be reached during normal operation
                return;
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
        Renderer::render_relf(&self.json_input, &self.filter_pattern)
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

    pub fn toggle_help(&mut self) {
        if self.format_mode == FormatMode::Help {
            // Exit help mode - restore to previous mode (View or Edit)
            self.format_mode = self.previous_format_mode;
            self.showing_help = false;
            self.scroll = 0;
            self.convert_json();
        } else {
            // Enter help mode - remember current mode
            self.previous_format_mode = self.format_mode;
            self.format_mode = FormatMode::Help;
            self.showing_help = true;
            self.show_help();
        }
    }

    pub fn show_help(&mut self) {
        // Create help text based on README.md
        self.rendered_content = vec![
            "VIEW MODE CONTROLS".to_string(),
            "".to_string(),
            "Navigation:".to_string(),
            "  j/k or ↑/↓   - select card (or mouse wheel)".to_string(),
            "  gg           - select first card".to_string(),
            "  G            - select last card".to_string(),
            "  :gi          - jump to first INSIDE entry".to_string(),
            "  :go          - jump to first OUTSIDE entry".to_string(),
            "  /            - search forward".to_string(),
            "  n/N          - next/prev match (jumps to card)".to_string(),
            "  :noh         - clear search highlighting".to_string(),
            "".to_string(),
            "Editing:".to_string(),
            "  Enter        - open edit overlay for selected card".to_string(),
            "  :ai          - add new INSIDE entry (jumps to it)".to_string(),
            "  :ao          - add new OUTSIDE entry (jumps to it)".to_string(),
            "  :o           - order entries and auto-save".to_string(),
            "".to_string(),
            "Copy/Paste:".to_string(),
            "  :c           - copy all rendered content (with OUTSIDE/INSIDE headers)".to_string(),
            "  :ci          - copy INSIDE section only".to_string(),
            "  :co          - copy OUTSIDE section only".to_string(),
            "  :cu          - copy URL from selected card".to_string(),
            "  :v           - paste file path or JSON content".to_string(),
            "  :vu          - paste URL from clipboard to selected card".to_string(),
            "  :vi          - paste INSIDE from clipboard (overwrite)".to_string(),
            "  :vo          - paste OUTSIDE from clipboard (overwrite)".to_string(),
            "  :va          - paste both INSIDE and OUTSIDE from clipboard (append)".to_string(),
            "  :vai         - paste INSIDE from clipboard (append)".to_string(),
            "  :vao         - paste OUTSIDE from clipboard (append)".to_string(),
            "  :xi          - clear INSIDE section".to_string(),
            "  :xo          - clear OUTSIDE section".to_string(),
            "".to_string(),
            "Entry Operations:".to_string(),
            "  :dd          - delete selected entry".to_string(),
            "  :yy          - duplicate selected entry".to_string(),
            "".to_string(),
            "Filter (View mode only):".to_string(),
            "  :f pattern   - filter entries by pattern (display only, no save)".to_string(),
            "  :nof         - clear filter".to_string(),
            "".to_string(),
            "Other:".to_string(),
            "  r            - toggle View/Edit mode".to_string(),
            "  :h or ?      - help".to_string(),
            "  q or Esc     - quit".to_string(),
            "".to_string(),
            "EDIT MODE CONTROLS".to_string(),
            "".to_string(),
            "Navigation:".to_string(),
            "  h/j/k/l or arrow keys - move cursor".to_string(),
            "  e            - next word end".to_string(),
            "  b            - previous word start".to_string(),
            "  gg           - jump to top".to_string(),
            "  G            - jump to bottom".to_string(),
            "  :gi          - jump to first INSIDE entry".to_string(),
            "  :go          - jump to first OUTSIDE entry".to_string(),
            "".to_string(),
            "Editing:".to_string(),
            "  i            - enter insert mode".to_string(),
            "  Esc or Ctrl+[ - exit insert mode".to_string(),
            "  u            - undo".to_string(),
            "  Ctrl+r       - redo".to_string(),
            "  g-           - undo".to_string(),
            "  g+           - redo".to_string(),
            "".to_string(),
            "Search:".to_string(),
            "  /            - search forward".to_string(),
            "  n/N          - next/prev match".to_string(),
            "  :noh         - clear search highlighting".to_string(),
            "".to_string(),
            "Commands:".to_string(),
            "  :ai          - add INSIDE entry".to_string(),
            "  :ao          - add OUTSIDE entry".to_string(),
            "  :o           - order entries".to_string(),
            "  :dd          - delete current entry (entire object)".to_string(),
            "  :yy          - duplicate current entry (entire object)".to_string(),
            "  :ci          - copy INSIDE section (JSON format)".to_string(),
            "  :co          - copy OUTSIDE section (JSON format)".to_string(),
            "  :vi          - paste INSIDE from clipboard (overwrite)".to_string(),
            "  :vo          - paste OUTSIDE from clipboard (overwrite)".to_string(),
            "  :va          - paste both INSIDE and OUTSIDE from clipboard (append)".to_string(),
            "  :vai         - paste INSIDE from clipboard (append)".to_string(),
            "  :vao         - paste OUTSIDE from clipboard (append)".to_string(),
            "  :xi          - clear INSIDE section".to_string(),
            "  :xo          - clear OUTSIDE section".to_string(),
            "  :w           - save".to_string(),
            "  :wq          - save and quit".to_string(),
            "  :q           - quit".to_string(),
            "  :e           - reload file".to_string(),
            "  :ar          - toggle auto-reload (default: on)".to_string(),
            "  :f pattern   - filter entries (Edit mode: no effect)".to_string(),
            "  :nof         - clear filter".to_string(),
            "  :h or ?      - help".to_string(),
            "".to_string(),
            "Substitute:".to_string(),
            "  :s/pattern/replacement/     - substitute first occurrence in current line".to_string(),
            "  :s/pattern/replacement/g    - substitute all occurrences in current line".to_string(),
            "  :%s/pattern/replacement/    - substitute first occurrence in all lines".to_string(),
            "  :%s/pattern/replacement/g   - substitute all occurrences in all lines".to_string(),
        ];

        self.relf_line_styles.clear();
        self.relf_visual_styles.clear();
        self.relf_entries.clear();
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

    pub fn apply_filter(&mut self, pattern: String) {
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        self.filter_pattern = pattern.clone();

        // Re-render with filter applied
        self.convert_json();

        let filtered_count = self.relf_entries.len();
        self.set_status(&format!("Filter: {} ({} entries)", pattern, filtered_count));
    }

    pub fn clear_filter(&mut self) {
        if !self.filter_pattern.is_empty() {
            self.filter_pattern.clear();
            self.convert_json();
            self.set_status("Filter cleared");
        }
    }
}
