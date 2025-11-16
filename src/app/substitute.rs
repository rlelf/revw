use super::{App, FormatMode, SubstituteMatch};

impl App {
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

        let lines = self.get_content_lines();

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
        let mut lines = self.get_content_lines();
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
            self.set_content_from_lines(lines);
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
            let mut lines = self.get_content_lines();
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
            self.set_content_from_lines(lines);

            self.substitute_confirmations.clear();
            self.current_substitute_index = 0;
            self.is_modified = true;
            self.set_status(&format!("{} substitution{} made", remaining_count, if remaining_count == 1 { "" } else { "s" }));
        } else {
            if should_substitute {
                // Perform this substitution
                let match_item = &self.substitute_confirmations[self.current_substitute_index];
                let mut lines = self.get_content_lines();

                if match_item.line < lines.len() {
                    let line = &mut lines[match_item.line];
                    if match_item.col + match_item.pattern.len() <= line.len() {
                        line.replace_range(
                            match_item.col..match_item.col + match_item.pattern.len(),
                            &match_item.replacement,
                        );
                        self.set_content_from_lines(lines);
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
