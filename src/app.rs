mod clipboard;
mod command;
mod completion;
mod edit;
mod explorer;
mod explorer_ops;
mod file;
mod help;
mod history;
mod markdown;
mod navigation;
mod outline;
mod pdf;
mod search;
mod substitute;
mod token;
mod toon;
mod undo;

use crate::config::{BorderStyle, ColorScheme, RcConfig};
use crate::content_ops::ContentOperations;
use crate::json_ops::JsonOperations;
use crate::markdown_ops::MarkdownOperations;
use crate::navigation::Navigator;
use crate::toon_ops::ToonOperations;
use crate::rendering::{RelfEntry, RelfLineStyle, RelfRenderResult, Renderer};
use crate::syntax_highlight::SyntaxHighlighter;
use crate::ui::markdown_highlight::highlight_markdown_with_code_blocks;
use ratatui::text::Span;
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FileMode {
    Json,
    Markdown,
    Toon,
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
    pub markdown_input: String,
    pub toon_input: String,
    pub rendered_content: Vec<String>,
    pub relf_line_styles: Vec<RelfLineStyle>,
    pub relf_visual_styles: Vec<RelfLineStyle>,
    pub relf_entries: Vec<RelfEntry>,
    pub selected_entry_index: usize, // Currently selected entry in View mode
    pub editing_entry: bool, // Whether we're editing entry in overlay
    pub edit_buffer: Vec<String>, // Buffer for editing entry fields
    pub edit_buffer_is_placeholder: Vec<bool>, // Track if each field is a placeholder
    pub edit_field_index: usize, // Which field is being edited
    pub edit_field_editing_mode: bool, // Whether editing within a field (Enter pressed)
    pub edit_insert_mode: bool, // Whether in insert mode within overlay
    pub edit_skip_normal_mode: bool, // True if entered insert mode directly with 'i' (skip normal mode on Esc)
    pub edit_cursor_pos: usize, // Cursor position within current field
    pub edit_hscroll: u16, // Horizontal scroll offset for overlay fields
    pub edit_vscroll: u16, // Vertical scroll offset for context field
    pub edit_yank_buffer: String, // Yank buffer for overlay context field
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
    pub completion_candidates: Vec<String>, // Tab completion candidates
    pub completion_index: usize,    // Current completion index
    pub completion_original: String, // Original command before completion
    pub is_modified: bool,          // Track if content has been modified
    pub content_cursor_line: usize, // Current line in content
    pub content_cursor_col: usize,  // Current column in content line
    pub show_cursor: bool,          // Show/hide cursor in Normal mode
    pub dd_count: usize,            // Count consecutive 'd' presses for dd command
    pub yy_count: usize,            // Count consecutive 'y' presses for yy command
    pub line_yank_buffer: String,   // Buffer for yanked line (dd/yy commands)
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
    pub file_path_changed: bool, // Signal that file path changed and watcher needs update
    // Scrollbar interaction state
    pub dragging_scrollbar: Option<ScrollbarType>,
    // Substitute confirmation state
    pub substitute_confirmations: Vec<SubstituteMatch>,
    pub current_substitute_index: usize,
    // Double-click detection
    pub last_click_time: Option<Instant>,
    // Line number display setting
    pub show_line_numbers: bool,
    pub show_relative_line_numbers: bool,
    // Maximum visible cards in View mode (1-10, default 5)
    pub max_visible_cards: usize,
    // Show file extension in explorer
    pub show_extension: bool,
    // Command history buffers (max 10 entries each)
    pub command_history: Vec<String>,     // History for : commands
    pub search_history: Vec<String>,      // History for / searches
    pub command_history_index: Option<usize>, // Current position in command history
    pub search_history_index: Option<usize>,  // Current position in search history
    // File explorer (like vim :Lexplore)
    pub explorer_open: bool,
    pub explorer_entries: Vec<ExplorerEntry>,
    pub explorer_selected_index: usize,
    pub explorer_scroll: u16,
    pub explorer_horizontal_scroll: u16,
    pub explorer_current_dir: PathBuf,
    pub explorer_has_focus: bool, // Track which window has focus
    pub explorer_dir_changed: bool, // Signal that explorer directory changed and watcher needs update
    // File operation confirmation/prompt state
    pub file_op_pending: Option<FileOperation>,
    pub file_op_prompt_buffer: String, // Buffer for filename input during file operations
    // Visual/Select mode (View mode only)
    pub visual_mode: bool,
    pub visual_start_index: usize, // Start of visual selection
    pub visual_end_index: usize,   // End of visual selection (inclusive)
    // View Edit mode (Overlay mode only) - render \n as newlines
    pub view_edit_mode: bool,
    // Color scheme
    pub colorscheme: ColorScheme,
    // Border style (rounded or plain)
    pub border_style: BorderStyle,
    // Card outline overlay
    pub outline_open: bool,
    pub outline_selected_index: usize,
    pub outline_scroll: u16,
    pub outline_horizontal_scroll: u16,
    pub outline_opened_from_explorer: bool, // Track if outline was opened from explorer
    pub outline_has_focus: bool, // Track if outline has mouse focus
    pub outline_search_query: String, // Search query for outline
    pub outline_search_matches: Vec<usize>, // Indices of matching entries
    pub outline_search_current: usize, // Current match index in search_matches
    // File mode (JSON or Markdown)
    pub file_mode: FileMode,
    // Syntax highlighter (lazy initialized)
    pub syntax_highlighter: Option<SyntaxHighlighter>,
    // Cache for markdown syntax highlighting (Edit mode)
    pub markdown_highlight_cache: Vec<Vec<Span<'static>>>,
}

#[derive(Clone)]
pub struct ExplorerEntry {
    pub path: PathBuf,
    pub is_expanded: bool,  // Only meaningful for directories
    pub depth: usize,       // Indentation level from root (0 = root)
}

#[derive(Clone, PartialEq)]
pub enum FileOperation {
    Delete(PathBuf),         // Delete file (needs y/n confirmation)
    Copy(PathBuf),           // Copy file (needs destination filename)
    Rename(PathBuf),         // Rename file (needs new filename)
    Create,                  // Create new file (needs filename)
    CreateDir,               // Create new directory (needs directory name)
}

#[derive(Clone, Copy, PartialEq)]
pub enum ScrollbarType {
    Vertical,
    Horizontal,
}

#[derive(Clone)]
pub struct UndoState {
    pub json_input: String,
    pub markdown_input: String,
    pub toon_input: String,
    pub content_cursor_line: usize,
    pub content_cursor_col: usize,
    pub scroll: u16,
}

impl App {
    pub fn new(format_mode: FormatMode) -> Self {
        // Load RC configuration
        let rc_config = RcConfig::load();

        let app = Self {
            input_mode: InputMode::Normal,
            json_input: String::new(),
            markdown_input: String::new(),
            toon_input: String::new(),
            rendered_content: vec![],
            relf_line_styles: Vec::new(),
            relf_visual_styles: Vec::new(),
            relf_entries: Vec::new(),
            selected_entry_index: 0,
            editing_entry: false,
            edit_buffer: Vec::new(),
            edit_buffer_is_placeholder: Vec::new(),
            edit_field_index: 0,
            edit_field_editing_mode: false,
            edit_insert_mode: false,
            edit_skip_normal_mode: false,
            edit_cursor_pos: 0,
            edit_hscroll: 0,
            edit_vscroll: 0,
            edit_yank_buffer: String::new(),
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
            completion_candidates: Vec::new(),
            completion_index: 0,
            completion_original: String::new(),
            is_modified: false,
            content_cursor_line: 0,
            content_cursor_col: 0,
            show_cursor: true,
            dd_count: 0,
            yy_count: 0,
            line_yank_buffer: String::new(),
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
            file_path_changed: false,
            dragging_scrollbar: None,
            substitute_confirmations: Vec::new(),
            current_substitute_index: 0,
            last_click_time: None,
            show_line_numbers: rc_config.show_line_numbers,
            show_relative_line_numbers: rc_config.show_relative_line_numbers,
            show_extension: rc_config.show_extension,
            max_visible_cards: rc_config.max_visible_cards,
            command_history: Vec::new(),
            search_history: Vec::new(),
            command_history_index: None,
            search_history_index: None,
            explorer_open: false,
            explorer_entries: Vec::new(),
            explorer_selected_index: 0,
            explorer_scroll: 0,
            explorer_horizontal_scroll: 0,
            explorer_current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            explorer_has_focus: true, // Explorer has focus when opened
            explorer_dir_changed: false,
            file_op_pending: None,
            file_op_prompt_buffer: String::new(),
            visual_mode: false,
            visual_start_index: 0,
            visual_end_index: 0,
            view_edit_mode: false,
            colorscheme: rc_config.colorscheme,
            border_style: rc_config.border_style,
            outline_open: false,
            outline_selected_index: 0,
            outline_scroll: 0,
            outline_horizontal_scroll: 0,
            outline_opened_from_explorer: false,
            outline_has_focus: false,
            outline_search_query: String::new(),
            outline_search_matches: Vec::new(),
            outline_search_current: 0,
            file_mode: if rc_config.default_format.as_deref() == Some("markdown") {
                FileMode::Markdown
            } else {
                FileMode::Json
            },
            syntax_highlighter: None,
            markdown_highlight_cache: Vec::new(),
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
        let active_is_empty = if self.is_markdown_file() {
            self.markdown_input.is_empty()
        } else if self.is_toon_file() {
            self.toon_input.is_empty()
        } else {
            self.json_input.is_empty()
        };

        if active_is_empty {
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
                self.rendered_content = if self.is_markdown_file() {
                    // Update highlight cache for markdown
                    self.update_markdown_highlight_cache();
                    self.render_markdown()
                } else if self.is_toon_file() {
                    self.render_toon()
                } else {
                    self.render_json()
                };
                self.relf_line_styles.clear();
                self.relf_visual_styles.clear();
                // Don't reset scroll in Edit mode - preserve cursor position
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
                } else if !self.json_input.is_empty()
                    && (self.rendered_content.is_empty()
                        || (self.rendered_content.len() >= 2
                            && self.rendered_content[0].contains("Not valid JSON")))
                    && !self.is_markdown_file()
                    && !self.is_toon_file()
                {
                    // Only show error if we have input content, it's not markdown, and parsing failed
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

    fn render_markdown(&self) -> Vec<String> {
        self.markdown_input.lines().map(|line| line.to_string()).collect()
    }

    fn render_toon(&self) -> Vec<String> {
        self.toon_input.lines().map(|line| line.to_string()).collect()
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

    /// Check if the current file is a Markdown file
    pub fn is_markdown_file(&self) -> bool {
        // Check file extension if file path exists
        if let Some(is_md) = self.file_path
            .as_ref()
            .and_then(|path| path.extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
        {
            return is_md;
        }

        // If no file path, use file_mode setting
        self.file_mode == FileMode::Markdown
    }

    /// Get the appropriate content operations handler based on file type
    fn get_operations(&self) -> Box<dyn ContentOperations> {
        match self.file_mode {
            FileMode::Markdown => Box::new(MarkdownOperations),
            FileMode::Toon => Box::new(ToonOperations),
            FileMode::Json => Box::new(JsonOperations),
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


    pub fn get_content_lines(&self) -> Vec<String> {
        let content = if self.is_markdown_file() && !self.markdown_input.is_empty() {
            &self.markdown_input
        } else if self.is_toon_file() && !self.toon_input.is_empty() {
            &self.toon_input
        } else {
            &self.json_input
        };

        // Use split('\n') instead of lines() to preserve trailing empty lines
        // Remove the last element if it's empty and was caused by trailing \n
        let mut result: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
        // If content ends with \n, split will create an extra empty string at the end
        // We want to keep it if the user intentionally has an empty line, but remove if it's just the trailing \n
        if result.len() > 1 && result.last() == Some(&String::new()) && content.ends_with('\n') {
            result.pop();
        }
        result
    }

    pub fn set_content_from_lines(&mut self, lines: Vec<String>) {
        if self.is_markdown_file() {
            self.markdown_input = lines.join("\n");
            // Always add trailing newline for consistency
            if !self.markdown_input.is_empty() {
                self.markdown_input.push('\n');
            }
            match self.parse_markdown(&self.markdown_input) {
                Ok(json_content) => {
                    self.json_input = json_content;
                }
                Err(e) => {
                    // Keep the old json_input but set a status message
                    self.set_status(&format!("Markdown parse error: {}", e));
                }
            }
        } else if self.is_toon_file() {
            self.toon_input = lines.join("\n");
            // Always add trailing newline for consistency
            if !self.toon_input.is_empty() {
                self.toon_input.push('\n');
            }
            match self.parse_toon(&self.toon_input) {
                Ok(json_content) => {
                    self.json_input = json_content;
                }
                Err(e) => {
                    // Keep the old json_input but set a status message
                    self.set_status(&format!("Toon parse error: {}", e));
                }
            }
        } else {
            self.json_input = lines.join("\n");
            // Always add trailing newline for consistency
            if !self.json_input.is_empty() {
                self.json_input.push('\n');
            }
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
        let lines = self.get_content_lines();

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
        self.rendered_content = help::get_help_content();
        self.relf_line_styles.clear();
        self.relf_visual_styles.clear();
        self.relf_entries.clear();
        self.scroll = 0;
    }


    pub fn clear_content(&mut self) {
        self.save_undo_state();
        self.json_input = String::new();
        self.markdown_input = String::new();
        self.toon_input = String::new();
        self.content_cursor_line = 0;
        self.content_cursor_col = 0;
        self.scroll = 0;
        self.view_edit_mode = false;
        self.markdown_highlight_cache.clear();
        self.is_modified = true;
        self.convert_json();
        self.set_status("Content cleared");
    }

    pub fn apply_filter(&mut self, pattern: String) {
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        // Re-render with filter applied
        self.convert_json();

        let filtered_count = self.relf_entries.len();
        self.set_status(&format!("Filter: {} ({} entries)", pattern, filtered_count));
        self.filter_pattern = pattern;
    }

    pub fn clear_filter(&mut self) {
        if !self.filter_pattern.is_empty() {
            self.filter_pattern.clear();
            self.convert_json();
            self.set_status("Filter cleared");
        }
    }

    /// Update markdown highlight cache (for Edit mode)
    pub fn update_markdown_highlight_cache(&mut self) {
        if !self.is_markdown_file() {
            return;
        }

        // Ensure syntax highlighter is initialized
        if self.syntax_highlighter.is_none() {
            self.syntax_highlighter = Some(SyntaxHighlighter::new(self.colorscheme.clone()));
        }

        let lines: Vec<String> = if self.is_markdown_file() {
            self.markdown_input.lines().map(|s| s.to_string()).collect()
        } else {
            self.json_input.lines().map(|s| s.to_string()).collect()
        };

        self.markdown_highlight_cache = highlight_markdown_with_code_blocks(
            &lines,
            &self.colorscheme,
            self.syntax_highlighter.as_ref(),
        );
    }

}
