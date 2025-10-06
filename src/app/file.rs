use super::App;
use std::{{fs, path::PathBuf, time::Instant}};

impl App {
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

}
