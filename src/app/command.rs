use super::{App, FormatMode};
use std::path::PathBuf;

impl App {
    pub fn handle_vim_input(&mut self, c: char) -> bool {
        self.vim_buffer.push(c);

        // Handle explorer-specific commands
        if self.explorer_open && self.explorer_has_focus {
            if self.vim_buffer == "go" {
                // Preview file without moving focus (NERDTree-like)
                self.explorer_preview_entry();
                self.vim_buffer.clear();
                return true;
            }
        }

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
        } else if self.vim_buffer == "g-" {
            // Undo (vim-style, not in help mode)
            if !self.showing_help && self.format_mode == FormatMode::Edit {
                self.undo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer == "g+" {
            // Redo (vim-style, not in help mode)
            if !self.showing_help && self.format_mode == FormatMode::Edit {
                self.redo();
            }
            self.vim_buffer.clear();
            return true;
        } else if self.vim_buffer.len() >= 2 {
            self.vim_buffer.clear();
        }

        false
    }

    pub fn execute_command(&mut self) -> bool {
        let cmd = self.command_buffer.clone();
        let cmd = cmd.trim();

        // Handle explorer-specific commands when explorer has focus
        if self.explorer_open && self.explorer_has_focus {
            if cmd == "a" {
                self.explorer_create_file();
                return false;
            } else if cmd == "d" {
                self.explorer_create_dir();
                return false;
            } else if cmd == "m" {
                self.explorer_rename_file();
                return false;
            } else if cmd == "dd" {
                self.explorer_delete_file();
                return false;
            } else if cmd == "yy" {
                self.explorer_copy_file();
                return false;
            }
        }

        if cmd == "w" {
            self.save_file();
        } else if cmd == "wq" {
            self.save_file();
            return true; // Signal to quit
        } else if cmd == "q" {
            // If explorer has focus, close explorer instead of quitting
            if self.explorer_open && self.explorer_has_focus {
                self.explorer_open = false;
                self.explorer_has_focus = false;
                return false; // Don't quit
            } else {
                return true; // Signal to quit
            }
        } else if cmd.starts_with("w ") {
            let filename = cmd.strip_prefix("w ").unwrap().trim().to_string();
            if !filename.ends_with(".json") {
                self.set_status("Error: Filename must end with .json");
            } else {
                self.save_file_as(&filename);
            }
        } else if cmd.starts_with("wq ") {
            let filename = cmd.strip_prefix("wq ").unwrap().trim().to_string();
            if !filename.ends_with(".json") {
                self.set_status("Error: Filename must end with .json");
                return false; // Don't quit on error
            } else {
                self.save_file_as(&filename);
                return true; // Signal to quit
            }
        } else if cmd == "e" {
            // Refresh/reload the file
            self.reload_file();
        } else if cmd.starts_with("e ") {
            // Open a different file
            let filename = cmd.strip_prefix("e ").unwrap().trim().to_string();
            if !filename.ends_with(".json") {
                self.set_status("Error: Filename must end with .json");
            } else {
                let path = PathBuf::from(filename);
                self.load_file(path);
            }
        } else if cmd == "enew" {
            // Clear file window (like vim :enew)
            self.json_input = String::new();
            self.file_path = None;
            self.file_path_changed = false;
            self.is_modified = false;
            self.content_cursor_line = 0;
            self.content_cursor_col = 0;
            self.scroll = 0;
            self.convert_json();
            self.set_status("New empty buffer");
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
        } else if cmd == "op" {
            // Order by percentage only
            self.order_by_percentage();
        } else if cmd == "on" {
            // Order by name only
            self.order_by_name();
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
        } else if cmd == "cc" {
            // Copy card(s) with rendering
            self.copy_cards_rendered();
        } else if cmd == "ccj" {
            // Copy card(s) as JSON
            self.copy_cards_json();
        } else if cmd == "dc" {
            // Delete card(s)
            self.delete_cards();
        } else if cmd == "vu" {
            // Paste URL from clipboard to selected entry
            self.paste_url_to_selected();
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
        } else if cmd == "dd" {
            // Delete entry in both View and Edit modes
            if self.format_mode == FormatMode::Edit {
                self.delete_current_entry();
                self.is_modified = true;
            } else if !self.relf_entries.is_empty() {
                self.delete_selected_entry();
                self.is_modified = true;
                // Auto-save after deletion in View mode
                self.save_file();
            }
        } else if cmd == "yy" {
            // Duplicate entry in both View and Edit modes
            self.duplicate_selected_entry();
        } else if cmd == "noh" {
            // Clear search highlighting
            self.clear_search_highlight();
        } else if cmd == "nof" {
            // Clear filter
            self.clear_filter();
        } else if cmd.starts_with("f ") {
            // Filter entries in View mode
            if self.format_mode == FormatMode::View {
                let pattern = cmd.strip_prefix("f ").unwrap().trim().to_string();
                self.apply_filter(pattern);
            } else {
                self.set_status("Filter only works in View mode");
            }
        } else if cmd == "Lexplore" || cmd == "Lex" || cmd == "lx" {
            // Toggle file explorer (like vim netrw)
            self.toggle_explorer();
        } else if cmd == "h" {
            self.toggle_help();
        } else if cmd == "c" {
            // Copy all content to clipboard
            self.copy_to_clipboard();
        } else if cmd == "v" {
            // Paste from clipboard
            self.paste_from_clipboard();
        } else if cmd == "x" {
            // Clear all content
            self.clear_content();
        } else if cmd.starts_with("s/") || cmd.starts_with("%s/") {
            // Substitute command: :s/pattern/replacement/flags or :%s/pattern/replacement/flags
            self.execute_substitute(cmd);
        } else if cmd == "set number" || cmd == "set nu" {
            // Enable line numbers
            self.show_line_numbers = true;
            self.set_status("Line numbers enabled");
        } else if cmd == "set nonumber" || cmd == "set nonu" {
            // Disable line numbers
            self.show_line_numbers = false;
            self.set_status("Line numbers disabled");
        } else if cmd.starts_with("set card=") {
            // Set maximum visible cards
            if let Some(value_str) = cmd.strip_prefix("set card=") {
                if let Ok(value) = value_str.trim().parse::<usize>() {
                    if value >= 1 && value <= 10 {
                        self.max_visible_cards = value;
                        self.set_status(&format!("Max visible cards set to {}", value));
                    } else {
                        self.set_status("Card value must be between 1 and 10");
                    }
                } else {
                    self.set_status("Invalid card value");
                }
            }
        } else if cmd.starts_with("colorscheme ") {
            // Change color scheme
            use super::ColorScheme;
            let scheme_name = cmd.strip_prefix("colorscheme ").unwrap().trim();
            if let Some(scheme) = ColorScheme::by_name(scheme_name) {
                self.colorscheme = scheme;
                self.set_status(&format!("Color scheme changed to {}", scheme_name));
            } else {
                self.set_status(&format!("Unknown color scheme: {}", scheme_name));
            }
        } else {
            self.set_status(&format!("Unknown command: {}", cmd));
        }
        false // Don't quit
    }

}
