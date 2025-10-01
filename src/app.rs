use arboard::Clipboard;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use super::rendering::Renderer;
use super::navigation::Navigator;
use super::json_ops::JsonOperations;


#[derive(Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,  // For vim-style commands like :w, :wq
    Search,   // For vim-style search like /pattern
}

#[derive(Clone, Copy, PartialEq)]
pub enum FormatMode {
    Relf,     // Default mode
    Json,     // For editing
}

pub struct App {
    pub input_mode: InputMode,
    pub json_input: String,
    pub rendered_content: Vec<String>,
    pub previous_content: Vec<String>,  // Store content before showing help
    pub showing_help: bool,  // Track if help is being shown
    pub scroll: u16,
    pub max_scroll: u16,
    pub status_message: String,
    pub status_time: Option<Instant>,
    pub file_path: Option<PathBuf>,
    pub vim_buffer: String,
    pub format_mode: FormatMode,
    pub command_buffer: String,  // For vim commands like :w, :wq
    pub is_modified: bool,       // Track if content has been modified
    pub content_cursor_line: usize,  // Current line in content
    pub content_cursor_col: usize,   // Current column in content line
    pub show_cursor: bool,       // Show/hide cursor in Normal mode
    pub dd_count: usize,         // Count consecutive 'd' presses for dd command
    // Current renderable content width (inner area). Used for accurate wrapping.
    pub content_width: u16,
    // Horizontal scroll offset (used mainly in Relf mode without wrapping)
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
            previous_content: vec![],
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

    pub fn char_index_for_col(&self, s: &str, target_cols: usize) -> usize {
        Renderer::char_index_for_col(s, target_cols)
    }

    pub fn slice_columns(&self, s: &str, start_cols: usize, width_cols: usize) -> String {
        Renderer::slice_columns(s, start_cols, width_cols)
    }

    pub fn convert_json(&mut self) {
        if self.json_input.is_empty() {
            self.rendered_content = vec![];
            return;
        }
        
        // Reset help state when loading new content
        self.showing_help = false;

        match self.format_mode {
            FormatMode::Json => {
                // In JSON mode, always show raw content without any processing
                self.rendered_content = self.render_json();
                self.scroll = 0;
                self.set_status("");
            }
            FormatMode::Relf => {
                // In Relf mode, try to parse JSON directly
                self.rendered_content = self.render_relf();
                self.scroll = 0;
                if self.rendered_content.is_empty() ||
                   (self.rendered_content.len() >= 2 && self.rendered_content[0].contains("Not valid JSON")) {
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


    fn render_relf(&self) -> Vec<String> {
        Renderer::render_relf(&self.json_input)
    }

    fn render_json(&self) -> Vec<String> {
        Renderer::render_json(&self.json_input)
    }

    pub fn load_file(&mut self, path: PathBuf) {
        // Path cleaning - remove all kinds of quotes and whitespace
        let path_display = path.display().to_string();
        let cleaned_path_str = path_display
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches('`')
            .trim();
        
        // Fix common truncation issue: if path starts with "me/" instead of "/home/"
        let final_path_str = if cleaned_path_str.starts_with("me/") {
            let home_path = format!("/ho{}", cleaned_path_str);
            self.set_status(&format!("Fixed truncated path: {} -> {}", cleaned_path_str, home_path));
            home_path
        } else {
            cleaned_path_str.to_string()
        };
        
        let fixed_path = PathBuf::from(final_path_str);
        let final_path_display = fixed_path.display().to_string();
        
        match fs::read_to_string(&fixed_path) {
            Ok(content) => {
                self.json_input = content;


                self.file_path = Some(fixed_path.clone());

                self.set_status(&format!("Loaded: {}", final_path_display));

                self.convert_json();
            }
            Err(e) => {
                self.set_status(&format!("Error loading '{}': {}", final_path_display, e));
            }
        }
    }


    // Relf-mode navigation helpers
    pub fn relf_is_entry_start(&self, line: &str) -> bool { Navigator::relf_is_entry_start(line) }
    pub fn relf_is_boundary(&self, line: &str) -> bool { Navigator::relf_is_boundary(line) }
    pub fn relf_jump_down(&mut self) {
        if self.rendered_content.is_empty() { return; }
        let curr = self.scroll as usize;
        let mut i = curr.saturating_add(1);
        let lim = 12usize.min(self.get_visible_height() as usize); // keep jumps modest
        let mut steps = 0usize;
        // Prefer entry start first
        while i < self.rendered_content.len() && steps < lim {
            if self.relf_is_entry_start(&self.rendered_content[i]) { 
                let max_scroll = self.relf_content_max_scroll();
                self.scroll = (i as u16).min(max_scroll);
                return; 
            }
            i += 1; steps += 1;
        }
        // Fallback to other boundaries (blank/header)
        i = curr.saturating_add(1);
        steps = 0;
        while i < self.rendered_content.len() && steps < lim {
            if self.relf_is_boundary(&self.rendered_content[i]) { 
                let max_scroll = self.relf_content_max_scroll();
                self.scroll = (i as u16).min(max_scroll);
                return; 
            }
            i += 1; steps += 1;
        }
        // Fallback: move down but never beyond the last content page
        let content_max = self.relf_content_max_scroll();
        if self.scroll < content_max { self.scroll += 1; }
    }

    pub fn relf_jump_up(&mut self) {
        if self.rendered_content.is_empty() { return; }
        let lim = 12isize.min(self.get_visible_height() as isize);
        let mut i = self.scroll as isize - 1;
        let mut steps = 0isize;
        // Prefer entry start first
        while i >= 0 && steps < lim {
            if self.relf_is_entry_start(&self.rendered_content[i as usize]) { 
                let max_scroll = self.relf_content_max_scroll();
                let target = i as u16;
                self.scroll = std::cmp::min(target, max_scroll);
                return; 
            }
            i -= 1; steps += 1;
        }
        // Fallback to other boundaries
        i = self.scroll as isize - 1; steps = 0;
        while i >= 0 && steps < lim {
            if self.relf_is_boundary(&self.rendered_content[i as usize]) { 
                let max_scroll = self.relf_content_max_scroll();
                let target = i as u16;
                self.scroll = std::cmp::min(target, max_scroll);
                return; 
            }
            i -= 1; steps += 1;
        }
        self.scroll_up();
    }

    pub fn relf_max_hscroll(&self) -> u16 {
        let w = self.get_content_width() as usize;
        let mut max_cols = 0usize;
        for l in &self.rendered_content {
            let cols = self.display_width_str(l);
            if cols > max_cols { max_cols = cols; }
        }
        if max_cols > w { (max_cols - w) as u16 } else { 0 }
    }

    pub fn relf_content_max_scroll(&self) -> u16 {
        let total = self.rendered_content.len() as u16;
        let vis = self.get_visible_height();
        total.saturating_sub(vis)
    }

    pub fn relf_hscroll_by(&mut self, delta: i16) {
        let max_off = self.relf_max_hscroll();
        if delta >= 0 {
            let d = delta as u16;
            self.hscroll = (self.hscroll.saturating_add(d)).min(max_off);
        } else {
            let d = (-delta) as u16;
            self.hscroll = self.hscroll.saturating_sub(d);
        }
    }

    pub fn paste_from_clipboard(&mut self) {
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(text) => {
                    let trimmed = text.trim();
                    
                    // Check if it's a file path
                    if trimmed.starts_with('/') || trimmed.starts_with("~/") || 
                       trimmed.starts_with("./") || trimmed.starts_with("file://") {
                        // Try to load as file
                        let path = if trimmed.starts_with("file://") {
                            PathBuf::from(trimmed.strip_prefix("file://").unwrap_or(trimmed))
                        } else if trimmed.starts_with("~/") {
                            if let Ok(home) = std::env::var("HOME") {
                                PathBuf::from(trimmed.replacen("~/", &format!("{}/", home), 1))
                            } else {
                                PathBuf::from(trimmed)
                            }
                        } else {
                            PathBuf::from(trimmed)
                        };
                        self.load_file(path);
                    }
                    // Check if it looks like JSON
                    else if trimmed.starts_with('{') || trimmed.starts_with('[') {
                        self.json_input = text;
                        self.set_status("Pasted JSON content");
                        self.convert_json();
                    }
                    // Ignore status messages and other non-JSON text
                    else {
                        self.set_status("Clipboard doesn't contain JSON or file path");
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        if self.rendered_content.is_empty() {
            self.set_status("Nothing to copy");
            return;
        }

        let content = self.rendered_content.join("\n");
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(content) {
                Ok(()) => self.set_status("Copied to clipboard"),
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn clear_content(&mut self) {
        self.json_input.clear();
        self.rendered_content = vec![];
        self.showing_help = false;  // Reset help state


        self.file_path = None;
        self.scroll = 0;
        self.max_scroll = 0;
        self.set_status("");
    }

    pub fn set_status(&mut self, message: &str) {
        // Avoid setting the same status message repeatedly
        if self.status_message == message {
            return;
        }
        
        self.status_message = message.to_string();
        self.status_time = Some(Instant::now());
    }

    pub fn update_status(&mut self) {
        // Keep status messages visible much longer (30 seconds) especially for clipboard operations
        if let Some(time) = self.status_time {
            let timeout = if self.status_message.contains("copied") || self.status_message.contains("Loaded:") {
                Duration::from_secs(30) // Longer for important messages
            } else {
                Duration::from_secs(15) // Still longer for other messages
            };
            
            if time.elapsed() > timeout {
                self.status_message = "".to_string();
                self.status_time = None;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    pub fn page_up(&mut self) {
        // In JSON mode, move the cursor up by a full page of visual lines
        if self.format_mode == FormatMode::Json {
            let count = self.get_visible_height() as usize;
            for _ in 0..count {
                self.move_cursor_up();
            }
        } else {
            self.scroll = self.scroll.saturating_sub(self.get_visible_height());
        }
    }
    
    pub fn page_down(&mut self) {
        // In JSON mode, move the cursor down by a full page of visual lines
        if self.format_mode == FormatMode::Json {
            let count = self.get_visible_height() as usize;
            for _ in 0..count {
                self.move_cursor_down();
            }
        } else {
            let vis = self.get_visible_height();
            self.scroll = (self.scroll + vis).min(self.max_scroll);
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll;
    }

    pub fn handle_vim_input(&mut self, c: char) -> bool {
        self.vim_buffer.push(c);

        if self.vim_buffer == "gg" {
            self.scroll_to_top();
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "dd" {
            // Delete current data entry in JSON mode
            if self.format_mode == FormatMode::Json {
                self.delete_current_entry();
                self.is_modified = true;
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "g-" {
            // Undo (vim-style)
            if self.format_mode == FormatMode::Json {
                self.undo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "g+" {
            // Redo (vim-style)
            if self.format_mode == FormatMode::Json {
                self.redo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer.len() >= 2 {
            self.vim_buffer.clear();
        }

        false
    }

    pub fn delete_current_entry(&mut self) {
        // Save undo state before modification
        self.save_undo_state();

        let lines = self.get_json_lines();
        match JsonOperations::delete_entry_at_cursor(&self.json_input, self.content_cursor_line, &lines) {
            Ok((formatted, message)) => {
                self.json_input = formatted;
                self.convert_json();

                // Adjust cursor position
                let new_lines = self.get_json_lines();
                if self.content_cursor_line >= new_lines.len() && !new_lines.is_empty() {
                    self.content_cursor_line = new_lines.len() - 1;
                }
                self.content_cursor_col = 0;
                self.ensure_cursor_visible();
                self.set_status(&message);
            }
            Err(e) => self.set_status(&e),
        }
    }

    pub fn move_to_next_word_end(&mut self) {
        // Vim-like 'e': always make forward progress to the end of the next word
        let lines = self.get_json_lines();
        if lines.is_empty() { return; }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self.content_cursor_line.min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        // Iterator: advance one position forward from (li, ci)
        let next_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() { return None; }
            // move to next char on the same line
            if ci2 + 1 < line_chars[li2].len() { return Some((li2, ci2 + 1, line_chars[li2][ci2 + 1])); }
            // otherwise, jump to the first char of the next non-empty line
            li2 += 1;
            while li2 < line_chars.len() {
                if !line_chars[li2].is_empty() { return Some((li2, 0, line_chars[li2][0])); }
                li2 += 1;
            }
            None
        };

        // Start scanning strictly after current position to guarantee progress
        let mut in_word = false;
        let mut saw_any_word = false;
        while let Some((nli, nci, ch)) = next_pos(li, ci) {
            if is_word(ch) {
                saw_any_word = true;
                in_word = true; // we are inside a word
            } else if in_word {
                // We just stepped onto a non-word after a run of word chars.
                // li,ci still point to the last word char from the previous iteration.
                self.content_cursor_line = li;
                self.content_cursor_col = ci;
                self.ensure_cursor_visible();
                return;
            }
            // advance current position
            li = nli; ci = nci;
        }

        // Reached EOF: if we were inside a word, li,ci are at the last word char
        if !saw_any_word {
            // No more words: place at last char of file if any
            if let Some(last_line) = line_chars.len().checked_sub(1) {
                let last_col = line_chars[last_line].len();
                self.content_cursor_line = last_line;
                self.content_cursor_col = last_col.saturating_sub(1);
            }
        } else {
            self.content_cursor_line = li;
            self.content_cursor_col = ci;
        }
        self.ensure_cursor_visible();
    }
    
    pub fn move_to_previous_word_start(&mut self) {
        // Vim-like 'b': always make backward progress to the start of the previous word
        let lines = self.get_json_lines();
        if lines.is_empty() { return; }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self.content_cursor_line.min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        let prev_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() { return None; }
            if ci2 > 0 { return Some((li2, ci2 - 1, line_chars[li2][ci2 - 1])); }
            if li2 == 0 { return None; }
            li2 -= 1;
            while let Some(line) = line_chars.get(li2) {
                if !line.is_empty() { return Some((li2, line.len() - 1, line[line.len() - 1])); }
                if li2 == 0 { break; }
                li2 -= 1;
            }
            None
        };

        if li == 0 && ci == 0 { return; }

        // Start scanning strictly before current position to guarantee progress
        let mut in_word = false;
        let mut start_li = li; let mut start_ci = ci; // will hold the start index of the found word
        let mut saw_any_word = false;
        while let Some((pli, pci, ch)) = prev_pos(li, ci) {
            if is_word(ch) {
                saw_any_word = true;
                start_li = pli; start_ci = pci; // keep updating until we leave the word
                in_word = true;
            } else {
                if in_word {
                    // We just left a word while moving left; current saved pos is the word start
                    self.content_cursor_line = start_li;
                    self.content_cursor_col = start_ci;
                    self.ensure_cursor_visible();
                    return;
                }
            }
            li = pli; ci = pci;
        }

        // Reached BOF
        if saw_any_word {
            self.content_cursor_line = start_li;
            self.content_cursor_col = start_ci;
        } else {
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
        }
        self.ensure_cursor_visible();
    }
    
    pub fn show_help(&mut self) {
        if self.showing_help {
            // Return to previous content
            self.rendered_content = self.previous_content.clone();
            self.showing_help = false;
            self.scroll = 0;
            self.set_status("");
        } else {
            // Save current content and show help
            self.previous_content = self.rendered_content.clone();
            self.rendered_content = vec![
                "revw".to_string(),
                "".to_string(),
                "View Mode:".to_string(),
                "  v     - Paste file path or JSON content".to_string(),
                "  c     - Copy rendered content to clipboard".to_string(),
                "  r     - Toggle between Relf (default) and JSON mode".to_string(),
                "  x     - Clear content and status".to_string(),
                "  :h    - Toggle this help".to_string(),
                "  j/k/↑/↓ - Scroll".to_string(),
                "  /     - Search forward".to_string(),
                "  n     - Next search match".to_string(),
                "  N     - Previous search match".to_string(),
                "  q     - Quit".to_string(),
                "".to_string(),
                "JSON Edit Mode:".to_string(),
                "  i     - Insert mode".to_string(),
                "  e     - Move to next word end (like vim)".to_string(),
                "  b     - Move to previous word start (like vim)".to_string(),
                "  dd    - Delete current data entry".to_string(),
                "  u     - Undo".to_string(),
                "  Ctrl+r - Redo".to_string(),
                "  g-    - Undo".to_string(),
                "  g+    - Redo".to_string(),
                "  h/j/k/l - Move cursor (vim-like)".to_string(),
                "  :ai   - Add inside entry at top (date, context)".to_string(),
                "  :ao   - Add outside entry (name, context, url, percentage)".to_string(),
                "  :o    - Order entries (outside by %, inside by date)".to_string(),
                "  :w    - Save file".to_string(),
                "  :wq   - Save and quit".to_string(),
                "  :q    - Quit without saving".to_string(),
                "  :e    - Reload file".to_string(),
                "  :ar   - Toggle auto-reload (default: on)".to_string(),
                "  :h    - Toggle this help".to_string(),
                "  Esc   - Exit insert/command mode".to_string(),
                "".to_string(),
                "Usage:".to_string(),
                "  revw [file.json]     - Open in Relf mode".to_string(),
                "  revw --json [file]   - Open in JSON edit mode".to_string(),
                "  revw --output [file] - Output to file".to_string(),
                "  revw --stdout [file] - Output to stdout".to_string(),
            ];
            self.showing_help = true;
            self.scroll = 0;
            self.set_status("Help (press :h to return)");
        }
    }

    pub fn get_json_lines(&self) -> Vec<String> {
        self.json_input.lines().map(|s| s.to_string()).collect()
    }
    
    pub fn set_json_from_lines(&mut self, lines: Vec<String>) {
        self.json_input = lines.join("\n");
        // In JSON mode, update rendered content directly to preserve raw format
        if self.format_mode == FormatMode::Json {
            self.rendered_content = self.render_json();
        } else {
            self.convert_json();
        }
    }
    
    pub fn insert_char(&mut self, c: char) {
        if self.format_mode == FormatMode::Json {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                lines.push(String::new());
                self.content_cursor_line = 0;
                self.content_cursor_col = 0;
            }

            // Ensure cursor is within bounds
            if self.content_cursor_line >= lines.len() {
                self.content_cursor_line = lines.len().saturating_sub(1);
            }

            let line = &mut lines[self.content_cursor_line];
            let mut chars: Vec<char> = line.chars().collect();
            let pos = self.content_cursor_col.min(chars.len());
            chars.insert(pos, c);

            // Update the line with the new character
            lines[self.content_cursor_line] = chars.into_iter().collect();
            self.content_cursor_col += 1;
            self.set_json_from_lines(lines);
            self.ensure_cursor_visible();
        }
    }
    
    pub fn insert_newline(&mut self) {
        if self.format_mode == FormatMode::Json {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                lines.push(String::new());
                lines.push(String::new());
                self.content_cursor_line = 1;
                self.content_cursor_col = 0;
            } else {
                // Ensure cursor is within bounds
                if self.content_cursor_line >= lines.len() {
                    self.content_cursor_line = lines.len().saturating_sub(1);
                }
                
                let line = lines[self.content_cursor_line].clone();
                let split_pos = self.content_cursor_col.min(line.len());
                let (left, right) = line.split_at(split_pos);
                lines[self.content_cursor_line] = left.to_string();
                lines.insert(self.content_cursor_line + 1, right.to_string());
                self.content_cursor_line += 1;
                self.content_cursor_col = 0;
            }
            self.set_json_from_lines(lines);
            self.ensure_cursor_visible();
        }
    }
    
    pub fn backspace(&mut self) {
        if self.format_mode == FormatMode::Json {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                return;
            }
            if self.content_cursor_col > 0 && self.content_cursor_line < lines.len() {
                // Remove character before cursor (handle multi-byte chars)
                let mut chars: Vec<char> = lines[self.content_cursor_line].chars().collect();
                if self.content_cursor_col > 0 && self.content_cursor_col <= chars.len() {
                    chars.remove(self.content_cursor_col - 1);
                    lines[self.content_cursor_line] = chars.into_iter().collect();
                    self.content_cursor_col -= 1;
                    self.set_json_from_lines(lines);
                }
            } else if self.content_cursor_col == 0 && self.content_cursor_line > 0 {
                // Join with previous line
                let current_line = lines.remove(self.content_cursor_line);
                self.content_cursor_line -= 1;
                let prev_line_len = lines[self.content_cursor_line].chars().count();
                lines[self.content_cursor_line].push_str(&current_line);
                self.content_cursor_col = prev_line_len;
                self.set_json_from_lines(lines);
            }
        }
    }
    
    pub fn delete_char(&mut self) {
        if self.format_mode == FormatMode::Json {
            // Save undo state before modification
            self.save_undo_state();

            let mut lines = self.get_json_lines();
            if lines.is_empty() {
                return;
            }
            if self.content_cursor_line < lines.len() {
                let mut chars: Vec<char> = lines[self.content_cursor_line].chars().collect();
                if self.content_cursor_col < chars.len() {
                    chars.remove(self.content_cursor_col);
                    lines[self.content_cursor_line] = chars.into_iter().collect();
                    self.set_json_from_lines(lines);
                } else if self.content_cursor_line + 1 < lines.len() {
                    // Join with next line
                    let next_line = lines.remove(self.content_cursor_line + 1);
                    lines[self.content_cursor_line].push_str(&next_line);
                    self.set_json_from_lines(lines);
                }
            }
        }
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.content_cursor_col > 0 {
            self.content_cursor_col -= 1;
        } else if self.content_cursor_line > 0 {
            self.content_cursor_line -= 1;
            let lines = self.get_json_lines();
            if self.content_cursor_line < lines.len() {
                self.content_cursor_col = lines[self.content_cursor_line].chars().count();
            }
        }
        self.ensure_cursor_visible();
    }
    
    pub fn move_cursor_right(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }
        if self.content_cursor_line < lines.len() {
            let line_len = lines[self.content_cursor_line].chars().count();
            if self.content_cursor_col < line_len {
                self.content_cursor_col += 1;
            } else if self.content_cursor_line + 1 < lines.len() {
                self.content_cursor_line += 1;
                self.content_cursor_col = 0;
            }
        }
        self.ensure_cursor_visible();
    }
    
    pub fn move_cursor_up(&mut self) {
        if self.content_cursor_line > 0 {
            self.content_cursor_line -= 1;
            let lines = self.get_json_lines();
            if self.content_cursor_line < lines.len() {
                let line_len = lines[self.content_cursor_line].chars().count();
                self.content_cursor_col = self.content_cursor_col.min(line_len);
            }
        }
        self.ensure_cursor_visible();
    }
    
    pub fn move_cursor_down(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }
        
        // Keep cursor within actual content lines
        let content_lines = if self.format_mode == FormatMode::Json {
            lines.len()
        } else {
            self.rendered_content.len()
        };
        
        // Move cursor down if there's content, otherwise just scroll screen
        if self.content_cursor_line + 1 < content_lines {
            // Normal cursor movement within content
            self.content_cursor_line += 1;
            
            let line_len = if self.format_mode == FormatMode::Json && self.content_cursor_line < lines.len() {
                lines[self.content_cursor_line].chars().count()
            } else if self.content_cursor_line < self.rendered_content.len() {
                self.rendered_content[self.content_cursor_line].chars().count()
            } else {
                0
            };
            
            self.content_cursor_col = self.content_cursor_col.min(line_len);
        } else {
            // Cursor is at last line, just scroll the screen down (Mario style)
            let virtual_padding = 10;
            let max_scroll = (content_lines as u16 + virtual_padding).saturating_sub(self.get_visible_height());
            if self.scroll < max_scroll {
                self.scroll += 1;
            }
        }
        self.ensure_cursor_visible();
    }
    
    pub fn ensure_cursor_visible(&mut self) {
        let lines = self.get_json_lines();
        if lines.is_empty() {
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
            return;
        }
        
        // Ensure cursor stays within actual content bounds
        let content_lines = if self.format_mode == FormatMode::Json {
            lines.len()
        } else {
            self.rendered_content.len()
        };
        
        // Keep cursor within actual content lines (not in virtual padding)
        if self.content_cursor_line >= content_lines {
            self.content_cursor_line = content_lines.saturating_sub(1);
        }

        // Handle cursor column bounds
        let line_len = if self.format_mode == FormatMode::Json && self.content_cursor_line < lines.len() {
            lines[self.content_cursor_line].chars().count()
        } else if self.content_cursor_line < self.rendered_content.len() {
            self.rendered_content[self.content_cursor_line].chars().count()
        } else { 0 };
        if self.content_cursor_col > line_len { self.content_cursor_col = line_len; }

        // Vertical scrolling
        let cursor_line = if self.format_mode == FormatMode::Json { self.content_cursor_line as u16 } else { self.calculate_cursor_visual_position().0 };
        let visible_height = self.get_visible_height();
        let scrolloff = 3u16;
        if cursor_line < self.scroll { self.scroll = cursor_line; }
        else if visible_height > 0 && cursor_line >= self.scroll + visible_height { self.scroll = cursor_line.saturating_sub(visible_height - 1); }
        else if visible_height > scrolloff * 2 {
            if cursor_line < self.scroll + scrolloff { self.scroll = cursor_line.saturating_sub(scrolloff); }
            else if cursor_line > self.scroll + visible_height - scrolloff - 1 { self.scroll = cursor_line + scrolloff + 1 - visible_height; }
        }

        // Allow scrolling into virtual padding
        let virtual_padding = 10;
        let max_scroll = (content_lines as u16 + virtual_padding).saturating_sub(visible_height);
        if self.scroll > max_scroll { self.scroll = max_scroll; }

        // Horizontal follow for JSON mode
        if self.format_mode == FormatMode::Json {
            if self.content_cursor_line < lines.len() {
                let current = &lines[self.content_cursor_line];
                let col = self.prefix_display_width(current, self.content_cursor_col) as u16;
                let w = self.get_content_width();
                if col < self.hscroll { self.hscroll = col; }
                else if col >= self.hscroll + w { self.hscroll = col - w + 1; }
            }
        }
    }
    
    pub fn get_visible_height(&self) -> u16 {
        // Use the last measured inner content height from render pass
        if self.visible_height > 0 { self.visible_height } else { 20 }
    }
    
    pub fn get_content_width(&self) -> u16 {
        // Prefer the measured inner content width set during render.
        // Fallback to a reasonable default if unavailable.
        if self.content_width > 2 { self.content_width.saturating_sub(0) } else { 80 }
    }
    
    pub fn calculate_visual_lines(&self, text_line: &str) -> u16 {
        let width = self.get_content_width() as usize;
        Navigator::calculate_visual_lines(text_line, width)
    }

    pub fn build_visual_lines(&self) -> Vec<String> {
        let width = self.get_content_width() as usize;
        if width == 0 {
            return self.rendered_content.clone();
        }
        // In interactive modes, do not wrap: use horizontal pan
        if self.format_mode == FormatMode::Relf || self.format_mode == FormatMode::Json {
            return self.rendered_content.clone();
        }
        // Unused branch currently
        self.rendered_content.clone()
    }
    
    pub fn calculate_cursor_visual_position(&self) -> (u16, u16) {
        let lines = self.get_json_lines();
        let width = self.get_content_width() as usize;
        Navigator::calculate_cursor_visual_position(&lines, self.content_cursor_line, self.content_cursor_col, width)
    }

    pub fn execute_command(&mut self) -> bool {
        let cmd = self.command_buffer.clone();
        let cmd = cmd.trim();
        if cmd == "w" {
            self.save_file();
        } else if cmd == "wq" {
            self.save_file();
            return true; // Signal to quit
        } else if cmd == "q" {
            return true; // Signal to quit
        } else if cmd.starts_with("w ") {
            let filename = cmd.strip_prefix("w ").unwrap().trim().to_string();
            self.save_file_as(&filename);
        } else if cmd.starts_with("wq ") {
            let filename = cmd.strip_prefix("wq ").unwrap().trim().to_string();
            self.save_file_as(&filename);
            return true; // Signal to quit
        } else if cmd == "e" {
            // Refresh/reload the file
            self.reload_file();
        } else if cmd == "ar" {
            // Toggle auto-reload
            self.auto_reload = !self.auto_reload;
            let status = if self.auto_reload { "Auto-reload enabled" } else { "Auto-reload disabled" };
            self.set_status(status);
        } else if cmd == "ai" {
            // Add new inside entry at top
            self.append_inside();
        } else if cmd == "ao" {
            self.append_outside();
        } else if cmd == "o" {
            // Order entries
            self.order_entries();
        } else if cmd == "h" {
            self.show_help();
        } else {
            self.set_status(&format!("Unknown command: {}", cmd));
        }
        false // Don't quit
    }

    pub fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            match fs::write(path, &self.json_input) {
                Ok(()) => {
                    self.is_modified = false;
                    self.last_save_time = Some(Instant::now());
                    self.set_status(&format!("Saved: {}", path.display()));
                }
                Err(e) => {
                    self.set_status(&format!("Error saving: {}", e));
                }
            }
        } else {
            self.set_status("No filename. Use :w filename.json");
        }
    }

    pub fn save_file_as(&mut self, filename: &str) {
        let path = PathBuf::from(filename);
        match fs::write(&path, &self.json_input) {
            Ok(()) => {
                self.file_path = Some(path.clone());
                self.is_modified = false;
                self.last_save_time = Some(Instant::now());
                self.set_status(&format!("Saved: {}", path.display()));
            }
            Err(e) => {
                self.set_status(&format!("Error saving: {}", e));
            }
        }
    }

    pub fn reload_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.json_input = content;
                    self.is_modified = false;
                    self.convert_json();
                    self.content_cursor_line = 0;
                    self.content_cursor_col = 0;
                    self.scroll = 0;
                    self.set_status(&format!("Reloaded: {}", path.display()));
                }
                Err(e) => {
                    self.set_status(&format!("Error reloading: {}", e));
                }
            }
        } else {
            self.set_status("No file to reload");
        }
    }

    pub fn append_inside(&mut self) {
        match JsonOperations::add_inside_entry(&self.json_input) {
            Ok((formatted, line, col, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();
                self.input_mode = InputMode::Insert;
                self.content_cursor_line = line;
                self.content_cursor_col = col;
                self.ensure_cursor_visible();
                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    pub fn append_outside(&mut self) {
        match JsonOperations::add_outside_entry(&self.json_input) {
            Ok((formatted, line, col, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();
                self.input_mode = InputMode::Insert;
                self.content_cursor_line = line;
                self.content_cursor_col = col;
                self.ensure_cursor_visible();
                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    pub fn order_entries(&mut self) {
        match JsonOperations::order_entries(&self.json_input) {
            Ok((formatted, message)) => {
                self.json_input = formatted;
                self.is_modified = true;
                self.convert_json();
                self.set_status(&message);
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    pub fn save_undo_state(&mut self) {
        let state = UndoState {
            json_input: self.json_input.clone(),
            content_cursor_line: self.content_cursor_line,
            content_cursor_col: self.content_cursor_col,
            scroll: self.scroll,
        };
        self.undo_stack.push(state);
        // Clear redo stack when new change is made
        self.redo_stack.clear();

        // Limit undo stack size to 100 states
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            // Save current state to redo stack
            let redo_state = UndoState {
                json_input: self.json_input.clone(),
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.redo_stack.push(redo_state);

            // Restore previous state
            self.json_input = state.json_input;
            self.content_cursor_line = state.content_cursor_line;
            self.content_cursor_col = state.content_cursor_col;
            self.scroll = state.scroll;
            self.convert_json();
            self.set_status("Undo");
        } else {
            self.set_status("Already at oldest change");
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            // Save current state to undo stack
            let undo_state = UndoState {
                json_input: self.json_input.clone(),
                content_cursor_line: self.content_cursor_line,
                content_cursor_col: self.content_cursor_col,
                scroll: self.scroll,
            };
            self.undo_stack.push(undo_state);

            // Restore next state
            self.json_input = state.json_input;
            self.content_cursor_line = state.content_cursor_line;
            self.content_cursor_col = state.content_cursor_col;
            self.scroll = state.scroll;
            self.convert_json();
            self.set_status("Redo");
        } else {
            self.set_status("Already at newest change");
        }
    }

    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_buffer.clear();
    }

    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            self.input_mode = InputMode::Normal;
            return;
        }

        self.search_query = self.search_buffer.clone();
        self.find_matches();
        self.input_mode = InputMode::Normal;

        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.jump_to_current_match();
            self.set_status(&format!("Found {} matches for '{}'", self.search_matches.len(), self.search_query));
        } else {
            self.current_match_index = None;
            self.set_status(&format!("Pattern not found: {}", self.search_query));
        }
    }

    pub fn find_matches(&mut self) {
        self.search_matches.clear();
        
        let search_content = if self.format_mode == FormatMode::Json {
            &self.get_json_lines()
        } else {
            &self.rendered_content
        };

        let query_lower = self.search_query.to_lowercase();
        
        for (line_idx, line) in search_content.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                let actual_pos = start + pos;
                self.search_matches.push((line_idx, actual_pos));
                start = actual_pos + 1;
            }
        }
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(&format!("No matches for '{}'", self.search_query));
            }
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let next_idx = if current_idx + 1 >= self.search_matches.len() {
            0 // Wrap to beginning
        } else {
            current_idx + 1
        };
        
        self.current_match_index = Some(next_idx);
        self.jump_to_current_match();
        self.set_status(&format!("Match {} of {} for '{}'", next_idx + 1, self.search_matches.len(), self.search_query));
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(&format!("No matches for '{}'", self.search_query));
            }
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.search_matches.len() - 1 // Wrap to end
        } else {
            current_idx - 1
        };
        
        self.current_match_index = Some(prev_idx);
        self.jump_to_current_match();
        self.set_status(&format!("Match {} of {} for '{}'", prev_idx + 1, self.search_matches.len(), self.search_query));
    }

    pub fn jump_to_current_match(&mut self) {
        if let Some(match_idx) = self.current_match_index {
            if let Some(&(line, col)) = self.search_matches.get(match_idx) {
                if self.format_mode == FormatMode::Json {
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                } else {
                    // For Relf mode, just scroll to the line
                    self.scroll = line as u16;
                    let max_scroll = self.rendered_content.len().saturating_sub(self.get_visible_height() as usize) as u16;
                    if self.scroll > max_scroll {
                        self.scroll = max_scroll;
                    }
                }
                self.ensure_cursor_visible();
            }
        }
    }

}

