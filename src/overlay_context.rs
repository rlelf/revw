use crate::rendering::Renderer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrappedRow {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WrappedCursor {
    pub visual_row: usize,
    pub visual_col: usize,
    pub row_char_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrappedTextLayout {
    pub rows: Vec<WrappedRow>,
    pub cursor: WrappedCursor,
}

pub fn total_rows(text: &str, width: usize) -> usize {
    layout_wrapped_text(text, 0, width).rows.len()
}

pub fn move_cursor_vertical(text: &str, cursor_pos: usize, width: usize, delta: isize) -> usize {
    let layout = layout_wrapped_text(text, cursor_pos, width);
    if layout.rows.is_empty() {
        return 0;
    }

    let current_row = layout.cursor.visual_row as isize;
    let max_row = layout.rows.len().saturating_sub(1) as isize;
    let target_row = (current_row + delta).clamp(0, max_row) as usize;

    if target_row == layout.cursor.visual_row {
        return cursor_pos;
    }

    let target = &layout.rows[target_row];
    let desired_col = layout.cursor.visual_col;
    let mut measured = 0;
    let mut char_offset = 0;

    for ch in target.text.chars() {
        let ch_width = Renderer::display_width_str(&ch.to_string());
        if measured + ch_width > desired_col {
            break;
        }
        measured += ch_width;
        char_offset += 1;
    }

    target.start_pos + char_offset
}

pub fn layout_wrapped_text(text: &str, cursor_pos: usize, width: usize) -> WrappedTextLayout {
    let wrap_width = width.max(1);
    let logical_lines: Vec<&str> = text.split('\n').collect();
    let mut rows = Vec::new();
    let mut line_start_pos = 0;
    let mut cursor = WrappedCursor {
        visual_row: 0,
        visual_col: 0,
        row_char_offset: 0,
    };
    let mut cursor_found = false;

    for (line_idx, line) in logical_lines.iter().enumerate() {
        let line_chars: Vec<char> = line.chars().collect();
        let line_len = line_chars.len();

        if line_chars.is_empty() {
            let row_index = rows.len();
            rows.push(WrappedRow {
                text: String::new(),
                start_pos: line_start_pos,
                end_pos: line_start_pos,
            });

            if !cursor_found && cursor_pos == line_start_pos {
                cursor = WrappedCursor {
                    visual_row: row_index,
                    visual_col: 0,
                    row_char_offset: 0,
                };
                cursor_found = true;
            }
        } else {
            let mut start_char = 0;
            while start_char < line_len {
                let mut end_char = start_char;
                let mut width_used = 0;

                while end_char < line_len {
                    let ch = line_chars[end_char];
                    let ch_width = Renderer::display_width_str(&ch.to_string());

                    if width_used > 0 && width_used + ch_width > wrap_width {
                        break;
                    }

                    width_used += ch_width;
                    end_char += 1;

                    if width_used >= wrap_width {
                        break;
                    }
                }

                let row_index = rows.len();
                let row_text: String = line_chars[start_char..end_char].iter().collect();
                let row_start_pos = line_start_pos + start_char;
                let row_end_pos = line_start_pos + end_char;
                rows.push(WrappedRow {
                    text: row_text.clone(),
                    start_pos: row_start_pos,
                    end_pos: row_end_pos,
                });

                // cursor_pos < row_end_pos: strictly inside this visual row
                // cursor_pos == row_end_pos && end_char == line_len: cursor at end of logical line (last visual row)
                // cursor_pos == row_end_pos && end_char < line_len: at start of NEXT visual row — skip
                if !cursor_found
                    && cursor_pos >= row_start_pos
                    && (cursor_pos < row_end_pos || (cursor_pos == row_end_pos && end_char == line_len))
                {
                    let row_char_offset = cursor_pos.saturating_sub(row_start_pos);
                    let prefix: String = row_text.chars().take(row_char_offset).collect();
                    cursor = WrappedCursor {
                        visual_row: row_index,
                        visual_col: Renderer::display_width_str(&prefix),
                        row_char_offset,
                    };
                    cursor_found = true;
                }

                start_char = end_char;
            }
        }

        line_start_pos += line_len;
        if line_idx < logical_lines.len() - 1 {
            line_start_pos += 1;
        }
    }

    if rows.is_empty() {
        rows.push(WrappedRow {
            text: String::new(),
            start_pos: 0,
            end_pos: 0,
        });
    }

    if !cursor_found {
        let last_row_index = rows.len() - 1;
        let last_row = &rows[last_row_index];
        cursor = WrappedCursor {
            visual_row: last_row_index,
            visual_col: Renderer::display_width_str(&last_row.text),
            row_char_offset: last_row.text.chars().count(),
        };
    }

    WrappedTextLayout { rows, cursor }
}
