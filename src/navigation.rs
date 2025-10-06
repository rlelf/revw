pub struct Navigator;

impl Navigator {
    pub fn relf_is_header(t: &str) -> bool {
        t == "OUTSIDE" || t == "INSIDE"
    }

    pub fn relf_is_entry_start(line: &str) -> bool {
        line.starts_with("  ") && !line.starts_with("    ")
    }

    pub fn relf_is_boundary(line: &str) -> bool {
        let t = line.trim();
        t.is_empty() || Self::relf_is_header(t) || Self::relf_is_entry_start(line)
    }

    pub fn calculate_visual_lines(text_line: &str, width: usize) -> u16 {
        if width == 0 {
            return 1;
        }
        let chars: Vec<char> = text_line.chars().collect();
        if chars.is_empty() {
            return 1;
        }
        let indent = chars.iter().take_while(|c| **c == ' ').count();
        let avail = width.saturating_sub(indent).max(1);
        let content_len = chars.len().saturating_sub(indent);
        if content_len == 0 {
            return 1;
        }
        ((content_len + avail - 1) / avail).max(1) as u16
    }

    pub fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}
