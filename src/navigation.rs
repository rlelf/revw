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
        if width == 0 { return 1; }
        let chars: Vec<char> = text_line.chars().collect();
        if chars.is_empty() { return 1; }
        let indent = chars.iter().take_while(|c| **c == ' ').count();
        let avail = width.saturating_sub(indent).max(1);
        let content_len = chars.len().saturating_sub(indent);
        if content_len == 0 { return 1; }
        ((content_len + avail - 1) / avail).max(1) as u16
    }

    pub fn calculate_cursor_visual_position(
        lines: &[String], 
        cursor_line: usize, 
        cursor_col: usize,
        width: usize
    ) -> (u16, u16) {
        if lines.is_empty() || cursor_line >= lines.len() {
            return (cursor_line as u16, 0);
        }
        
        let mut visual_line = 0u16;
        
        for i in 0..cursor_line {
            if i < lines.len() {
                visual_line += Self::calculate_visual_lines(&lines[i], width);
            }
        }
        
        if cursor_line < lines.len() {
            let current_line = &lines[cursor_line];
            let chars: Vec<char> = current_line.chars().collect();
            let indent = chars.iter().take_while(|c| **c == ' ').count();
            let avail = width.saturating_sub(indent).max(1);

            let cursor_col = cursor_col.min(chars.len());
            if cursor_col <= indent {
                return (visual_line, cursor_col as u16);
            }
            let pos_in_content = cursor_col - indent;
            let extra_lines = (pos_in_content) / avail;
            let col_in_seg = (pos_in_content) % avail;
            visual_line += extra_lines as u16;
            return (visual_line, (indent + col_in_seg) as u16);
        }

        (visual_line, 0)
    }

    pub fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}