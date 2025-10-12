use std::fs;
use std::path::PathBuf;
use super::colorscheme::ColorScheme;

#[derive(Debug, Clone)]
pub struct RcConfig {
    pub show_line_numbers: bool,
    pub colorscheme: ColorScheme,
}

impl Default for RcConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            colorscheme: ColorScheme::default(),
        }
    }
}

impl RcConfig {
    /// Load configuration from ~/.revwrc
    pub fn load() -> Self {
        let mut config = Self::default();

        if let Some(rc_path) = Self::get_rc_path() {
            if let Ok(contents) = fs::read_to_string(&rc_path) {
                config.parse(&contents);
            }
        }

        config
    }

    /// Get the path to ~/.revwrc
    fn get_rc_path() -> Option<PathBuf> {
        dirs::home_dir().map(|mut path| {
            path.push(".revwrc");
            path
        })
    }

    /// Parse RC file contents
    fn parse(&mut self, contents: &str) {
        for line in contents.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with('"') {
                continue;
            }

            self.parse_line(line);
        }
    }

    /// Parse a single line
    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "set" => {
                if parts.len() >= 2 {
                    self.handle_set(&parts[1..]);
                }
            }
            "colorscheme" => {
                if parts.len() >= 2 {
                    self.handle_colorscheme(parts[1]);
                }
            }
            _ => {
                // Unknown command, ignore
            }
        }
    }

    /// Handle 'set' command
    fn handle_set(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        let option = args[0];

        match option {
            "number" | "nu" => {
                self.show_line_numbers = true;
            }
            "nonumber" | "nonu" => {
                self.show_line_numbers = false;
            }
            _ => {
                // Unknown option, ignore
            }
        }
    }

    /// Handle 'colorscheme' command
    fn handle_colorscheme(&mut self, name: &str) {
        if let Some(scheme) = ColorScheme::by_name(name) {
            self.colorscheme = scheme;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_set_number() {
        let mut config = RcConfig::default();
        config.parse("set number");
        assert!(config.show_line_numbers);
    }

    #[test]
    fn test_parse_set_nonumber() {
        let mut config = RcConfig::default();
        config.show_line_numbers = true;
        config.parse("set nonumber");
        assert!(!config.show_line_numbers);
    }

    #[test]
    fn test_parse_colorscheme() {
        let mut config = RcConfig::default();
        config.parse("colorscheme Monokai");
        assert_eq!(config.colorscheme.name, "Monokai");
    }

    #[test]
    fn test_parse_comments() {
        let mut config = RcConfig::default();
        config.parse("# This is a comment\nset number");
        assert!(config.show_line_numbers);
    }

    #[test]
    fn test_parse_multiline() {
        let mut config = RcConfig::default();
        let rc_contents = r#"
            # My revw config
            set number
            colorscheme Nord
        "#;
        config.parse(rc_contents);
        assert!(config.show_line_numbers);
        assert_eq!(config.colorscheme.name, "Nord");
    }
}
