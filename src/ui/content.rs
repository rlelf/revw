use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FormatMode, InputMode};

use super::json_highlight::highlight_json_line;
use super::utils::{apply_relf_style, slice_spans_by_width};

pub fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    // In View mode with entries, render as cards
    if app.format_mode == FormatMode::View && !app.relf_entries.is_empty() {
        super::cards::render_relf_cards(f, app, area);
        return;
    }

    // In Help mode, render help text
    if app.format_mode == FormatMode::Help {
        render_help_content(f, app, area);
        return;
    }

    let inner_area = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    // Update the app's notion of the current content width for accurate wrapping
    // Use inner area width (inside borders and margins)
    app.content_width = inner_area.width;
    // In View mode, disable horizontal scrolling entirely
    if app.format_mode == FormatMode::View {
        app.hscroll = 0;
    }
    // Remember actual visible height for correct scroll math elsewhere
    app.visible_height = inner_area.height;
    // Build visual (wrapped) lines and compute scroll bounds in visual rows
    let visual_lines = app.build_visual_lines();
    let lines_count = visual_lines.len() as u16;
    let visible_height = inner_area.height;
    let bottom_padding = 10u16; // Allow scrolling past end
    let padded_lines_count = lines_count + bottom_padding;
    app.max_scroll = padded_lines_count.saturating_sub(visible_height);

    let empty_line = String::new();
    let visible_content: Vec<_> = visual_lines
        .iter()
        .skip(app.scroll as usize)
        .chain(std::iter::repeat(&empty_line).take(bottom_padding as usize))
        .take(visible_height as usize)
        .collect();

    // Build content with cursor and horizontal viewport
    let content_text = {
        let w_cols = app.get_content_width() as usize;
        let off_cols = if app.format_mode == FormatMode::View {
            0
        } else {
            app.hscroll as usize
        };
        let mut lines_vec: Vec<Line> = Vec::new();

        for (line_idx, s) in visible_content.iter().enumerate() {
            let actual_idx = line_idx + app.scroll as usize;

            // Add line numbers if enabled in Edit mode
            let (line_number_prefix, adjusted_w_cols) = if app.format_mode == FormatMode::Edit && app.show_line_numbers {
                let total_lines = visual_lines.len();
                let line_num_width = format!("{}", total_lines).len().max(3);
                let line_num_str = if actual_idx < total_lines {
                    format!("{:>width$} ", actual_idx + 1, width = line_num_width)
                } else {
                    " ".repeat(line_num_width + 1)
                };
                let adjusted_width = w_cols.saturating_sub(line_num_width + 1);
                (line_num_str, adjusted_width)
            } else {
                (String::new(), w_cols)
            };

            let slice = app.slice_columns(s, off_cols, adjusted_w_cols);

            // Build spans for the line with search highlighting
            let mut line_number_span: Option<Span> = None;

            // Add line number span if present
            if !line_number_prefix.is_empty() {
                line_number_span = Some(Span::styled(
                    line_number_prefix,
                    Style::default().fg(app.colorscheme.line_number),
                ));
            }
            let line_style = if app.format_mode == FormatMode::View {
                app.relf_visual_styles.get(actual_idx)
            } else {
                None
            };

            let mut content_spans: Vec<Span> = Vec::new();

            if !app.search_query.is_empty() && app.format_mode == FormatMode::Edit {
                // In Edit mode with search: apply JSON highlighting to full line first
                let json_spans = highlight_json_line(s, &app.colorscheme);

                // Merge JSON highlighting with search match backgrounds on full line
                let query_lower = app.search_query.to_lowercase();
                let line_lower = s.to_lowercase();
                let mut result_spans: Vec<Span> = Vec::new();
                let mut char_pos = 0;

                for json_span in json_spans {
                    let span_text = json_span.content.to_string();
                    let span_len = span_text.len();
                    let span_start = char_pos;
                    let span_end = char_pos + span_len;

                    // Check if this span overlaps with any search match
                    let mut last_split = 0;

                    while let Some(match_pos) = line_lower[span_start..span_end].find(&query_lower) {
                        let abs_match_pos = span_start + match_pos;

                        if abs_match_pos < span_start + last_split {
                            break;
                        }

                        let rel_match_start = abs_match_pos - span_start;
                        let rel_match_end = (abs_match_pos + app.search_query.len()).min(span_end) - span_start;

                        // Check if this is the current match
                        let is_current_match = app
                            .current_match_index
                            .and_then(|idx| app.search_matches.get(idx))
                            .map(|(line, col)| *line == actual_idx && *col == abs_match_pos)
                            .unwrap_or(false);

                        let bg_color = if is_current_match {
                            Color::Rgb(255, 255, 150) // Light yellow
                        } else {
                            Color::Rgb(100, 180, 200) // Light cyan
                        };

                        // Add text before match (with original JSON color)
                        if rel_match_start > last_split {
                            result_spans.push(Span::styled(
                                span_text[last_split..rel_match_start].to_string(),
                                json_span.style,
                            ));
                        }

                        // Add matched text with background
                        result_spans.push(Span::styled(
                            span_text[rel_match_start..rel_match_end].to_string(),
                            json_span.style.bg(bg_color),
                        ));

                        last_split = rel_match_end;
                    }

                    // Add remaining text from this span
                    if last_split < span_len {
                        result_spans.push(Span::styled(
                            span_text[last_split..].to_string(),
                            json_span.style,
                        ));
                    }

                    char_pos = span_end;
                }

                // Slice the result spans to visible range
                content_spans = slice_spans_by_width(app, result_spans, off_cols, adjusted_w_cols);
            } else if !app.search_query.is_empty() {
                // View mode with search: original search highlighting logic
                let query_lower = app.search_query.to_lowercase();
                let line_lower = slice.to_lowercase();
                let mut last_pos = 0;

                while let Some(match_pos) = line_lower[last_pos..].find(&query_lower) {
                    let actual_pos = last_pos + match_pos;

                    // Add text before match
                    if actual_pos > last_pos {
                        content_spans.push(Span::styled(
                            slice[last_pos..actual_pos].to_string(),
                            apply_relf_style(Style::default().fg(app.colorscheme.text), line_style),
                        ));
                    }

                    // Check if this is the current match
                    let is_current_match = app
                        .current_match_index
                        .and_then(|idx| app.search_matches.get(idx))
                        .map(|(line, col)| *line == actual_idx && *col == actual_pos + off_cols)
                        .unwrap_or(false);

                    // Add highlighted match
                    let match_end = actual_pos + app.search_query.len();
                    let highlight_style = if is_current_match {
                        Style::default().fg(Color::Black).bg(Color::Yellow) // Current match
                    } else {
                        Style::default().fg(Color::Black).bg(Color::Cyan) // Other matches
                    };

                    content_spans.push(Span::styled(
                        slice[actual_pos..match_end.min(slice.len())].to_string(),
                        highlight_style,
                    ));

                    last_pos = match_end;
                }

                // Add remaining text after last match
                if last_pos < slice.len() {
                    content_spans.push(Span::styled(
                        slice[last_pos..].to_string(),
                        apply_relf_style(Style::default().fg(app.colorscheme.text), line_style),
                    ));
                }
            } else {
                // No search highlighting
                if app.format_mode == FormatMode::Edit {
                    // Apply JSON syntax highlighting to full line, then slice
                    let full_line_spans = highlight_json_line(s, &app.colorscheme);
                    content_spans = slice_spans_by_width(app, full_line_spans, off_cols, adjusted_w_cols);
                } else {
                    // In View mode, use plain text with line style
                    content_spans.push(Span::styled(
                        slice.clone(),
                        apply_relf_style(Style::default().fg(app.colorscheme.text), line_style),
                    ));
                }
            }

            // Add cursor if needed
            if app.format_mode == FormatMode::Edit
                && (app.input_mode == InputMode::Insert || app.input_mode == InputMode::Normal)
                && app.show_cursor
            {
                if actual_idx == app.content_cursor_line {
                    let cursor_char_pos = app.content_cursor_col;
                    let prefix_cols = app.prefix_display_width(s, cursor_char_pos);
                    if prefix_cols >= off_cols && prefix_cols < off_cols + adjusted_w_cols {
                        // Insert cursor while preserving existing highlighting
                        let insert_col_in_view = prefix_cols - off_cols;

                        // Calculate display width position across all spans
                        let mut display_width_count = 0;
                        let mut cursor_inserted = false;
                        let mut new_spans: Vec<Span> = Vec::new();

                        for span in content_spans.iter() {
                            let span_text = span.content.to_string();
                            let span_display_width = app.display_width_str(&span_text);

                            if !cursor_inserted && display_width_count + span_display_width >= insert_col_in_view {
                                // Cursor belongs in this span
                                // Find the character position within this span
                                let target_width_in_span = insert_col_in_view - display_width_count;

                                let span_chars: Vec<char> = span_text.chars().collect();
                                let mut pos_in_span = 0;
                                let mut accumulated_width = 0;

                                for (i, ch) in span_chars.iter().enumerate() {
                                    // Check if adding this character would exceed target
                                    let ch_width = app.display_width_str(&ch.to_string());
                                    if accumulated_width + ch_width > target_width_in_span {
                                        // Cursor should be placed before this character
                                        pos_in_span = i;
                                        break;
                                    }
                                    accumulated_width += ch_width;
                                    pos_in_span = i + 1;
                                }

                                // Split span at cursor position
                                if pos_in_span == 0 {
                                    // Cursor at start
                                    new_spans.push(Span::styled("│".to_string(), span.style));
                                    new_spans.push(span.clone());
                                } else if pos_in_span >= span_chars.len() {
                                    // Cursor at end
                                    new_spans.push(span.clone());
                                    new_spans.push(Span::styled("│".to_string(), span.style));
                                } else {
                                    // Cursor in middle
                                    let before: String = span_chars[..pos_in_span].iter().collect();
                                    let after: String = span_chars[pos_in_span..].iter().collect();

                                    new_spans.push(Span::styled(before, span.style));
                                    new_spans.push(Span::styled("│".to_string(), span.style));
                                    new_spans.push(Span::styled(after, span.style));
                                }
                                cursor_inserted = true;
                            } else {
                                new_spans.push(span.clone());
                            }

                            display_width_count += span_display_width;
                        }

                        // If cursor wasn't inserted yet, add it at the end
                        if !cursor_inserted {
                            let last_style = content_spans.last().map(|s| s.style).unwrap_or_default();
                            new_spans.push(Span::styled("│".to_string(), last_style));
                        }

                        content_spans = new_spans;
                    }
                }
            }

            // Combine line number and content spans
            let mut spans: Vec<Span> = Vec::new();
            if let Some(line_num_span) = line_number_span {
                spans.push(line_num_span);
            }
            spans.extend(content_spans);

            if spans.is_empty() {
                spans.push(Span::styled(
                    String::new(),
                    apply_relf_style(Style::default(), line_style),
                ));
            }

            lines_vec.push(Line::from(spans));
        }

        lines_vec
    };

    let title = match &app.file_path {
        Some(path) => {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Remove extension if show_extension is false
                let display_name = if !app.show_extension {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        stem.to_string()
                    } else {
                        name.to_string()
                    }
                } else {
                    name.to_string()
                };
                format!(" {} ", display_name)
            } else {
                String::new()
            }
        }
        None => String::new(),
    };

    let content = Paragraph::new(content_text).block(
        Block::default()
            .title(title)
            .title_style(Style::default().fg(app.colorscheme.window_title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.colorscheme.window_border))
            .style(Style::default().bg(app.colorscheme.background)),
    );

    f.render_widget(content, area);
}

fn render_help_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Create a block with border like View/Edit modes
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.colorscheme.window_border))
        .style(Style::default().bg(app.colorscheme.background));

    let inner_area = block.inner(area);

    app.visible_height = inner_area.height;
    app.content_width = inner_area.width;

    // Calculate visible range
    let total_lines = app.rendered_content.len();
    let visible_height = inner_area.height as usize;
    let scroll_pos = app.scroll as usize;

    // Update max_scroll
    app.max_scroll = if total_lines > visible_height {
        (total_lines - visible_height) as u16
    } else {
        0
    };

    // Clamp scroll
    if app.scroll > app.max_scroll {
        app.scroll = app.max_scroll;
    }

    // Get visible lines
    let start = scroll_pos;
    let end = (start + visible_height).min(total_lines);
    let visible_lines: Vec<Line> = app.rendered_content[start..end]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();

    let content = Paragraph::new(visible_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White).bg(app.colorscheme.background));

    f.render_widget(content, area);
}
