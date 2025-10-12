use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FormatMode, InputMode};
use crate::rendering::{RelfEntry, RelfLineStyle};

// JSON syntax highlighting
fn highlight_json_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Push accumulated text
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Gray),
                    ));
                    current.clear();
                }

                // Start collecting string
                let mut string_content = String::from("\"");
                let mut escaped = false;

                while let Some(next_ch) = chars.next() {
                    string_content.push(next_ch);
                    if next_ch == '\\' && !escaped {
                        escaped = true;
                    } else if next_ch == '"' && !escaped {
                        break;
                    } else {
                        escaped = false;
                    }
                }

                // Determine if this is a key (followed by ':')
                let mut temp_chars = chars.clone();
                let mut is_key = false;
                while let Some(peek_ch) = temp_chars.next() {
                    if peek_ch == ':' {
                        is_key = true;
                        break;
                    } else if !peek_ch.is_whitespace() {
                        break;
                    }
                }

                let color = if is_key {
                    Color::Rgb(156, 220, 254) // Keys in light blue (VS Code style)
                } else {
                    Color::Rgb(206, 145, 120) // String values in orange/peach (VS Code style)
                };

                spans.push(Span::styled(
                    string_content,
                    Style::default().fg(color),
                ));
            }
            '{' | '}' | '[' | ']' => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Gray),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Rgb(255, 217, 102)), // Yellow/gold (VS Code style)
                ));
            }
            ':' | ',' => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Gray),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
            't' | 'f' | 'n' => {
                // Check for true, false, null
                let peek_str: String = std::iter::once(ch)
                    .chain(chars.clone().take(4))
                    .collect();

                if peek_str.starts_with("true") || peek_str.starts_with("false") || peek_str.starts_with("null") {
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(Color::Gray),
                        ));
                        current.clear();
                    }

                    let keyword = if peek_str.starts_with("true") {
                        chars.nth(2); // skip 'r', 'u', 'e'
                        "true"
                    } else if peek_str.starts_with("false") {
                        chars.nth(3); // skip 'a', 'l', 's', 'e'
                        "false"
                    } else {
                        chars.nth(2); // skip 'u', 'l', 'l'
                        "null"
                    };

                    spans.push(Span::styled(
                        keyword.to_string(),
                        Style::default().fg(Color::Rgb(86, 156, 214)), // Purple/blue (VS Code style)
                    ));
                } else {
                    current.push(ch);
                }
            }
            '0'..='9' | '-' => {
                // Numbers
                let mut num = String::from(ch);
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' || next_ch == 'e' || next_ch == 'E' || next_ch == '-' || next_ch == '+' {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Gray),
                    ));
                    current.clear();
                }

                spans.push(Span::styled(
                    num,
                    Style::default().fg(Color::Rgb(181, 206, 168)), // Light green (VS Code style)
                ));
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(
            current,
            Style::default().fg(Color::Gray),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            String::new(),
            Style::default().fg(Color::Gray),
        ));
    }

    spans
}

// Slice spans by display width (for horizontal scrolling)
fn slice_spans_by_width(app: &App, spans: Vec<Span>, start_col: usize, width: usize) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let mut current_col = 0;
    let end_col = start_col + width;

    for span in spans {
        let text = span.content.to_string();
        let span_width = app.display_width_str(&text);
        let span_start = current_col;
        let span_end = current_col + span_width;

        if span_end <= start_col {
            // This span is entirely before the visible range
            current_col = span_end;
            continue;
        }

        if span_start >= end_col {
            // This span is entirely after the visible range
            break;
        }

        // This span overlaps with visible range - need to slice it
        let visible_start = if span_start < start_col {
            start_col - span_start
        } else {
            0
        };

        let visible_end = if span_end > end_col {
            span_width - (span_end - end_col)
        } else {
            span_width
        };

        // Slice the text by display width
        let sliced_text = app.slice_columns(&text, visible_start, visible_end - visible_start);

        if !sliced_text.is_empty() {
            result.push(Span::styled(sliced_text, span.style));
        }

        current_col = span_end;
    }

    result
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Split horizontally if explorer is open
    let content_area = if app.explorer_open {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(chunks[0]);

        // Render explorer in left panel
        render_explorer(f, app, horizontal_chunks[0]);

        // Return right panel for main content
        horizontal_chunks[1]
    } else {
        chunks[0]
    };

    // Always render content and status bar (even when overlay is active)
    render_content(f, app, content_area);
    render_status_bar(f, app, chunks[1]);

    // Render editing overlay on top if active
    if app.editing_entry {
        render_edit_overlay(f, app);
    }
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    // In View mode with entries, render as cards
    if app.format_mode == FormatMode::View && !app.relf_entries.is_empty() {
        render_relf_cards(f, app, area);
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
                    Style::default().fg(Color::DarkGray),
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
                let json_spans = highlight_json_line(s);

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
                            apply_relf_style(Style::default().fg(Color::Gray), line_style),
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
                        apply_relf_style(Style::default().fg(Color::Gray), line_style),
                    ));
                }
            } else {
                // No search highlighting
                if app.format_mode == FormatMode::Edit {
                    // Apply JSON syntax highlighting to full line, then slice
                    let full_line_spans = highlight_json_line(s);
                    content_spans = slice_spans_by_width(app, full_line_spans, off_cols, adjusted_w_cols);
                } else {
                    // In View mode, use plain text with line style
                    content_spans.push(Span::styled(
                        slice.clone(),
                        apply_relf_style(Style::default().fg(Color::Gray), line_style),
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
                                    let ch_width = app.display_width_str(&ch.to_string());
                                    if accumulated_width >= target_width_in_span {
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
                format!(" {} ", name)
            } else {
                String::new()
            }
        }
        None => String::new(),
    };

    let content = Paragraph::new(content_text).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(26, 28, 34))),
    );

    f.render_widget(content, area);
}

fn render_help_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Create a block with border like View/Edit modes
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(26, 28, 34)));

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
        .style(Style::default().fg(Color::White).bg(Color::Rgb(26, 28, 34)));

    f.render_widget(content, area);
}

fn render_relf_cards(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match &app.file_path {
        Some(path) => {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                format!(" {} ", name)
            } else {
                String::new()
            }
        }
        None => String::new(),
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(26, 28, 34)));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    app.content_width = inner_area.width;
    app.visible_height = inner_area.height;
    // Don't reset hscroll here - allow scrolling within cards

    let num_entries = app.relf_entries.len();
    if num_entries == 0 {
        return;
    }

    // Use selected_entry_index to determine which entries to show
    let selected = app.selected_entry_index;

    // Limit number of visible cards (use app setting)
    let max_visible_cards = app.max_visible_cards;

    // Calculate scroll window to keep selected entry visible
    let scroll_start = if selected < max_visible_cards {
        0
    } else {
        selected - max_visible_cards + 1
    };

    // Get visible entries
    let visible_entries: Vec<(usize, &RelfEntry)> = app.relf_entries
        .iter()
        .enumerate()
        .skip(scroll_start)
        .take(max_visible_cards)
        .collect();

    if visible_entries.is_empty() {
        return;
    }

    // Create constraints with Min for flexible heights
    let constraints: Vec<Constraint> = visible_entries
        .iter()
        .map(|_| Constraint::Min(3)) // Minimum 3 lines per card
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // Render each card with Block border
    for (i, (entry_idx, entry)) in visible_entries.iter().enumerate() {
        let is_selected = *entry_idx == selected;

        // Check if this card is in Visual mode selection range
        let in_visual_range = if app.visual_mode {
            let visual_start = app.visual_start_index.min(app.visual_end_index);
            let visual_end = app.visual_start_index.max(app.visual_end_index);
            *entry_idx >= visual_start && *entry_idx <= visual_end
        } else {
            false
        };

        // Highlight selected card with different border color
        let border_style = if in_visual_range {
            // Visual mode selection: cyan border
            Style::default().fg(Color::Cyan).bg(entry.bg_color)
        } else if is_selected {
            // Current cursor: yellow border
            Style::default().fg(Color::Yellow).bg(entry.bg_color)
        } else {
            Style::default().bg(entry.bg_color)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(border_style);

        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);

        // Check if this is an outside entry (has name field)
        if entry.name.is_some() {
            // Outside entry: corner layout
            render_outside_card(f, app, entry, chunks[i], inner, is_selected);
        } else {
            // Inside entry: simple layout
            render_inside_card(f, app, entry, chunks[i], inner, is_selected);
        }
    }
}

fn render_outside_card(f: &mut Frame, app: &App, entry: &RelfEntry, card_area: Rect, inner_area: Rect, is_selected: bool) {
    // Render labels on the border (outside the inner area)
    let name = entry.name.as_deref().unwrap_or("");
    let url = entry.url.as_deref().unwrap_or("");

    // Top-left: name (on the border) - only if not empty
    if !name.is_empty() {
        let name_text = format!(" {} ", name);
        let name_span = if !app.search_query.is_empty() {
            highlight_search_in_line(
                &name_text,
                &app.search_query,
                Style::default().fg(Color::Rgb(156, 220, 254)),
            )
        } else {
            Line::styled(name_text, Style::default().fg(Color::Rgb(156, 220, 254)))
        };
        let name_area = Rect { x: card_area.x + 2, y: card_area.y, width: card_area.width.saturating_sub(4), height: 1 };
        let name_para = Paragraph::new(name_span).alignment(Alignment::Left);
        f.render_widget(name_para, name_area);
    }

    // Top-right: url (on the border)
    if !url.is_empty() {
        let url_text = format!(" {} ", url);
        let url_span = if !app.search_query.is_empty() {
            highlight_search_in_line(
                &url_text,
                &app.search_query,
                Style::default().fg(Color::Rgb(156, 220, 254)),
            )
        } else {
            Line::styled(url_text, Style::default().fg(Color::Rgb(156, 220, 254)))
        };
        let url_area = Rect { x: card_area.x + 2, y: card_area.y, width: card_area.width.saturating_sub(4), height: 1 };
        let url_para = Paragraph::new(url_span).alignment(Alignment::Right);
        f.render_widget(url_para, url_area);
    }

    // Bottom-right: percentage (on the border) - only if not null
    if let Some(percentage) = entry.percentage {
        let percentage_text = format!(" {}% ", percentage);
        let percentage_span = Line::styled(
            percentage_text,
            Style::default().fg(Color::Rgb(156, 220, 254)),
        );
        let percentage_area = Rect {
            x: card_area.x + 2,
            y: card_area.y + card_area.height.saturating_sub(1),
            width: card_area.width.saturating_sub(4),
            height: 1
        };
        let percentage_para = Paragraph::new(percentage_span).alignment(Alignment::Right);
        f.render_widget(percentage_para, percentage_area);
    }

    // Middle: context (inside the card)
    let context = entry.context.as_deref().unwrap_or("");
    if !context.is_empty() {
        // Split context by \n for rendering - handle both literal \n and actual newlines
        let context_with_newlines = context.replace("\\n", "\n");
        let vscroll = if is_selected { app.hscroll as usize } else { 0 };
        // Use full height of inner area
        let visible_lines = inner_area.height as usize;

        let context_lines: Vec<Line> = context_with_newlines
            .lines()
            .skip(vscroll)
            .take(visible_lines)
            .map(|line| {
                if !app.search_query.is_empty() {
                    highlight_search_in_line(
                        line,
                        &app.search_query,
                        Style::default().fg(Color::Gray),
                    )
                } else {
                    Line::styled(line.to_string(), Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let context_para = Paragraph::new(context_lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);
        f.render_widget(context_para, inner_area);
    }
}

fn render_inside_card(f: &mut Frame, app: &App, entry: &RelfEntry, card_area: Rect, inner_area: Rect, is_selected: bool) {
    // Date on the border (top-left)
    if let Some(date) = &entry.date {
        let date_text = format!(" {} ", date);
        let date_span = if !app.search_query.is_empty() {
            highlight_search_in_line(
                &date_text,
                &app.search_query,
                Style::default().fg(Color::Rgb(156, 220, 254)),
            )
        } else {
            Line::styled(
                date_text,
                Style::default().fg(Color::Rgb(156, 220, 254)),
            )
        };
        let date_area = Rect { x: card_area.x + 2, y: card_area.y, width: card_area.width.saturating_sub(4), height: 1 };
        let date_para = Paragraph::new(date_span).alignment(Alignment::Left);
        f.render_widget(date_para, date_area);
    }

    // Context inside the card
    if let Some(context) = &entry.context {
        // Split context by \n for rendering - handle both literal \n and actual newlines
        let context_with_newlines = context.replace("\\n", "\n");
        let vscroll = if is_selected { app.hscroll as usize } else { 0 };
        // Use full height of inner area
        let visible_lines = inner_area.height as usize;

        let context_lines: Vec<Line> = context_with_newlines
            .lines()
            .skip(vscroll)
            .take(visible_lines)
            .map(|line| {
                if !app.search_query.is_empty() {
                    highlight_search_in_line(
                        line,
                        &app.search_query,
                        Style::default().fg(Color::Gray),
                    )
                } else {
                    Line::styled(line.to_string(), Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let context_para = Paragraph::new(context_lines).wrap(Wrap { trim: false });
        f.render_widget(context_para, inner_area);
    }
}

fn highlight_search_in_line(line: &str, query: &str, base_style: Style) -> Line<'static> {
    let query_lower = query.to_lowercase();
    let line_lower = line.to_lowercase();
    let mut spans = Vec::new();
    let mut byte_pos = 0;

    while byte_pos < line_lower.len() {
        if let Some(match_pos) = line_lower[byte_pos..].find(&query_lower) {
            let actual_byte_pos = byte_pos + match_pos;

            // Add text before match (ensuring char boundaries)
            if actual_byte_pos > byte_pos && line.is_char_boundary(byte_pos) && line.is_char_boundary(actual_byte_pos) {
                spans.push(Span::styled(
                    line[byte_pos..actual_byte_pos].to_string(),
                    base_style,
                ));
            }

            // Add highlighted match (ensuring char boundaries)
            let match_end_byte = actual_byte_pos + query_lower.len();
            if line.is_char_boundary(actual_byte_pos) && match_end_byte <= line.len() {
                let safe_end = if line.is_char_boundary(match_end_byte) {
                    match_end_byte
                } else {
                    // Find next char boundary
                    (match_end_byte..=line.len())
                        .find(|&i| line.is_char_boundary(i))
                        .unwrap_or(line.len())
                };

                spans.push(Span::styled(
                    line[actual_byte_pos..safe_end].to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ));
                byte_pos = safe_end;
            } else {
                byte_pos = match_end_byte;
            }

            // Ensure we're on a char boundary
            while byte_pos < line.len() && !line.is_char_boundary(byte_pos) {
                byte_pos += 1;
            }
        } else {
            break;
        }
    }

    // Add remaining text after last match
    if byte_pos < line.len() && line.is_char_boundary(byte_pos) {
        spans.push(Span::styled(line[byte_pos..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(line.to_string(), base_style));
    }

    Line::from(spans)
}

fn apply_relf_style(mut style: Style, line_style: Option<&RelfLineStyle>) -> Style {
    if let Some(ls) = line_style {
        if let Some(fg) = ls.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = ls.bg {
            style = style.bg(bg);
        }
        if ls.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
    }
    style
}

fn render_edit_overlay(f: &mut Frame, app: &App) {
    // Create a centered popup area
    let area = f.area();

    let popup_width = area.width.min(80);
    // Increase height to show more of the background: use 70% of screen height or calculated size
    let calculated_height = app.edit_buffer.len() as u16 + 4;
    let max_height = (area.height * 7) / 10; // 70% of screen height
    let popup_height = calculated_height.max(max_height.min(area.height - 4));

    // Align x to even column to prevent wide-char (CJK) rendering issues with borders
    let x_centered = (area.width.saturating_sub(popup_width)) / 2;
    let x_aligned = x_centered & !1; // Force to even number

    let popup_area = Rect {
        x: x_aligned,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Create a slightly wider clear area to avoid cutting wide characters at boundaries
    let clear_area = Rect {
        x: x_aligned.saturating_sub(1),
        y: popup_area.y,
        width: popup_width.saturating_add(2).min(area.width.saturating_sub(x_aligned.saturating_sub(1))),
        height: popup_height,
    };

    // Clear the wider area to fully erase any wide characters
    f.render_widget(Clear, clear_area);

    // Fill the clear area with background color using spaces
    // This ensures complete coverage, especially for wide characters
    let blank_lines: Vec<Line> = (0..clear_area.height)
        .map(|_| Line::from(" ".repeat(clear_area.width as usize)))
        .collect();
    let blank_paragraph = Paragraph::new(blank_lines)
        .style(Style::default().bg(Color::Rgb(30, 30, 35)));
    f.render_widget(blank_paragraph, clear_area);

    // Determine if editing INSIDE or OUTSIDE entry
    // INSIDE: date, context, Exit (3 fields)
    // OUTSIDE: name, context, url, percentage, Exit (5 fields)
    let title = if app.edit_buffer.len() == 3 {
        " INSIDE "
    } else {
        " OUTSIDE "
    };

    // Render the popup as a single card with rounded borders on top
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(30, 30, 35)).fg(Color::White));

    f.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);

    // Render each field with proper windowing and scrolling
    let mut lines = Vec::new();
    let window_width = inner_area.width as usize;

    for (i, field) in app.edit_buffer.iter().enumerate() {
        let is_selected = i == app.edit_field_index;

        // Check if this is a placeholder using the placeholder flag
        let is_placeholder = i < app.edit_buffer_is_placeholder.len()
                           && app.edit_buffer_is_placeholder[i];

        let style = if is_selected {
            // View Edit mode or Insert mode: Yellow (both are editing modes)
            // Normal mode: Cyan
            if app.edit_insert_mode || app.view_edit_mode {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            }
        } else if is_placeholder {
            // Show placeholders in dim gray
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Check if this is context field (index 1 in both INSIDE and OUTSIDE)
        let is_context_field = (app.edit_buffer.len() == 3 && i == 1) || // INSIDE context
                               (app.edit_buffer.len() == 5 && i == 1);   // OUTSIDE context

        // Render newlines for context field:
        // - Field selection mode (not editing): render \n as newlines (multi-line)
        // - View Edit mode: render \n as newlines (multi-line)
        // - Normal/Insert mode: show raw \n (single-line with wrapping)
        // - Other fields: never render \n as newlines
        let should_render_newlines = is_context_field && (!app.edit_field_editing_mode || app.view_edit_mode);

        if is_context_field && should_render_newlines {
            // Context field with newlines: dynamic window with scrolling
            let text_with_newlines = field.replace("\\n", "\n");
            let field_lines: Vec<&str> = text_with_newlines.lines().collect();

            // Dynamic window size for context field
            // Calculate available space: inner_area height minus other fields
            // Each non-context field takes 1 line + 1 blank line = 2 lines each
            let num_other_fields = app.edit_buffer.len() - 1; // All fields except context
            let other_fields_height = num_other_fields * 2; // Each field + blank line
            let available_height = inner_area.height as usize;
            let min_window_height = 5;
            let max_window_height = if available_height > other_fields_height {
                (available_height - other_fields_height).max(min_window_height)
            } else {
                min_window_height
            };

            // Determine window height based on mode
            let actual_lines = field_lines.len();
            let window_height = if !app.edit_field_editing_mode {
                // Field selection mode: use max window height (like View mode)
                max_window_height
            } else if actual_lines < min_window_height {
                // View Edit mode: use actual lines if fewer than minimum
                actual_lines.max(1) // At least 1 line for empty content
            } else {
                // View Edit mode: use smaller of actual lines or max window height
                actual_lines.min(max_window_height)
            };

            let vscroll = app.edit_vscroll as usize;

            // Apply vertical scroll
            let visible_lines: Vec<&str> = field_lines
                .iter()
                .skip(vscroll)
                .take(window_height)
                .copied()
                .collect();

            // Calculate cursor position if editing
            let (cursor_line, cursor_col) = if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
                // Calculate which line and column the cursor is on
                let mut char_count = 0;
                let mut cursor_line_idx = 0;
                let mut cursor_col_in_line = 0;

                for (line_idx, line) in field_lines.iter().enumerate() {
                    let line_len = line.chars().count();
                    let separator_len = if line_idx < field_lines.len() - 1 { 2 } else { 0 }; // "\\n" = 2 chars

                    if app.edit_cursor_pos <= char_count + line_len {
                        cursor_line_idx = line_idx;
                        cursor_col_in_line = app.edit_cursor_pos - char_count;
                        break;
                    }

                    char_count += line_len + separator_len;
                }

                (cursor_line_idx, cursor_col_in_line)
            } else {
                (0, 0)
            };

            // Render each visible line (no horizontal scrolling for context field)
            for (visible_idx, line_text) in visible_lines.iter().enumerate() {
                let actual_line_idx = vscroll + visible_idx;
                let mut display_line = line_text.to_string();

                // Add cursor if this is the line with the cursor
                if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) && actual_line_idx == cursor_line {
                    let char_count = display_line.chars().count();
                    let cursor_char_pos = cursor_col.min(char_count);
                    let byte_pos = if cursor_char_pos == 0 {
                        0
                    } else if cursor_char_pos >= char_count {
                        display_line.len()
                    } else {
                        display_line.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(display_line.len())
                    };
                    display_line.insert(byte_pos, '|');
                }

                // Context field doesn't use horizontal scrolling, just display as-is
                // Text will wrap naturally in the Paragraph widget
                lines.push(Line::styled(display_line, style));
            }
        } else if is_context_field {
            // Context field in Normal/Insert mode: show raw \n with wrapping
            let mut display_text = field.clone();

            // Add cursor in insert mode or field editing mode
            if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
                let char_count = field.chars().count();
                let cursor_char_pos = app.edit_cursor_pos.min(char_count);
                let byte_pos = if cursor_char_pos == 0 {
                    0
                } else if cursor_char_pos >= char_count {
                    field.len()
                } else {
                    field.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(field.len())
                };
                display_text.insert(byte_pos, '|');
            }

            // Calculate available height dynamically (similar to View Edit mode)
            let num_other_fields = app.edit_buffer.len() - 1; // All fields except context
            let other_fields_height = num_other_fields * 2; // Each field + blank line
            let available_height = inner_area.height as usize;
            let min_window_height = 5;
            let max_wrapped_lines = if available_height > other_fields_height {
                (available_height - other_fields_height).max(min_window_height)
            } else {
                min_window_height
            };

            // Split text into chunks that fit within window width (wrapping)
            let chars: Vec<char> = display_text.chars().collect();
            let mut line_start = 0;
            let mut wrapped_line_count = 0;

            while line_start < chars.len() && wrapped_line_count < max_wrapped_lines {
                let mut line_width = 0;
                let mut line_end = line_start;

                for (idx, ch) in chars[line_start..].iter().enumerate() {
                    let char_width = app.display_width_str(&ch.to_string());
                    if line_width + char_width > window_width && line_width > 0 {
                        break;
                    }
                    line_width += char_width;
                    line_end = line_start + idx + 1;
                }

                if line_end == line_start {
                    // Edge case: single character wider than window
                    line_end = line_start + 1;
                }

                let line_text: String = chars[line_start..line_end].iter().collect();
                lines.push(Line::styled(line_text, style));

                line_start = line_end;
                wrapped_line_count += 1;
            }

            // If empty, add at least one line
            if chars.is_empty() {
                lines.push(Line::styled(String::new(), style));
            }
        } else {
            // Single-line field: apply horizontal scrolling
            let mut display_text = field.clone();

            // Add cursor in insert mode or field editing mode
            if is_selected && (app.edit_insert_mode || app.edit_field_editing_mode) {
                let char_count = field.chars().count();
                let cursor_char_pos = app.edit_cursor_pos.min(char_count);
                let byte_pos = if cursor_char_pos == 0 {
                    0
                } else if cursor_char_pos >= char_count {
                    field.len()
                } else {
                    field.char_indices().nth(cursor_char_pos).map(|(i, _)| i).unwrap_or(field.len())
                };
                display_text.insert(byte_pos, '|');
            }

            // Apply horizontal scroll if this is the selected field and in editing mode
            let scrolled_text = if is_selected && app.edit_field_editing_mode {
                let hscroll = app.edit_hscroll as usize;
                // Use display width slicing
                app.slice_columns(&display_text, hscroll, window_width)
            } else {
                // No scrolling for non-editing fields, just truncate if too long
                if app.display_width_str(&display_text) > window_width {
                    app.slice_columns(&display_text, 0, window_width)
                } else {
                    display_text
                }
            };

            lines.push(Line::styled(scrolled_text, style));
        }

        // Add blank line between fields
        if i < app.edit_buffer.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(content, inner_area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    // Left side: status message
    if !app.status_message.is_empty() {
        let status_text = format!(" {} ", app.status_message);
        spans.push(Span::styled(
            status_text,
            Style::default().fg(Color::Cyan),
        ));
    }

    // Right side: cursor position in Edit mode
    if app.format_mode == FormatMode::Edit {
        let current_line = app.content_cursor_line + 1;
        let current_col = app.content_cursor_col + 1;
        let position_text = format!("{}:{} ", current_line, current_col);

        // Calculate padding to right-align
        let status_width = if !app.status_message.is_empty() {
            app.status_message.len() + 2
        } else {
            0
        };
        let position_width = position_text.len();
        let available_width = area.width as usize;

        if available_width > status_width + position_width {
            let padding_width = available_width - status_width - position_width;
            spans.push(Span::raw(" ".repeat(padding_width)));
        }

        spans.push(Span::styled(
            position_text,
            Style::default().fg(Color::DarkGray),
        ));
    }

    let status_widget = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left);

    f.render_widget(status_widget, area);
}

fn render_explorer(f: &mut Frame, app: &App, area: Rect) {
    // Show only folder name, not full path
    let title = if let Some(folder_name) = app.explorer_current_dir.file_name().and_then(|n| n.to_str()) {
        format!(" {} ", folder_name)
    } else {
        " . ".to_string()
    };

    // Use same gray color as file window
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(26, 28, 34)));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Calculate visible range
    let visible_height = inner_area.height as usize;
    let scroll_pos = app.explorer_scroll as usize;
    let total_entries = app.explorer_entries.len();

    let start = scroll_pos.min(total_entries.saturating_sub(1));
    let end = (start + visible_height).min(total_entries);

    // Render entries
    let mut lines = Vec::new();
    for (i, entry) in app.explorer_entries[start..end].iter().enumerate() {
        let abs_index = start + i;
        let is_selected = abs_index == app.explorer_selected_index;

        // Build indentation based on depth
        let indent = "  ".repeat(entry.depth);

        // Get file/directory name
        let name = entry.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???")
            .to_string();

        // Add expand/collapse indicator for directories
        let indicator = if entry.path.is_dir() {
            if entry.is_expanded {
                "▾ " // Expanded
            } else {
                "▸ " // Collapsed
            }
        } else {
            "  " // File (no indicator)
        };

        // Combine indent, indicator, and name
        let display_text = format!("{}{}{}", indent, indicator, name);

        // Show directories in cyan, files in gray
        let color = if entry.path.is_dir() {
            Color::Cyan
        } else {
            Color::Gray
        };

        let style = if is_selected {
            Style::default().fg(color).bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        lines.push(Line::styled(display_text, style));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(content, inner_area);
}
