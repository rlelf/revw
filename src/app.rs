use super::json_ops::JsonOperations;
use super::navigation::Navigator;
use super::rendering::{RelfEntry, RelfLineStyle, RelfRenderResult, Renderer};
use arboard::Clipboard;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use unicode_width::UnicodeWidthChar;

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
            self.set_status(&format!(
                "Fixed truncated path: {} -> {}",
                cleaned_path_str, home_path
            ));
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
    pub fn relf_is_entry_start(&self, line: &str) -> bool {
        Navigator::relf_is_entry_start(line)
    }
    pub fn relf_is_boundary(&self, line: &str) -> bool {
        Navigator::relf_is_boundary(line)
    }
    pub fn relf_jump_down(&mut self) {
        if self.rendered_content.is_empty() {
            return;
        }
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
            i += 1;
            steps += 1;
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
            i += 1;
            steps += 1;
        }
        // Fallback: move down but never beyond the last content page
        let content_max = self.relf_content_max_scroll();
        if self.scroll < content_max {
            self.scroll += 1;
        }
    }

    pub fn relf_jump_up(&mut self) {
        if self.rendered_content.is_empty() {
            return;
        }
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
            i -= 1;
            steps += 1;
        }
        // Fallback to other boundaries
        i = self.scroll as isize - 1;
        steps = 0;
        while i >= 0 && steps < lim {
            if self.relf_is_boundary(&self.rendered_content[i as usize]) {
                let max_scroll = self.relf_content_max_scroll();
                let target = i as u16;
                self.scroll = std::cmp::min(target, max_scroll);
                return;
            }
            i -= 1;
            steps += 1;
        }
        self.scroll_up();
    }

    pub fn relf_max_hscroll(&self) -> u16 {
        let w = self.get_content_width() as usize;
        let mut max_cols = 0usize;
        for l in &self.rendered_content {
            let cols = self.display_width_str(l);
            if cols > max_cols {
                max_cols = cols;
            }
        }
        if max_cols > w {
            (max_cols - w) as u16
        } else {
            0
        }
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
                    if trimmed.starts_with('/')
                        || trimmed.starts_with("~/")
                        || trimmed.starts_with("./")
                        || trimmed.starts_with("file://")
                    {
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
        // In View mode with cards, copy all entries with OUTSIDE/INSIDE sections
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    let mut all_content = Vec::new();

                    // Add OUTSIDE section
                    if outside_count > 0 {
                        all_content.push("OUTSIDE".to_string());
                        all_content.push(String::new());

                        for (i, entry) in self.relf_entries.iter().enumerate() {
                            if i < outside_count {
                                if i > 0 {
                                    all_content.push(String::new());
                                }
                                for line in &entry.lines {
                                    all_content.push(line.clone());
                                }
                            }
                        }

                        all_content.push(String::new());
                    }

                    // Add INSIDE section
                    let inside_count = self.relf_entries.len() - outside_count;
                    if inside_count > 0 {
                        all_content.push("INSIDE".to_string());
                        all_content.push(String::new());

                        for (i, entry) in self.relf_entries.iter().enumerate() {
                            if i >= outside_count {
                                if i > outside_count {
                                    all_content.push(String::new());
                                }
                                for line in &entry.lines {
                                    all_content.push(line.clone());
                                }
                            }
                        }
                    }

                    if all_content.is_empty() {
                        self.set_status("Nothing to copy");
                        return;
                    }

                    let content = all_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
        }

        // Fallback to rendered_content
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

    pub fn copy_inside_data(&mut self) {
        // In view mode, copy all INSIDE entries from relf_entries
        if self.format_mode == FormatMode::View {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    // Collect INSIDE entries (indices >= outside_count)
                    let mut inside_content = Vec::new();
                    inside_content.push("INSIDE".to_string());
                    inside_content.push(String::new());

                    for (i, entry) in self.relf_entries.iter().enumerate() {
                        if i >= outside_count {
                            // Add blank line between entries (but not before first entry)
                            if i > outside_count {
                                inside_content.push(String::new());
                            }
                            for line in &entry.lines {
                                inside_content.push(line.clone());
                            }
                        }
                    }

                    if inside_content.is_empty() {
                        self.set_status("No INSIDE entries found");
                        return;
                    }

                    let content = inside_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied INSIDE section to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
            self.set_status("Failed to parse JSON");
            return;
        }

        // In Edit mode, copy with "inside: [...]" wrapper
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(json_value) => {
                if let Some(obj) = json_value.as_object() {
                    if let Some(inside) = obj.get("inside") {
                        // Create wrapper object with "inside" key
                        let mut wrapper = serde_json::Map::new();
                        wrapper.insert("inside".to_string(), inside.clone());
                        let wrapper_value = Value::Object(wrapper);

                        match serde_json::to_string_pretty(&wrapper_value) {
                            Ok(formatted) => match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(formatted) {
                                    Ok(()) => self.set_status("Copied inside data to clipboard"),
                                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                                },
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => {
                                self.set_status(&format!("Error formatting inside data: {}", e))
                            }
                        }
                    } else {
                        self.set_status("No 'inside' field found");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn start_editing_entry(&mut self) {
        // Load fields from JSON (not from rendered lines) to include empty fields
        if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
            if let Some(obj) = json_value.as_object() {
                let mut current_idx = 0;
                let target_idx = self.selected_entry_index;

                // Check outside section
                if let Some(outside) = obj.get("outside") {
                    if let Some(outside_array) = outside.as_array() {
                        if target_idx < current_idx + outside_array.len() {
                            let local_idx = target_idx - current_idx;
                            if let Some(entry_obj) = outside_array[local_idx].as_object() {
                                // Load all fields including empty ones, use placeholder if empty
                                let name = entry_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let context = entry_obj.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let url = entry_obj.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let percentage = entry_obj.get("percentage").and_then(|v| v.as_i64()).unwrap_or(0);

                                self.edit_buffer = vec![
                                    if name.is_empty() { "name".to_string() } else { name },
                                    if context.is_empty() { "context".to_string() } else { context },
                                    if url.is_empty() { "url".to_string() } else { url },
                                    if percentage == 0 && !entry_obj.contains_key("percentage") { "percentage".to_string() } else { percentage.to_string() },
                                ];
                                self.edit_field_index = 0;
                                self.editing_entry = true;
                                self.edit_field_editing_mode = false;
                                self.edit_insert_mode = false;
                                self.edit_cursor_pos = 0;
                                return;
                            }
                        }
                        current_idx += outside_array.len();
                    }
                }

                // Check inside section
                if let Some(inside) = obj.get("inside") {
                    if let Some(inside_array) = inside.as_array() {
                        if target_idx < current_idx + inside_array.len() {
                            let local_idx = target_idx - current_idx;
                            if let Some(entry_obj) = inside_array[local_idx].as_object() {
                                // Load all fields including empty ones, use placeholder if empty
                                let date = entry_obj.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let context = entry_obj.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                self.edit_buffer = vec![
                                    if date.is_empty() { "date".to_string() } else { date },
                                    if context.is_empty() { "context".to_string() } else { context },
                                ];
                                self.edit_field_index = 0;
                                self.editing_entry = true;
                                self.edit_field_editing_mode = false;
                                self.edit_insert_mode = false;
                                self.edit_cursor_pos = 0;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save_edited_entry(&mut self) {
        // Save the edited entry back to JSON
        if self.edit_buffer.is_empty() {
            self.editing_entry = false;
            return;
        }

        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut json_value) => {
                if let Some(obj) = json_value.as_object_mut() {
                    let mut current_idx = 0;
                    let target_idx = self.selected_entry_index;
                    let mut found = false;

                    // Check outside section
                    if let Some(outside) = obj.get_mut("outside") {
                        if let Some(outside_array) = outside.as_array_mut() {
                            if target_idx < current_idx + outside_array.len() {
                                let local_idx = target_idx - current_idx;
                                if let Some(entry_obj) = outside_array[local_idx].as_object_mut() {
                                    // Update fields
                                    if self.edit_buffer.len() >= 1 {
                                        entry_obj.insert("name".to_string(), Value::String(self.edit_buffer[0].clone()));
                                    }
                                    if self.edit_buffer.len() >= 2 {
                                        entry_obj.insert("context".to_string(), Value::String(self.edit_buffer[1].clone()));
                                    }
                                    if self.edit_buffer.len() >= 3 {
                                        entry_obj.insert("url".to_string(), Value::String(self.edit_buffer[2].clone()));
                                    }
                                    if self.edit_buffer.len() >= 4 {
                                        // Parse percentage
                                        if let Ok(pct) = self.edit_buffer[3].trim_end_matches('%').parse::<i64>() {
                                            entry_obj.insert("percentage".to_string(), Value::Number(pct.into()));
                                        }
                                    }
                                    found = true;
                                }
                            } else {
                                current_idx += outside_array.len();
                            }
                        }
                    }

                    // Check inside section
                    if !found {
                        if let Some(inside) = obj.get_mut("inside") {
                            if let Some(inside_array) = inside.as_array_mut() {
                                let local_idx = target_idx - current_idx;
                                if local_idx < inside_array.len() {
                                    if let Some(entry_obj) = inside_array[local_idx].as_object_mut() {
                                        // Update fields (date and context for inside)
                                        if self.edit_buffer.len() >= 1 {
                                            entry_obj.insert("date".to_string(), Value::String(self.edit_buffer[0].clone()));
                                        }
                                        if self.edit_buffer.len() >= 2 {
                                            entry_obj.insert("context".to_string(), Value::String(self.edit_buffer[1].clone()));
                                        }
                                        found = true;
                                    }
                                }
                            }
                        }
                    }

                    if found {
                        match serde_json::to_string_pretty(&json_value) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.is_modified = true;
                                self.convert_json();
                                self.set_status("Entry updated");
                                // Auto-save after editing
                                self.save_file();
                            }
                            Err(e) => self.set_status(&format!("Error formatting JSON: {}", e)),
                        }
                    }
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }

        self.editing_entry = false;
    }

    pub fn cancel_editing_entry(&mut self) {
        self.editing_entry = false;
        self.edit_buffer.clear();
        self.edit_field_index = 0;
        self.edit_insert_mode = false;
        self.edit_cursor_pos = 0;
    }

    pub fn copy_selected_url(&mut self) {
        // Copy URL from selected entry in Relf card mode
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            if let Some(entry) = self.relf_entries.get(self.selected_entry_index) {
                // Find URL in entry lines (usually starts with "http")
                let url = entry.lines.iter().find(|line| line.starts_with("http"));

                if let Some(url_str) = url {
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(url_str.clone()) {
                            Ok(()) => self.set_status(&format!("Copied URL: {}", url_str)),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                } else {
                    self.set_status("No URL found in selected entry");
                }
            } else {
                self.set_status("No entry selected");
            }
            return;
        }

        self.set_status("Not in card view mode");
    }

    pub fn paste_inside_overwrite(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "inside" array from clipboard
                            let new_inside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("inside").cloned()
                            } else {
                                None
                            };

                            if let Some(new_inside) = new_inside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Overwrite inside
                                            obj.insert("inside".to_string(), new_inside);

                                            // Format and save
                                            match serde_json::to_string_pretty(&current_json) {
                                                Ok(formatted) => {
                                                    self.json_input = formatted;
                                                    self.is_modified = true;
                                                    self.convert_json();
                                                    self.set_status("INSIDE section overwritten from clipboard");
                                                }
                                                Err(e) => self.set_status(&format!("Format error: {}", e)),
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'inside' field in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_outside_overwrite(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "outside" array from clipboard
                            let new_outside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("outside").cloned()
                            } else {
                                None
                            };

                            if let Some(new_outside) = new_outside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Overwrite outside
                                            obj.insert("outside".to_string(), new_outside);

                                            // Format and save
                                            match serde_json::to_string_pretty(&current_json) {
                                                Ok(formatted) => {
                                                    self.json_input = formatted;
                                                    self.is_modified = true;
                                                    self.convert_json();
                                                    self.set_status("OUTSIDE section overwritten from clipboard");
                                                }
                                                Err(e) => self.set_status(&format!("Format error: {}", e)),
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'outside' field in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_inside_append(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "inside" array from clipboard
                            let new_inside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("inside").and_then(|v| v.as_array()).cloned()
                            } else {
                                None
                            };

                            if let Some(new_inside_items) = new_inside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Get or create inside array
                                            let inside_array = obj.entry("inside".to_string())
                                                .or_insert(Value::Array(vec![]));

                                            if let Some(arr) = inside_array.as_array_mut() {
                                                // Append new items
                                                for item in new_inside_items {
                                                    arr.push(item);
                                                }

                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.convert_json();
                                                        self.set_status("INSIDE entries appended from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current 'inside' is not an array");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'inside' array in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_outside_append(&mut self) {
        // Get clipboard content
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&clipboard_text) {
                        Ok(clipboard_json) => {
                            // Extract "outside" array from clipboard
                            let new_outside = if let Some(obj) = clipboard_json.as_object() {
                                obj.get("outside").and_then(|v| v.as_array()).cloned()
                            } else {
                                None
                            };

                            if let Some(new_outside_items) = new_outside {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(obj) = current_json.as_object_mut() {
                                            // Get or create outside array
                                            let outside_array = obj.entry("outside".to_string())
                                                .or_insert(Value::Array(vec![]));

                                            if let Some(arr) = outside_array.as_array_mut() {
                                                // Append new items
                                                for item in new_outside_items {
                                                    arr.push(item);
                                                }

                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.convert_json();
                                                        self.set_status("OUTSIDE entries appended from clipboard");
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("Current 'outside' is not an array");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("No 'outside' array in clipboard JSON");
                            }
                        }
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn paste_append_all(&mut self) {
        // Append both inside and outside from clipboard
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(clipboard_text) => {
                    match serde_json::from_str::<Value>(&clipboard_text) {
                        Ok(clipboard_json) => {
                            if let Some(clipboard_obj) = clipboard_json.as_object() {
                                // Parse current JSON
                                match serde_json::from_str::<Value>(&self.json_input) {
                                    Ok(mut current_json) => {
                                        if let Some(current_obj) = current_json.as_object_mut() {
                                            let mut appended_sections = Vec::new();

                                            // Append INSIDE entries
                                            if let Some(clipboard_inside) = clipboard_obj.get("inside").and_then(|v| v.as_array()) {
                                                let inside_array = current_obj.entry("inside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = inside_array.as_array_mut() {
                                                    for item in clipboard_inside {
                                                        arr.push(item.clone());
                                                    }
                                                    appended_sections.push("INSIDE");
                                                }
                                            }

                                            // Append OUTSIDE entries
                                            if let Some(clipboard_outside) = clipboard_obj.get("outside").and_then(|v| v.as_array()) {
                                                let outside_array = current_obj.entry("outside".to_string())
                                                    .or_insert(Value::Array(vec![]));

                                                if let Some(arr) = outside_array.as_array_mut() {
                                                    for item in clipboard_outside {
                                                        arr.push(item.clone());
                                                    }
                                                    appended_sections.push("OUTSIDE");
                                                }
                                            }

                                            if !appended_sections.is_empty() {
                                                // Format and save
                                                match serde_json::to_string_pretty(&current_json) {
                                                    Ok(formatted) => {
                                                        self.json_input = formatted;
                                                        self.is_modified = true;
                                                        self.convert_json();
                                                        self.set_status(&format!("{} appended from clipboard", appended_sections.join(" and ")));
                                                    }
                                                    Err(e) => self.set_status(&format!("Format error: {}", e)),
                                                }
                                            } else {
                                                self.set_status("No inside/outside arrays in clipboard");
                                            }
                                        } else {
                                            self.set_status("Current JSON is not an object");
                                        }
                                    }
                                    Err(e) => self.set_status(&format!("Invalid current JSON: {}", e)),
                                }
                            } else {
                                self.set_status("Clipboard JSON is not an object");
                            }
                        }
                        Err(e) => self.set_status(&format!("Clipboard is not valid JSON: {}", e)),
                    }
                }
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    pub fn clear_inside(&mut self) {
        // Clear INSIDE section
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut current_json) => {
                if let Some(obj) = current_json.as_object_mut() {
                    // Set inside to empty array
                    obj.insert("inside".to_string(), Value::Array(vec![]));

                    // Format and save
                    match serde_json::to_string_pretty(&current_json) {
                        Ok(formatted) => {
                            self.json_input = formatted;
                            self.is_modified = true;
                            self.convert_json();
                            self.set_status("INSIDE section cleared");
                        }
                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                    }
                } else {
                    self.set_status("Current JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn clear_outside(&mut self) {
        // Clear OUTSIDE section
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut current_json) => {
                if let Some(obj) = current_json.as_object_mut() {
                    // Set outside to empty array
                    obj.insert("outside".to_string(), Value::Array(vec![]));

                    // Format and save
                    match serde_json::to_string_pretty(&current_json) {
                        Ok(formatted) => {
                            self.json_input = formatted;
                            self.is_modified = true;
                            self.convert_json();
                            self.set_status("OUTSIDE section cleared");
                        }
                        Err(e) => self.set_status(&format!("Format error: {}", e)),
                    }
                } else {
                    self.set_status("Current JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn copy_outside_data(&mut self) {
        // In view mode, copy all OUTSIDE entries from relf_entries
        if self.format_mode == FormatMode::View {
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    // Collect OUTSIDE entries (indices < outside_count)
                    let mut outside_content = Vec::new();
                    outside_content.push("OUTSIDE".to_string());
                    outside_content.push(String::new());

                    for (i, entry) in self.relf_entries.iter().enumerate() {
                        if i < outside_count {
                            // Add blank line between entries (but not before first entry)
                            if i > 0 {
                                outside_content.push(String::new());
                            }
                            for line in &entry.lines {
                                outside_content.push(line.clone());
                            }
                        }
                    }

                    if outside_content.is_empty() {
                        self.set_status("No OUTSIDE entries found");
                        return;
                    }

                    let content = outside_content.join("\n");
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(content) {
                            Ok(()) => self.set_status("Copied OUTSIDE section to clipboard"),
                            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                        },
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    }
                    return;
                }
            }
            self.set_status("Failed to parse JSON");
            return;
        }

        // In Edit mode, copy with "outside: [...]" wrapper
        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(json_value) => {
                if let Some(obj) = json_value.as_object() {
                    if let Some(outside) = obj.get("outside") {
                        // Create wrapper object with "outside" key
                        let mut wrapper = serde_json::Map::new();
                        wrapper.insert("outside".to_string(), outside.clone());
                        let wrapper_value = Value::Object(wrapper);

                        match serde_json::to_string_pretty(&wrapper_value) {
                            Ok(formatted) => match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(formatted) {
                                    Ok(()) => self.set_status("Copied outside data to clipboard"),
                                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                                },
                                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                            },
                            Err(e) => {
                                self.set_status(&format!("Error formatting outside data: {}", e))
                            }
                        }
                    } else {
                        self.set_status("No 'outside' field found");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn clear_content(&mut self) {
        self.json_input.clear();
        self.rendered_content = vec![];
        self.relf_line_styles.clear();
        self.relf_visual_styles.clear();
        self.relf_entries.clear();
        self.previous_relf_styles.clear();
        self.previous_relf_visual_styles.clear();
        self.showing_help = false; // Reset help state

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
            let timeout = if self.status_message.contains("copied")
                || self.status_message.contains("Loaded:")
            {
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
        // In Edit mode, move the cursor up by a full page of visual lines
        if self.format_mode == FormatMode::Edit {
            let count = self.get_visible_height() as usize;
            for _ in 0..count {
                self.move_cursor_up();
            }
        } else {
            self.scroll = self.scroll.saturating_sub(self.get_visible_height());
        }
    }

    pub fn page_down(&mut self) {
        // In Edit mode, move the cursor down by a full page of visual lines
        if self.format_mode == FormatMode::Edit {
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
            if self.showing_help {
                // Allow scrolling to top in help mode (takes priority)
                self.scroll_to_top();
            } else if self.format_mode == FormatMode::Edit {
                self.scroll_to_top();
                self.content_cursor_line = 0;
                self.content_cursor_col = 0;
            } else if !self.relf_entries.is_empty() {
                // Jump to first card
                self.selected_entry_index = 0;
            } else {
                self.scroll_to_top();
                self.content_cursor_line = 0;
                self.content_cursor_col = 0;
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "dd" {
            // Delete current data entry
            if self.format_mode == FormatMode::Edit {
                self.delete_current_entry();
                self.is_modified = true;
            } else if !self.relf_entries.is_empty() {
                // Delete selected entry in card view
                self.delete_selected_entry();
                self.is_modified = true;
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "g-" {
            // Undo (vim-style)
            if self.format_mode == FormatMode::Edit {
                self.undo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "g+" {
            // Redo (vim-style)
            if self.format_mode == FormatMode::Edit {
                self.redo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer.len() >= 2 {
            self.vim_buffer.clear();
        }

        false
    }

    pub fn delete_selected_entry(&mut self) {
        // Delete the selected entry from relf_entries by removing it from JSON
        if self.relf_entries.is_empty() || self.selected_entry_index >= self.relf_entries.len() {
            self.set_status("No entry to delete");
            return;
        }

        match serde_json::from_str::<Value>(&self.json_input) {
            Ok(mut json_value) => {
                if let Some(obj) = json_value.as_object_mut() {
                    // Count entries to find which section and index
                    let mut current_idx = 0;
                    let target_idx = self.selected_entry_index;
                    let mut found = false;

                    // Check outside section first
                    if let Some(outside) = obj.get_mut("outside") {
                        if let Some(outside_array) = outside.as_array_mut() {
                            let outside_count = outside_array.len();
                            if target_idx < current_idx + outside_count {
                                let local_idx = target_idx - current_idx;
                                outside_array.remove(local_idx);
                                found = true;
                            } else {
                                current_idx += outside_count;
                            }
                        }
                    }

                    // Check inside section if not found
                    if !found {
                        if let Some(inside) = obj.get_mut("inside") {
                            if let Some(inside_array) = inside.as_array_mut() {
                                let local_idx = target_idx - current_idx;
                                if local_idx < inside_array.len() {
                                    inside_array.remove(local_idx);
                                    found = true;
                                }
                            }
                        }
                    }

                    if found {
                        // Update JSON and re-render
                        match serde_json::to_string_pretty(&json_value) {
                            Ok(formatted) => {
                                self.json_input = formatted;
                                self.convert_json();

                                // Move selection up (to previous entry)
                                if !self.relf_entries.is_empty() {
                                    if self.selected_entry_index > 0 {
                                        self.selected_entry_index -= 1;
                                    } else if self.selected_entry_index >= self.relf_entries.len() {
                                        self.selected_entry_index = self.relf_entries.len() - 1;
                                    }
                                }

                                self.set_status("Entry deleted");
                            }
                            Err(e) => self.set_status(&format!("Error formatting JSON: {}", e)),
                        }
                    } else {
                        self.set_status("Could not find entry to delete");
                    }
                } else {
                    self.set_status("JSON is not an object");
                }
            }
            Err(e) => self.set_status(&format!("Invalid JSON: {}", e)),
        }
    }

    pub fn delete_current_entry(&mut self) {
        // Save undo state before modification
        self.save_undo_state();

        let lines = self.get_json_lines();
        match JsonOperations::delete_entry_at_cursor(
            &self.json_input,
            self.content_cursor_line,
            &lines,
        ) {
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

    pub fn jump_to_first_outside(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // In Edit mode, find the first outside entry
            let lines = self.get_json_lines();
            for (i, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("\"outside\"") {
                    // Move to the first entry after "outside": [
                    if i + 1 < lines.len() {
                        self.content_cursor_line = i + 1;
                        self.content_cursor_col = 0;
                        self.ensure_cursor_visible();
                        self.set_status("Jumped to first OUTSIDE entry");
                        return;
                    }
                }
            }
            self.set_status("No OUTSIDE entries found");
        } else {
            // In View mode, jump to first card in outside section
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    if let Some(outside) = obj.get("outside") {
                        if let Some(outside_array) = outside.as_array() {
                            if !outside_array.is_empty() {
                                self.selected_entry_index = 0;
                                self.set_status("Jumped to first OUTSIDE entry");
                                return;
                            }
                        }
                    }
                }
            }
            self.set_status("No OUTSIDE entries found");
        }
    }

    pub fn jump_to_first_inside(&mut self) {
        if self.format_mode == FormatMode::Edit {
            // In Edit mode, find the first inside entry
            let lines = self.get_json_lines();
            for (i, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("\"inside\"") {
                    // Move to the first entry after "inside": [
                    if i + 1 < lines.len() {
                        self.content_cursor_line = i + 1;
                        self.content_cursor_col = 0;
                        self.ensure_cursor_visible();
                        self.set_status("Jumped to first INSIDE entry");
                        return;
                    }
                }
            }
            self.set_status("No INSIDE entries found");
        } else {
            // In View mode, jump to first card in inside section
            if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                if let Some(obj) = json_value.as_object() {
                    let outside_count = obj
                        .get("outside")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    if let Some(inside) = obj.get("inside") {
                        if let Some(inside_array) = inside.as_array() {
                            if !inside_array.is_empty() && outside_count < self.relf_entries.len() {
                                self.selected_entry_index = outside_count;
                                self.set_status("Jumped to first INSIDE entry");
                                return;
                            }
                        }
                    }
                }
            }
            self.set_status("No INSIDE entries found");
        }
    }

    pub fn move_to_next_word_end(&mut self) {
        // Vim-like 'e': always make forward progress to the end of the next word
        let lines = self.get_json_lines();
        if lines.is_empty() {
            return;
        }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self
            .content_cursor_line
            .min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        // Iterator: advance one position forward from (li, ci)
        let next_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() {
                return None;
            }
            // move to next char on the same line
            if ci2 + 1 < line_chars[li2].len() {
                return Some((li2, ci2 + 1, line_chars[li2][ci2 + 1]));
            }
            // otherwise, jump to the first char of the next non-empty line
            li2 += 1;
            while li2 < line_chars.len() {
                if !line_chars[li2].is_empty() {
                    return Some((li2, 0, line_chars[li2][0]));
                }
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
            li = nli;
            ci = nci;
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
        if lines.is_empty() {
            return;
        }

        let is_word = Navigator::is_word_char;
        let line_chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let mut li = self
            .content_cursor_line
            .min(line_chars.len().saturating_sub(1));
        let mut ci = self.content_cursor_col;

        let prev_pos = |mut li2: usize, ci2: usize| -> Option<(usize, usize, char)> {
            if li2 >= line_chars.len() {
                return None;
            }
            if ci2 > 0 {
                return Some((li2, ci2 - 1, line_chars[li2][ci2 - 1]));
            }
            if li2 == 0 {
                return None;
            }
            li2 -= 1;
            while let Some(line) = line_chars.get(li2) {
                if !line.is_empty() {
                    return Some((li2, line.len() - 1, line[line.len() - 1]));
                }
                if li2 == 0 {
                    break;
                }
                li2 -= 1;
            }
            None
        };

        if li == 0 && ci == 0 {
            return;
        }

        // Start scanning strictly before current position to guarantee progress
        let mut in_word = false;
        let mut start_li = li;
        let mut start_ci = ci; // will hold the start index of the found word
        let mut saw_any_word = false;
        while let Some((pli, pci, ch)) = prev_pos(li, ci) {
            if is_word(ch) {
                saw_any_word = true;
                start_li = pli;
                start_ci = pci; // keep updating until we leave the word
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
            li = pli;
            ci = pci;
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
            self.relf_line_styles = self.previous_relf_styles.clone();
            self.relf_visual_styles = self.previous_relf_visual_styles.clone();
            self.showing_help = false;
            self.scroll = 0;
            self.set_status("");
        } else {
            // Save current content and show help
            self.previous_content = self.rendered_content.clone();
            self.previous_relf_styles = self.relf_line_styles.clone();
            self.previous_relf_visual_styles = self.relf_visual_styles.clone();
            self.rendered_content = vec![
                "revw".to_string(),
                "".to_string(),
                "View Mode (Card View):".to_string(),
                "  v     - Paste file path or JSON content".to_string(),
                "  c     - Copy rendered content to clipboard".to_string(),
                "  r     - Toggle between View (default) and Edit mode".to_string(),
                "  x     - Clear content and status".to_string(),
                "  j/k/↑/↓ - Select card (or mouse wheel)".to_string(),
                "  gg    - Select first card".to_string(),
                "  G     - Select last card".to_string(),
                "  :gi   - Jump to first INSIDE entry".to_string(),
                "  :go   - Jump to first OUTSIDE entry".to_string(),
                "  /     - Search forward (highlights and jumps to card)".to_string(),
                "  n     - Next search match (jumps to card)".to_string(),
                "  N     - Previous search match (jumps to card)".to_string(),
                "  :noh  - Clear search highlighting".to_string(),
                "  :d    - Delete selected card".to_string(),
                "  :cu   - Copy URL from selected card".to_string(),
                "  Enter - Open edit overlay".to_string(),
                "  :h    - Toggle this help".to_string(),
                "  q     - Quit".to_string(),
                "".to_string(),
                "Edit Overlay (opened with Enter):".to_string(),
                "  j/k/↑/↓ - Navigate fields".to_string(),
                "  i     - Enter insert mode".to_string(),
                "  Ctrl+[ - Exit insert mode".to_string(),
                "  ←/→   - Move cursor (in insert mode)".to_string(),
                "  Enter - Save changes".to_string(),
                "  q/Esc - Cancel".to_string(),
                "".to_string(),
                "Edit Mode:".to_string(),
                "  i     - Insert mode".to_string(),
                "  e     - Move to next word end (like vim)".to_string(),
                "  b     - Move to previous word start (like vim)".to_string(),
                "  :d    - Delete current data entry".to_string(),
                "  u     - Undo".to_string(),
                "  Ctrl+r - Redo".to_string(),
                "  g-    - Undo".to_string(),
                "  g+    - Redo".to_string(),
                "  h/j/k/l - Move cursor (vim-like)".to_string(),
                "  :s/old/new/     - Replace first match on current line".to_string(),
                "  :s/old/new/g    - Replace all matches on current line".to_string(),
                "  :%s/old/new/g   - Replace all matches in file".to_string(),
                "  :%s/old/new/gc  - Replace all with confirmation".to_string(),
                "  :gi   - Jump to first INSIDE entry".to_string(),
                "  :go   - Jump to first OUTSIDE entry".to_string(),
                "  :ai   - Add inside entry at top (date, context)".to_string(),
                "  :ao   - Add outside entry (name, context, url, percentage)".to_string(),
                "  :o    - Order entries (outside by %, inside by date)".to_string(),
                "  :ci   - Copy only inside data to clipboard".to_string(),
                "  :co   - Copy only outside data to clipboard".to_string(),
                "  :w    - Save file".to_string(),
                "  :wq   - Save and quit".to_string(),
                "  :q    - Quit without saving".to_string(),
                "  :e    - Reload current file".to_string(),
                "  :e <file> - Open a different file".to_string(),
                "  :ar   - Toggle auto-reload (default: on)".to_string(),
                "  :h    - Toggle this help".to_string(),
                "  Esc   - Exit insert/command mode".to_string(),
                "".to_string(),
                "Usage:".to_string(),
                "  revw [file.json]     - Open in View mode".to_string(),
                "  revw --json [file]   - Open in Edit mode".to_string(),
                "  revw --output [file] - Output to file".to_string(),
                "  revw --stdout [file] - Output to stdout".to_string(),
            ];
            self.relf_line_styles = vec![RelfLineStyle::default(); self.rendered_content.len()];
            self.relf_visual_styles.clear();
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
        // In Edit mode, update rendered content directly to preserve raw format
        if self.format_mode == FormatMode::Edit {
            self.rendered_content = self.render_json();
            self.relf_line_styles.clear();
            self.relf_visual_styles.clear();
        } else {
            self.convert_json();
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.format_mode == FormatMode::Edit {
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
        if self.format_mode == FormatMode::Edit {
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
        if self.format_mode == FormatMode::Edit {
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
        if self.format_mode == FormatMode::Edit {
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
        let content_lines = if self.format_mode == FormatMode::Edit {
            lines.len()
        } else {
            self.rendered_content.len()
        };

        // Move cursor down if there's content, otherwise just scroll screen
        if self.content_cursor_line + 1 < content_lines {
            // Normal cursor movement within content
            self.content_cursor_line += 1;

            let line_len =
                if self.format_mode == FormatMode::Edit && self.content_cursor_line < lines.len() {
                    lines[self.content_cursor_line].chars().count()
                } else if self.content_cursor_line < self.rendered_content.len() {
                    self.rendered_content[self.content_cursor_line]
                        .chars()
                        .count()
                } else {
                    0
                };

            self.content_cursor_col = self.content_cursor_col.min(line_len);
        } else {
            // Cursor is at last line, just scroll the screen down (Mario style)
            // Don't add virtual padding in help mode
            let virtual_padding = if self.showing_help { 0 } else { 10 };
            let max_scroll =
                (content_lines as u16 + virtual_padding).saturating_sub(self.get_visible_height());
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
        let content_lines = if self.format_mode == FormatMode::Edit {
            lines.len()
        } else {
            self.rendered_content.len()
        };

        // Keep cursor within actual content lines (not in virtual padding)
        if self.content_cursor_line >= content_lines {
            self.content_cursor_line = content_lines.saturating_sub(1);
        }

        // Handle cursor column bounds
        let line_len =
            if self.format_mode == FormatMode::Edit && self.content_cursor_line < lines.len() {
                lines[self.content_cursor_line].chars().count()
            } else if self.content_cursor_line < self.rendered_content.len() {
                self.rendered_content[self.content_cursor_line]
                    .chars()
                    .count()
            } else {
                0
            };
        if self.content_cursor_col > line_len {
            self.content_cursor_col = line_len;
        }

        // Vertical scrolling
        let cursor_line = if self.format_mode == FormatMode::Edit {
            self.content_cursor_line as u16
        } else {
            self.calculate_cursor_visual_position().0
        };
        let visible_height = self.get_visible_height();
        let scrolloff = 3u16;
        if cursor_line < self.scroll {
            self.scroll = cursor_line;
        } else if visible_height > 0 && cursor_line >= self.scroll + visible_height {
            self.scroll = cursor_line.saturating_sub(visible_height - 1);
        } else if visible_height > scrolloff * 2 {
            if cursor_line < self.scroll + scrolloff {
                self.scroll = cursor_line.saturating_sub(scrolloff);
            } else if cursor_line > self.scroll + visible_height - scrolloff - 1 {
                self.scroll = cursor_line + scrolloff + 1 - visible_height;
            }
        }

        // Allow scrolling into virtual padding
        let virtual_padding = 10;
        let max_scroll = (content_lines as u16 + virtual_padding).saturating_sub(visible_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        // Horizontal follow for Edit mode
        if self.format_mode == FormatMode::Edit {
            if self.content_cursor_line < lines.len() {
                let current = &lines[self.content_cursor_line];
                let col = self.prefix_display_width(current, self.content_cursor_col) as u16;
                let w = self.get_content_width();
                if col < self.hscroll {
                    self.hscroll = col;
                } else if col >= self.hscroll + w {
                    self.hscroll = col - w + 1;
                }
            }
        }
    }

    pub fn get_visible_height(&self) -> u16 {
        // Use the last measured inner content height from render pass
        if self.visible_height > 0 {
            self.visible_height
        } else {
            20
        }
    }

    pub fn get_content_width(&self) -> u16 {
        // Prefer the measured inner content width set during render.
        // Fallback to a reasonable default if unavailable.
        if self.content_width > 2 {
            self.content_width.saturating_sub(0)
        } else {
            80
        }
    }

    pub fn calculate_visual_lines(&self, text_line: &str) -> u16 {
        let width = self.get_content_width() as usize;
        Navigator::calculate_visual_lines(text_line, width)
    }

    pub fn build_visual_lines(&mut self) -> Vec<String> {
        let raw_width = self.get_content_width();
        if raw_width == 0 {
            if self.format_mode == FormatMode::View {
                self.relf_visual_styles = self.relf_line_styles.clone();
            } else {
                self.relf_visual_styles.clear();
            }
            return self.rendered_content.clone();
        }

        let width = raw_width as usize;

        if self.format_mode == FormatMode::View {
            let mut wrapped_lines = Vec::new();
            let mut visual_styles = Vec::new();

            for (idx, line) in self.rendered_content.iter().enumerate() {
                let style = self.relf_line_styles.get(idx).cloned().unwrap_or_default();

                if line.is_empty() {
                    wrapped_lines.push(String::new());
                    visual_styles.push(style);
                    continue;
                }

                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                let indent_width: usize = indent
                    .chars()
                    .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
                    .sum();

                let mut current_line = String::new();
                let mut current_width = 0usize;

                for ch in line.chars() {
                    let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);

                    if current_width + ch_width > width && !current_line.is_empty() {
                        wrapped_lines.push(current_line.clone());
                        visual_styles.push(style.clone());
                        current_line.clear();
                        current_width = 0;

                        if !indent.is_empty() && indent_width < width {
                            current_line.push_str(&indent);
                            current_width = indent_width;
                        }
                    }

                    current_line.push(ch);
                    current_width += ch_width;
                }

                if !current_line.is_empty() {
                    wrapped_lines.push(current_line);
                    visual_styles.push(style);
                }
            }

            self.relf_visual_styles = visual_styles;
            return wrapped_lines;
        }

        if self.format_mode == FormatMode::Edit {
            self.relf_visual_styles.clear();
            return self.rendered_content.clone();
        }

        // Unused branch currently
        self.relf_visual_styles.clear();
        self.rendered_content.clone()
    }

    pub fn calculate_cursor_visual_position(&self) -> (u16, u16) {
        let lines = self.get_json_lines();
        let width = self.get_content_width() as usize;
        Navigator::calculate_cursor_visual_position(
            &lines,
            self.content_cursor_line,
            self.content_cursor_col,
            width,
        )
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
        } else if cmd.starts_with("e ") {
            // Open a different file
            let filename = cmd.strip_prefix("e ").unwrap().trim().to_string();
            let path = PathBuf::from(filename);
            self.load_file(path);
        } else if cmd == "ar" {
            // Toggle auto-reload
            self.auto_reload = !self.auto_reload;
            let status = if self.auto_reload {
                "Auto-reload enabled"
            } else {
                "Auto-reload disabled"
            };
            self.set_status(status);
        } else if cmd == "ai" {
            // Add new inside entry at top
            self.append_inside();
        } else if cmd == "ao" {
            self.append_outside();
        } else if cmd == "o" {
            // Order entries
            self.order_entries();
        } else if cmd == "gi" {
            // Jump to first INSIDE entry
            self.jump_to_first_inside();
        } else if cmd == "go" {
            // Jump to first OUTSIDE entry
            self.jump_to_first_outside();
        } else if cmd == "ci" {
            // Copy inside data
            self.copy_inside_data();
        } else if cmd == "co" {
            // Copy outside data
            self.copy_outside_data();
        } else if cmd == "cu" {
            // Copy URL from selected entry
            self.copy_selected_url();
        } else if cmd == "vi" {
            // Paste INSIDE from clipboard (overwrite)
            self.paste_inside_overwrite();
        } else if cmd == "vo" {
            // Paste OUTSIDE from clipboard (overwrite)
            self.paste_outside_overwrite();
        } else if cmd == "va" {
            // Append from clipboard (both inside and outside)
            self.paste_append_all();
        } else if cmd == "vai" {
            // Paste INSIDE from clipboard (append)
            self.paste_inside_append();
        } else if cmd == "vao" {
            // Paste OUTSIDE from clipboard (append)
            self.paste_outside_append();
        } else if cmd == "xi" {
            // Clear INSIDE section
            self.clear_inside();
        } else if cmd == "xo" {
            // Clear OUTSIDE section
            self.clear_outside();
        } else if cmd == "d" {
            // Delete entry (works in both View and Edit mode)
            if self.format_mode == FormatMode::Edit {
                self.delete_current_entry();
                self.is_modified = true;
            } else if !self.relf_entries.is_empty() {
                self.delete_selected_entry();
                self.is_modified = true;
                // Auto-save after deletion in View mode
                self.save_file();
            }
        } else if cmd == "noh" {
            // Clear search highlighting
            self.clear_search_highlight();
        } else if cmd == "h" {
            self.show_help();
        } else if cmd.starts_with("s/") || cmd.starts_with("%s/") {
            // Substitute command: :s/pattern/replacement/flags or :%s/pattern/replacement/flags
            self.execute_substitute(cmd);
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

                // Jump to the new entry (don't open edit overlay or insert mode)
                if self.format_mode == FormatMode::View {
                    // New inside entry is added at the beginning of inside array
                    // Index = outside.length (start of INSIDE section)
                    if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            let outside_count = obj
                                .get("outside")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            // INSIDE section starts right after OUTSIDE
                            self.selected_entry_index = outside_count;
                            self.scroll = 0;
                        }
                    }
                } else {
                    // In Edit mode, just move cursor to the new entry
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                }
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

                // Jump to the new entry (don't open edit overlay or insert mode)
                if self.format_mode == FormatMode::View {
                    // New outside entry is added at the end of outside array
                    // Index = outside.length - 1 (last OUTSIDE entry)
                    if let Ok(json_value) = serde_json::from_str::<Value>(&self.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            let outside_count = obj
                                .get("outside")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            // Last outside entry
                            self.selected_entry_index = outside_count.saturating_sub(1);
                            self.scroll = 0;
                        }
                    }
                } else {
                    // In Edit mode, just move cursor to the new entry
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                }
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

                // Auto-save in view mode
                if self.format_mode == FormatMode::View {
                    self.save_file();
                }

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
        self.set_status("/");
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
            self.set_status(&format!(
                "Found {} matches for '{}'",
                self.search_matches.len(),
                self.search_query
            ));
        } else {
            self.current_match_index = None;
            self.set_status(&format!("Pattern not found: {}", self.search_query));
        }
    }

    pub fn clear_search_highlight(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;
        self.set_status("Search highlight cleared");
    }

    pub fn find_matches(&mut self) {
        self.search_matches.clear();

        // For card view, search within entry content
        if self.format_mode == FormatMode::View && !self.relf_entries.is_empty() {
            let query_lower = self.search_query.to_lowercase();

            for (entry_idx, entry) in self.relf_entries.iter().enumerate() {
                for (_line_idx, line) in entry.lines.iter().enumerate() {
                    let line_lower = line.to_lowercase();
                    let mut byte_pos = 0;

                    while byte_pos < line_lower.len() {
                        if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
                            let actual_byte_pos = byte_pos + match_pos;
                            // Convert byte position to char position
                            let char_pos = line[..actual_byte_pos.min(line.len())].chars().count();
                            // Store entry_idx in line position, and char position in col position
                            self.search_matches.push((entry_idx, char_pos));
                            // Move past this match, ensuring we stay on char boundary
                            byte_pos = actual_byte_pos + query_lower.len();
                            while byte_pos < line_lower.len() && !line_lower.is_char_boundary(byte_pos) {
                                byte_pos += 1;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            return;
        }

        let search_content = if self.format_mode == FormatMode::Edit {
            &self.get_json_lines()
        } else {
            &self.rendered_content
        };

        let query_lower = self.search_query.to_lowercase();

        for (line_idx, line) in search_content.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let mut byte_pos = 0;

            while byte_pos < line_lower.len() {
                if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
                    let actual_byte_pos = byte_pos + match_pos;
                    // Convert byte position to char position for storage
                    let char_pos = line[..actual_byte_pos.min(line.len())].chars().count();
                    self.search_matches.push((line_idx, char_pos));
                    // Move past this match, ensuring we stay on char boundary
                    byte_pos = actual_byte_pos + query_lower.len();
                    // If we're not on a char boundary, find the next one
                    while byte_pos < line_lower.len() && !line_lower.is_char_boundary(byte_pos) {
                        byte_pos += 1;
                    }
                } else {
                    break;
                }
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
        self.set_status(&format!(
            "Match {} of {} for '{}'",
            next_idx + 1,
            self.search_matches.len(),
            self.search_query
        ));
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
        self.set_status(&format!(
            "Match {} of {} for '{}'",
            prev_idx + 1,
            self.search_matches.len(),
            self.search_query
        ));
    }

    pub fn jump_to_current_match(&mut self) {
        if let Some(match_idx) = self.current_match_index {
            if let Some(&(line, col)) = self.search_matches.get(match_idx) {
                if self.format_mode == FormatMode::Edit {
                    self.content_cursor_line = line;
                    self.content_cursor_col = col;
                    self.ensure_cursor_visible();
                } else if !self.relf_entries.is_empty() {
                    // For card view, jump to the entry
                    self.selected_entry_index = line;
                } else {
                    // For View mode, just scroll to the line
                    self.scroll = line as u16;
                    let max_scroll = self
                        .rendered_content
                        .len()
                        .saturating_sub(self.get_visible_height() as usize)
                        as u16;
                    if self.scroll > max_scroll {
                        self.scroll = max_scroll;
                    }
                    self.ensure_cursor_visible();
                }
            }
        }
    }

    pub fn execute_substitute(&mut self, cmd: &str) {
        // Only works in Edit mode
        if self.format_mode != FormatMode::Edit {
            self.set_status("Substitute only works in Edit mode");
            return;
        }

        // Parse the substitute command
        let is_global_file = cmd.starts_with("%s/");
        let cmd_prefix = if is_global_file { "%s/" } else { "s/" };
        let cmd_rest = cmd.strip_prefix(cmd_prefix).unwrap_or("");

        // Split by '/' to get pattern, replacement, and flags
        let parts: Vec<&str> = cmd_rest.splitn(3, '/').collect();
        if parts.len() < 2 {
            self.set_status("Invalid substitute syntax. Use :s/pattern/replacement/[flags]");
            return;
        }

        let pattern = parts[0];
        let replacement = parts[1];
        let flags = if parts.len() == 3 { parts[2] } else { "" };

        if pattern.is_empty() {
            self.set_status("Empty pattern");
            return;
        }

        let global_line = flags.contains('g');
        let confirm = flags.contains('c');

        // Save undo state before making changes
        self.save_undo_state();

        if confirm {
            // Build list of all matches for confirmation
            self.build_substitute_confirmations(pattern, replacement, is_global_file, global_line);
            if self.substitute_confirmations.is_empty() {
                self.set_status(&format!("Pattern not found: {}", pattern));
            } else {
                self.current_substitute_index = 0;
                self.set_status(&format!(
                    "Replace with '{}'? (y/n/a/q) [{}/{}]",
                    replacement,
                    self.current_substitute_index + 1,
                    self.substitute_confirmations.len()
                ));
            }
        } else {
            // Perform substitution without confirmation
            let count = self.perform_substitute(pattern, replacement, is_global_file, global_line);
            if count > 0 {
                self.is_modified = true;
                self.convert_json();
                self.set_status(&format!("{} substitution{} made", count, if count == 1 { "" } else { "s" }));
            } else {
                self.set_status(&format!("Pattern not found: {}", pattern));
                // Remove the undo state we just saved since nothing changed
                self.undo_stack.pop();
            }
        }
    }

    fn build_substitute_confirmations(&mut self, pattern: &str, replacement: &str, is_global_file: bool, global_line: bool) {
        self.substitute_confirmations.clear();

        let lines: Vec<String> = self.json_input.lines().map(|s| s.to_string()).collect();

        let line_range = if is_global_file {
            0..lines.len()
        } else {
            self.content_cursor_line..self.content_cursor_line + 1
        };

        for line_idx in line_range {
            if line_idx >= lines.len() {
                break;
            }
            let line = &lines[line_idx];

            if global_line {
                // Find all occurrences on this line
                let mut search_start = 0;
                while let Some(pos) = line[search_start..].find(pattern) {
                    let actual_pos = search_start + pos;
                    self.substitute_confirmations.push(SubstituteMatch {
                        line: line_idx,
                        col: actual_pos,
                        pattern: pattern.to_string(),
                        replacement: replacement.to_string(),
                    });
                    search_start = actual_pos + pattern.len();
                }
            } else {
                // Find only first occurrence on this line
                if let Some(pos) = line.find(pattern) {
                    self.substitute_confirmations.push(SubstituteMatch {
                        line: line_idx,
                        col: pos,
                        pattern: pattern.to_string(),
                        replacement: replacement.to_string(),
                    });
                }
            }
        }
    }

    fn perform_substitute(&mut self, pattern: &str, replacement: &str, is_global_file: bool, global_line: bool) -> usize {
        let mut lines: Vec<String> = self.json_input.lines().map(|s| s.to_string()).collect();
        let mut count = 0;

        let line_range = if is_global_file {
            0..lines.len()
        } else {
            self.content_cursor_line..self.content_cursor_line + 1
        };

        for line_idx in line_range {
            if line_idx >= lines.len() {
                break;
            }

            if global_line {
                // Replace all occurrences on this line
                let original = lines[line_idx].clone();
                lines[line_idx] = original.replace(pattern, replacement);
                // Count how many replacements were made
                if lines[line_idx] != original {
                    count += original.matches(pattern).count();
                }
            } else {
                // Replace only first occurrence on this line
                if let Some(pos) = lines[line_idx].find(pattern) {
                    lines[line_idx].replace_range(pos..pos + pattern.len(), replacement);
                    count += 1;
                }
            }
        }

        if count > 0 {
            self.json_input = lines.join("\n");
            // Preserve trailing newline if original had one
            if self.json_input.chars().last() != Some('\n') && !self.json_input.is_empty() {
                if let Some(last_char) = self.json_input.chars().last() {
                    if last_char != '\n' {
                        self.json_input.push('\n');
                    }
                }
            }
        }

        count
    }

    pub fn handle_substitute_confirmation(&mut self, answer: char) {
        if self.substitute_confirmations.is_empty() {
            return;
        }

        let mut should_substitute = false;
        let mut quit = false;
        let mut all = false;

        match answer {
            'y' => should_substitute = true,
            'n' => should_substitute = false,
            'a' => {
                should_substitute = true;
                all = true;
            }
            'q' => quit = true,
            _ => return,
        }

        if quit {
            self.substitute_confirmations.clear();
            self.current_substitute_index = 0;
            self.set_status("Substitute cancelled");
            // Remove the undo state we saved since we're cancelling
            self.undo_stack.pop();
            return;
        }

        if all {
            // Perform all remaining substitutions
            let remaining_count = self.substitute_confirmations.len() - self.current_substitute_index;

            // Collect all matches to apply
            let matches_to_apply: Vec<SubstituteMatch> = self.substitute_confirmations
                [self.current_substitute_index..]
                .to_vec();

            // Apply substitutions in reverse order to maintain positions
            let mut lines: Vec<String> = self.json_input.lines().map(|s| s.to_string()).collect();
            for match_item in matches_to_apply.iter().rev() {
                if match_item.line < lines.len() {
                    let line = &mut lines[match_item.line];
                    if match_item.col + match_item.pattern.len() <= line.len() {
                        line.replace_range(
                            match_item.col..match_item.col + match_item.pattern.len(),
                            &match_item.replacement,
                        );
                    }
                }
            }
            self.json_input = lines.join("\n");

            self.substitute_confirmations.clear();
            self.current_substitute_index = 0;
            self.is_modified = true;
            self.convert_json();
            self.set_status(&format!("{} substitution{} made", remaining_count, if remaining_count == 1 { "" } else { "s" }));
        } else {
            if should_substitute {
                // Perform this substitution
                let match_item = &self.substitute_confirmations[self.current_substitute_index];
                let mut lines: Vec<String> = self.json_input.lines().map(|s| s.to_string()).collect();

                if match_item.line < lines.len() {
                    let line = &mut lines[match_item.line];
                    if match_item.col + match_item.pattern.len() <= line.len() {
                        line.replace_range(
                            match_item.col..match_item.col + match_item.pattern.len(),
                            &match_item.replacement,
                        );
                        self.json_input = lines.join("\n");
                        self.is_modified = true;
                    }
                }
            }

            // Move to next match
            self.current_substitute_index += 1;

            if self.current_substitute_index >= self.substitute_confirmations.len() {
                // All done
                let total = self.substitute_confirmations.len();
                self.substitute_confirmations.clear();
                self.current_substitute_index = 0;
                self.convert_json();
                self.set_status(&format!("{} substitution{} completed", total, if total == 1 { "" } else { "s" }));
            } else {
                // Show next confirmation prompt
                let next_match = &self.substitute_confirmations[self.current_substitute_index];
                self.set_status(&format!(
                    "Replace with '{}'? (y/n/a/q) [{}/{}]",
                    next_match.replacement,
                    self.current_substitute_index + 1,
                    self.substitute_confirmations.len()
                ));
            }
        }
    }
}
