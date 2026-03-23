use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FormatMode, InputMode};
use crate::overlay_context::layout_wrapped_text;

use super::json_highlight::highlight_json_line;
use super::markdown_highlight::highlight_markdown_line;
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

    // Edit mode: use self-made wrap (like overlay) for proper visual-row navigation
    if app.format_mode == FormatMode::Edit {
        render_edit_wrapped(f, app, area);
        return;
    }

    let inner_area = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    // Update the app's notion of the current content width for accurate wrapping
    // Use inner area width (inside borders and margins)
    app.content_width = inner_area.width;
    // Disable horizontal scrolling in both View mode and Edit mode (Edit uses wrapping)
    app.hscroll = 0;
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
        let off_cols = app.hscroll as usize; // always 0 (wrapping handles overflow)
        let mut lines_vec: Vec<Line> = Vec::new();

        for (line_idx, s) in visible_content.iter().enumerate() {
            let actual_idx = line_idx + app.scroll as usize;

            // Add line numbers if enabled in Edit mode
            let (line_number_prefix, adjusted_w_cols) = if app.format_mode == FormatMode::Edit && app.show_line_numbers {
                let total_lines = visual_lines.len();
                let line_num_width = format!("{}", total_lines).len().max(3);
                let line_num_str = if actual_idx < total_lines {
                    if app.show_relative_line_numbers {
                        // Relative line numbers: show absolute for current line, relative distance for others
                        let cursor_line = app.content_cursor_line;
                        if actual_idx == cursor_line {
                            // Current line shows absolute number
                            format!("{:>width$} ", actual_idx + 1, width = line_num_width)
                        } else {
                            // Other lines show relative distance
                            let distance = (actual_idx as isize - cursor_line as isize).unsigned_abs();
                            format!("{:>width$} ", distance, width = line_num_width)
                        }
                    } else {
                        // Absolute line numbers only
                        format!("{:>width$} ", actual_idx + 1, width = line_num_width)
                    }
                } else {
                    " ".repeat(line_num_width + 1)
                };
                // Use a large value so spans aren't truncated; Wrap handles the overflow
                let _ = w_cols.saturating_sub(line_num_width + 1);
                (line_num_str, usize::MAX / 4)
            } else {
                (String::new(), usize::MAX / 4)
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
                // In Edit mode with search: apply syntax highlighting to full line first
                let json_spans = if app.is_markdown_file() {
                    // Use cached highlight if available
                    if actual_idx < app.markdown_highlight_cache.len() {
                        app.markdown_highlight_cache[actual_idx].clone()
                    } else {
                        highlight_markdown_line(s, &app.colorscheme)
                    }
                } else {
                    highlight_json_line(s, &app.colorscheme)
                };

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

                    // Ensure span_end doesn't exceed line_lower length
                    let safe_span_end = span_end.min(line_lower.len());
                    if span_start >= line_lower.len() {
                        // Skip this span if start is already out of bounds
                        result_spans.push(json_span);
                        char_pos += span_len;
                        continue;
                    }

                    while let Some(match_pos) = line_lower[span_start..safe_span_end].find(&query_lower) {
                        let abs_match_pos = span_start + match_pos;

                        if abs_match_pos < span_start + last_split {
                            break;
                        }

                        let rel_match_start = abs_match_pos - span_start;
                        let rel_match_end = (abs_match_pos + app.search_query.len()).min(span_end) - span_start;

                        // Ensure indices are within span_text bounds and on UTF-8 char boundaries
                        let safe_rel_match_start = rel_match_start.min(span_text.len());
                        let safe_rel_match_end = rel_match_end.min(span_text.len());

                        // Validate UTF-8 char boundaries
                        if !span_text.is_char_boundary(safe_rel_match_start)
                            || !span_text.is_char_boundary(safe_rel_match_end)
                            || !span_text.is_char_boundary(last_split) {
                            break;
                        }

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
                        if safe_rel_match_start > last_split {
                            result_spans.push(Span::styled(
                                span_text[last_split..safe_rel_match_start].to_string(),
                                json_span.style,
                            ));
                        }

                        // Add matched text with background
                        result_spans.push(Span::styled(
                            span_text[safe_rel_match_start..safe_rel_match_end].to_string(),
                            json_span.style.bg(bg_color),
                        ));

                        last_split = safe_rel_match_end;
                    }

                    // Add remaining text from this span
                    if last_split < span_len && span_text.is_char_boundary(last_split) {
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
                    // Apply syntax highlighting to full line, then slice
                    let full_line_spans = if app.is_markdown_file() {
                        // Use cached highlight if available
                        if actual_idx < app.markdown_highlight_cache.len() {
                            app.markdown_highlight_cache[actual_idx].clone()
                        } else {
                            highlight_markdown_line(s, &app.colorscheme)
                        }
                    } else {
                        highlight_json_line(s, &app.colorscheme)
                    };
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
                    if prefix_cols >= off_cols {
                        // Insert cursor while preserving existing highlighting
                        // (with wrapping enabled, cursor may be on a wrapped row - ratatui places it correctly)
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

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.colorscheme.window_title))
        .borders(Borders::ALL)
        .border_type(app.border_style.to_border_type())
        .border_style(Style::default().fg(app.colorscheme.window_border))
        .style(Style::default().bg(app.colorscheme.background));

    let content = if app.format_mode == FormatMode::Edit {
        // Edit mode: wrap long lines instead of horizontal scrolling
        Paragraph::new(content_text).block(block).wrap(Wrap { trim: false })
    } else {
        Paragraph::new(content_text).block(block)
    };

    f.render_widget(content, area);
}

/// Edit mode rendering using self-made wrap (like overlay_context), so j/k move by
/// visual rows and line numbers are correctly accounted for in the wrap width.
fn render_edit_wrapped(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = area.inner(Margin { horizontal: 1, vertical: 1 });
    app.content_width = inner_area.width;
    app.visible_height = inner_area.height;
    app.hscroll = 0;

    // --- Compute line-number gutter width ---
    let lines = app.get_content_lines();
    let total_logical = lines.len().max(1);
    let (gutter_width, content_wrap_width) = if app.show_line_numbers {
        let g = format!("{}", total_logical).len().max(3) + 1;
        // Reserve 1 column so the cursor does not cover the last visible char
        (g, (inner_area.width as usize).saturating_sub(g + 1))
    } else {
        // Reserve 1 column so the cursor does not cover the last visible char
        (0, (inner_area.width as usize).saturating_sub(1))
    };

    // --- Build flat content string and layout ---
    let flat_content = lines.join("\n");
    let flat_cursor = app.cursor_flat_pos();
    let wrap_width = content_wrap_width.max(1);
    let layout = layout_wrapped_text(&flat_content, flat_cursor, wrap_width);

    let total_vis_rows = layout.rows.len();
    let vis_height = inner_area.height as usize;
    let bottom_padding = 10usize;
    app.max_scroll = (total_vis_rows + bottom_padding).saturating_sub(vis_height) as u16;
    if app.scroll > app.max_scroll {
        app.scroll = app.max_scroll;
    }

    // --- Pre-compute logical line start positions (flat char offsets) ---
    let mut line_starts: Vec<usize> = Vec::with_capacity(lines.len());
    {
        let mut pos = 0usize;
        for l in &lines {
            line_starts.push(pos);
            pos += l.chars().count() + 1;
        }
    }

    // --- Render visible visual rows ---
    let vscroll = app.scroll as usize;
    let cursor_vis_row = layout.cursor.visual_row;
    let cursor_is_active = app.show_cursor
        && (app.input_mode == InputMode::Normal || app.input_mode == InputMode::Insert);

    let mut lines_vec: Vec<Line> = Vec::with_capacity(vis_height);

    for row_off in 0..vis_height {
        let row_idx = vscroll + row_off;

        if row_idx >= total_vis_rows {
            lines_vec.push(Line::from(Span::raw("")));
            continue;
        }

        let row = &layout.rows[row_idx];

        // Determine logical line index (binary search over line_starts)
        let logical_idx = line_starts.partition_point(|&s| s <= row.start_pos).saturating_sub(1);

        // Is this the first visual row for its logical line?
        let is_first_row_of_logical = row_idx == 0
            || {
                let prev = &layout.rows[row_idx - 1];
                line_starts.partition_point(|&s| s <= prev.start_pos).saturating_sub(1)
                    < logical_idx
            };

        // --- Line number span ---
        let line_num_span: Option<Span> = if gutter_width > 0 {
            let num_str = if is_first_row_of_logical {
                let digits = gutter_width - 1;
                if app.show_relative_line_numbers {
                    let cursor_logical = line_starts
                        .partition_point(|&s| s <= flat_cursor)
                        .saturating_sub(1);
                    if logical_idx == cursor_logical {
                        format!("{:>width$} ", logical_idx + 1, width = digits)
                    } else {
                        let dist = (logical_idx as isize - cursor_logical as isize).unsigned_abs();
                        format!("{:>width$} ", dist, width = digits)
                    }
                } else {
                    format!("{:>width$} ", logical_idx + 1, width = digits)
                }
            } else {
                " ".repeat(gutter_width)
            };
            Some(Span::styled(num_str, Style::default().fg(app.colorscheme.line_number)))
        } else {
            None
        };

        let display_text = row.text.clone();

        // --- Syntax highlighting ---
        let mut content_spans: Vec<Span> = if app.is_markdown_file() {
            highlight_markdown_line(&display_text, &app.colorscheme)
        } else {
            highlight_json_line(&display_text, &app.colorscheme)
        };

        // --- Search highlighting (inline, applied over syntax spans) ---
        if !app.search_query.is_empty() {
            let query_lower = app.search_query.to_lowercase();
            let text_lower = display_text.to_lowercase();
            // Check if there's any match before rebuilding spans
            if text_lower.contains(&query_lower) {
                let actual_vis_row = row_idx;
                content_spans = rebuild_spans_with_search(
                    &display_text,
                    content_spans,
                    &app.search_query,
                    &app.search_matches,
                    app.current_match_index,
                    logical_idx,
                    // column offset of this visual row within the logical line
                    row.start_pos.saturating_sub(*line_starts.get(logical_idx).unwrap_or(&0)),
                    actual_vis_row,
                );
            }
        }

        if cursor_is_active && row_idx == cursor_vis_row {
            content_spans = apply_block_cursor_to_spans(
                content_spans,
                layout.cursor.row_char_offset,
            );
        }

        // Combine spans
        let mut spans: Vec<Span> = Vec::new();
        if let Some(ln) = line_num_span {
            spans.push(ln);
        }
        spans.extend(content_spans);
        lines_vec.push(Line::from(spans));
    }

    // --- Build block and render ---
    let title = match &app.file_path {
        Some(path) => {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let display_name = if !app.show_extension {
                path.file_stem().and_then(|s| s.to_str()).unwrap_or(name).to_string()
            } else {
                name.to_string()
            };
            format!(" {} ", display_name)
        }
        None => String::new(),
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.colorscheme.window_title))
        .borders(Borders::ALL)
        .border_type(app.border_style.to_border_type())
        .border_style(Style::default().fg(app.colorscheme.window_border))
        .style(Style::default().bg(app.colorscheme.background));

    f.render_widget(Paragraph::new(lines_vec).block(block), area);
}

fn apply_block_cursor_to_spans(
    spans: Vec<Span<'static>>,
    cursor_char_pos: usize,
) -> Vec<Span<'static>> {
    let cursor_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(110, 170, 255))
        .add_modifier(Modifier::BOLD);
    let mut result = Vec::new();
    let mut seen_chars = 0usize;

    for span in spans {
        let text = span.content.to_string();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if cursor_char_pos >= seen_chars && cursor_char_pos < seen_chars + len {
            let local = cursor_char_pos - seen_chars;
            let before: String = chars[..local].iter().collect();
            let at_cursor = chars[local].to_string();
            let after: String = chars[local + 1..].iter().collect();

            if !before.is_empty() {
                result.push(Span::styled(before, span.style));
            }
            result.push(Span::styled(at_cursor, cursor_style));
            if !after.is_empty() {
                result.push(Span::styled(after, span.style));
            }
            seen_chars = cursor_char_pos + 1;
        } else {
            result.push(span);
            seen_chars += len;
        }
    }

    if cursor_char_pos >= seen_chars {
        result.push(Span::styled(" ".to_string(), cursor_style));
    }

    result
}

/// Rebuild syntax-highlighted spans for a visual row, adding search match backgrounds.
fn rebuild_spans_with_search(
    display_text: &str,
    syntax_spans: Vec<Span<'static>>,
    query: &str,
    search_matches: &[(usize, usize)],
    current_match_index: Option<usize>,
    logical_line: usize,
    col_offset_in_line: usize,
    _vis_row: usize,
) -> Vec<Span<'static>> {
    let query_lower = query.to_lowercase();
    let text_lower = display_text.to_lowercase();
    let mut result = Vec::new();
    let mut char_pos = 0usize; // position within display_text (chars)

    for span in syntax_spans {
        let span_text = span.content.to_string();
        let span_chars: Vec<char> = span_text.chars().collect();
        let span_len = span_chars.len();
        let mut seg_start = 0usize; // within span_text

        let mut i = 0usize;
        while i < span_len {
            let abs_char = char_pos + i;
            // Find query match starting at or after abs_char
            let text_bytes_before: usize = display_text.chars().take(abs_char).map(|c| c.len_utf8()).sum();
            let remaining = &text_lower[text_bytes_before..];
            if let Some(match_off) = remaining.find(&query_lower) {
                let match_start_char = abs_char + remaining[..match_off].chars().count();
                let match_end_char = match_start_char + query.chars().count();

                // Is this match within this span?
                let span_char_start = char_pos;
                let span_char_end = char_pos + span_len;
                if match_start_char >= span_char_end {
                    break; // match is in a later span
                }

                // Is this the current match?
                let is_current = current_match_index
                    .and_then(|idx| search_matches.get(idx))
                    .map(|(l, c)| *l == logical_line && *c == col_offset_in_line + match_start_char)
                    .unwrap_or(false);
                let bg = if is_current {
                    Color::Rgb(255, 255, 150)
                } else {
                    Color::Rgb(100, 180, 200)
                };

                // Before match
                let before_in_span = match_start_char.saturating_sub(span_char_start);
                if before_in_span > seg_start {
                    let s: String = span_chars[seg_start..before_in_span].iter().collect();
                    result.push(Span::styled(s, span.style));
                }
                // Match portion (clamped to this span)
                let match_start_in_span = before_in_span;
                let match_end_in_span = (match_end_char.saturating_sub(span_char_start)).min(span_len);
                if match_end_in_span > match_start_in_span {
                    let s: String = span_chars[match_start_in_span..match_end_in_span].iter().collect();
                    result.push(Span::styled(s, span.style.bg(bg)));
                }
                seg_start = match_end_in_span;
                i = seg_start;
            } else {
                break;
            }
        }

        // Remaining text in span after last match
        if seg_start < span_len {
            let s: String = span_chars[seg_start..].iter().collect();
            result.push(Span::styled(s, span.style));
        }
        char_pos += span_len;
    }

    result
}

fn render_help_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Create a block with border like View/Edit modes
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(app.border_style.to_border_type())
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
