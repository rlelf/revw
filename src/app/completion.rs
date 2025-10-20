use super::{App, ColorScheme};
use std::path::PathBuf;

impl App {
    // Command completion with Tab key - cycles through candidates
    pub fn complete_command(&mut self) {
        let cmd = self.command_buffer.trim().to_string();

        // If we have active candidates and command hasn't changed, cycle to next one
        if !self.completion_candidates.is_empty() &&
           self.completion_candidates.iter().any(|c| c == &cmd) {
            self.completion_index = (self.completion_index + 1) % self.completion_candidates.len();
            self.command_buffer = self.completion_candidates[self.completion_index].clone();
            self.set_status(&format!(":{}", self.command_buffer));
            return;
        }

        // Otherwise, find new candidates
        self.completion_original = cmd.clone();
        self.completion_candidates.clear();

        // Handle colorscheme completion
        if cmd.starts_with("colorscheme") {
            let partial = if cmd == "colorscheme" {
                ""
            } else if cmd.starts_with("colorscheme ") {
                cmd.strip_prefix("colorscheme ").unwrap_or("")
            } else {
                ""
            };

            let schemes = ColorScheme::all_scheme_names();
            let mut matches: Vec<String> = schemes.iter()
                .filter(|s| {
                    if partial.is_empty() {
                        true  // Show all schemes if no partial input
                    } else {
                        s.to_lowercase().starts_with(&partial.to_lowercase())
                    }
                })
                .map(|s| format!("colorscheme {}", s))
                .collect();

            if !matches.is_empty() {
                matches.sort();
                self.completion_candidates = matches;
                self.completion_index = 0;
                self.command_buffer = self.completion_candidates[0].clone();
                self.set_status(&format!(":{}", self.command_buffer));
            }
        }
        // Handle :e file completion
        else if cmd == "e" || cmd.starts_with("e ") {
            let partial = cmd.strip_prefix("e ").unwrap_or("");
            self.complete_file_path(partial);
        }
        // Handle command name completion
        else {
            let commands = vec![
                "w", "wq", "q", "e", "ai", "ao", "o", "op", "on", "dd", "yy",
                "c", "ci", "co", "cu", "v", "vu", "vi", "vo", "va", "vai", "vao",
                "xi", "xo", "gi", "go", "noh", "nof", "f", "cc", "ccj", "dc",
                "set", "colorscheme", "ar", "h", "a", "d", "m",
                "Lexplore", "Lex", "lx",
            ];

            let mut matches: Vec<String> = commands.iter()
                .filter(|c| c.starts_with(cmd.as_str()))
                .map(|s| s.to_string())
                .collect();

            if !matches.is_empty() {
                matches.sort();
                self.completion_candidates = matches;
                self.completion_index = 0;
                self.command_buffer = self.completion_candidates[0].clone();
                self.set_status(&format!(":{}", self.command_buffer));
            }
        }
    }

    // Reset completion state when command buffer changes
    pub fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = 0;
        self.completion_original.clear();
    }

    fn complete_file_path(&mut self, partial: &str) {
        use std::fs;

        // Determine the directory and filename part
        let path_buf = if partial.is_empty() {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        } else {
            PathBuf::from(partial)
        };

        let (dir, file_prefix) = if partial.ends_with('/') || partial.ends_with('\\') || partial.is_empty() {
            (path_buf.clone(), String::new())
        } else if path_buf.is_dir() {
            (path_buf.clone(), String::new())
        } else {
            let dir = path_buf.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            let file_prefix = path_buf.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (dir, file_prefix)
        };

        // Read directory and find matching files
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut matches: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.to_lowercase().starts_with(&file_prefix.to_lowercase()) {
                        let full_path = if dir == PathBuf::from(".") {
                            name.clone()
                        } else {
                            dir.join(&name).to_string_lossy().to_string()
                        };
                        Some(format!("e {}", full_path))
                    } else {
                        None
                    }
                })
                .collect();

            matches.sort();

            if !matches.is_empty() {
                self.completion_candidates = matches;
                self.completion_index = 0;
                self.command_buffer = self.completion_candidates[0].clone();
                self.set_status(&format!(":{}", self.command_buffer));
            }
        }
    }
}
