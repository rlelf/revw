use super::super::super::App;
use arboard::Clipboard;

impl App {
    /// Copy content as JSON format
    pub fn copy_json(&mut self) {
        // Copy current content as JSON (works in both Edit and View modes)
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(self.json_input.clone()) {
                Ok(()) => self.set_status("Copied as JSON"),
                Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
        }
    }

    /// Copy content as Markdown format
    pub fn copy_markdown(&mut self) {
        // Copy current content as Markdown (works in both Edit and View modes)
        match self.convert_to_markdown() {
            Ok(markdown_content) => {
                match Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(markdown_content) {
                        Ok(()) => self.set_status("Copied as Markdown"),
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                }
            }
            Err(e) => self.set_status(&format!("Failed to convert to Markdown: {}", e)),
        }
    }

    /// Copy content as Toon format
    pub fn copy_toon(&mut self) {
        // Copy current content as Toon (works in both Edit and View modes)
        match self.convert_to_toon() {
            Ok(toon_content) => {
                match Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(toon_content) {
                        Ok(()) => self.set_status("Copied as Toon"),
                        Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                    },
                    Err(e) => self.set_status(&format!("Clipboard error: {}", e)),
                }
            }
            Err(e) => self.set_status(&format!("Failed to convert to Toon: {}", e)),
        }
    }
}
