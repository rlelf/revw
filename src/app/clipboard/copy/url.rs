use super::super::super::{App, FormatMode};
use arboard::Clipboard;

impl App {
    /// Copy URL from selected entry to clipboard
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
}
