use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FormatMode, InputMode};
use crate::rendering::{RelfEntry, RelfLineStyle};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Always render content and status bar
    if !app.editing_entry {
        render_content(f, app, chunks[0]);
    } else {
        // Render empty content area with border when overlay is active
        render_empty_content(f, app, chunks[0]);
    }
    render_status_bar(f, app, chunks[1]);

    // Render editing overlay on top if active
    if app.editing_entry {
        render_edit_overlay(f, app);
    }
}

fn render_empty_content(f: &mut Frame, app: &App, area: Rect) {
    // Render background cards with colors but no text when overlay is active
    let title = match &app.file_path {
        Some(path) => format!(" {} ", path.display()),
        None => String::new(),
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Render empty cards with background colors
    let num_entries = app.relf_entries.len();
    if num_entries == 0 {
        return;
    }

    let selected = app.selected_entry_index;
    let max_visible_cards = 5;
    let scroll_start = if selected < max_visible_cards {
        0
    } else {
        selected - max_visible_cards + 1
    };

    let visible_entries: Vec<(usize, &RelfEntry)> = app.relf_entries
        .iter()
        .enumerate()
        .skip(scroll_start)
        .take(max_visible_cards)
        .collect();

    if visible_entries.is_empty() {
        return;
    }

    let constraints: Vec<Constraint> = visible_entries
        .iter()
        .map(|_| Constraint::Min(3))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // Render only background colors (no borders, no text)
    for (i, (_entry_idx, entry)) in visible_entries.iter().enumerate() {
        // Fill the entire card area with just background color
        let filler = Block::default()
            .style(Style::default().bg(entry.bg_color));
        f.render_widget(filler, chunks[i]);
    }
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    // In Relf mode with entries, render as cards
    if app.format_mode == FormatMode::Relf && !app.relf_entries.is_empty() {
        render_relf_cards(f, app, area);
        return;
    }

    let inner_area = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    // Update the app's notion of the current content width for accurate wrapping
    // Use inner area width (inside borders and margins)
    app.content_width = inner_area.width;
    // In Relf mode, disable horizontal scrolling entirely
    if app.format_mode == FormatMode::Relf {
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
        let off_cols = if app.format_mode == FormatMode::Relf {
            0
        } else {
            app.hscroll as usize
        };
        let mut lines_vec: Vec<Line> = Vec::new();

        for (line_idx, s) in visible_content.iter().enumerate() {
            let actual_idx = line_idx + app.scroll as usize;
            let slice = app.slice_columns(s, off_cols, w_cols);

            // Build spans for the line with search highlighting
            let mut spans: Vec<Span> = Vec::new();
            let line_style = if app.format_mode == FormatMode::Relf {
                app.relf_visual_styles.get(actual_idx)
            } else {
                None
            };

            if !app.search_query.is_empty() {
                // Highlight search matches
                let query_lower = app.search_query.to_lowercase();
                let line_lower = slice.to_lowercase();
                let mut last_pos = 0;

                while let Some(match_pos) = line_lower[last_pos..].find(&query_lower) {
                    let actual_pos = last_pos + match_pos;

                    // Add text before match
                    if actual_pos > last_pos {
                        spans.push(Span::styled(
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

                    spans.push(Span::styled(
                        slice[actual_pos..match_end.min(slice.len())].to_string(),
                        highlight_style,
                    ));

                    last_pos = match_end;
                }

                // Add remaining text after last match
                if last_pos < slice.len() {
                    spans.push(Span::styled(
                        slice[last_pos..].to_string(),
                        apply_relf_style(Style::default().fg(Color::Gray), line_style),
                    ));
                }
            } else {
                // No search highlighting, just use plain text
                spans.push(Span::styled(
                    slice.clone(),
                    apply_relf_style(Style::default().fg(Color::Gray), line_style),
                ));
            }

            // Add cursor if needed
            if app.format_mode == FormatMode::Json
                && (app.input_mode == InputMode::Insert || app.input_mode == InputMode::Normal)
                && app.show_cursor
            {
                if actual_idx == app.content_cursor_line {
                    let cursor_char_pos = app.content_cursor_col;
                    let prefix_cols = app.prefix_display_width(s, cursor_char_pos);
                    if prefix_cols >= off_cols && prefix_cols < off_cols + w_cols {
                        // Always show cursor as vertical bar
                        let line_chars: Vec<char> = slice.chars().collect();
                        let insert_col_in_view = prefix_cols - off_cols;
                        let insert_idx = app.char_index_for_col(&slice, insert_col_in_view);

                        let mut new_chars = line_chars.clone();
                        let pos = insert_idx.min(new_chars.len());
                        new_chars.insert(pos, '│');
                        spans = vec![Span::styled(
                            new_chars.into_iter().collect::<String>(),
                            apply_relf_style(Style::default().fg(Color::Gray), line_style),
                        )];
                    }
                }
            }

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
        Some(path) => format!(" {} ", path.display()),
        None => String::new(),
    };

    let content = Paragraph::new(content_text).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(content, area);
}

fn render_relf_cards(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match &app.file_path {
        Some(path) => format!(" {} ", path.display()),
        None => String::new(),
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    app.content_width = inner_area.width;
    app.visible_height = inner_area.height;
    app.hscroll = 0;

    let num_entries = app.relf_entries.len();
    if num_entries == 0 {
        return;
    }

    // Use selected_entry_index to determine which entries to show
    let selected = app.selected_entry_index;

    // Limit number of visible cards
    let max_visible_cards = 5;

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

        let mut lines = Vec::new();

        // First line is bold title (with search highlight)
        if let Some(first) = entry.lines.first() {
            if !app.search_query.is_empty() {
                lines.push(highlight_search_in_line(
                    first,
                    &app.search_query,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                lines.push(Line::styled(
                    first.as_str(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        // Remaining lines (with search highlight)
        for (idx, line) in entry.lines.iter().enumerate().skip(1) {
            let fg = if idx == entry.lines.len() - 1 {
                Color::Rgb(160, 200, 120)
            } else if line.starts_with("http") {
                Color::Rgb(120, 170, 255)
            } else {
                Color::Gray
            };

            if !app.search_query.is_empty() {
                lines.push(highlight_search_in_line(
                    line,
                    &app.search_query,
                    Style::default().fg(fg),
                ));
            } else {
                lines.push(Line::styled(line.as_str(), Style::default().fg(fg)));
            }
        }

        // Highlight selected card with different border color
        let border_style = if is_selected {
            Style::default().fg(Color::Yellow).bg(entry.bg_color)
        } else {
            Style::default().bg(entry.bg_color)
        };

        let card = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(border_style),
            );

        f.render_widget(card, chunks[i]);
    }
}

fn highlight_search_in_line<'a>(line: &'a str, query: &str, base_style: Style) -> Line<'a> {
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
    let popup_height = (app.edit_buffer.len() as u16 + 4).min(area.height - 4);

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Render the popup as a single card with rounded borders
    let block = Block::default()
        .title(" Edit Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(30, 30, 35)).fg(Color::White));

    f.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);

    // Render each field as simple lines with color-based selection
    let mut lines = Vec::new();
    for (i, field) in app.edit_buffer.iter().enumerate() {
        let is_selected = i == app.edit_field_index;

        let style = if is_selected {
            if app.edit_insert_mode {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::Gray)
        };

        // Add cursor in insert mode
        let display_text = if is_selected && app.edit_insert_mode {
            let cursor_pos = app.edit_cursor_pos.min(field.len());
            let mut text = field.clone();
            text.insert(cursor_pos, '|');
            text
        } else {
            field.clone()
        };

        lines.push(Line::styled(display_text, style));
    }

    let content = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(content, inner_area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    if !app.status_message.is_empty() {
        let status_text = format!(" {} ", app.status_message);

        let status_widget = Paragraph::new(Line::from(vec![Span::styled(
            status_text,
            Style::default().fg(Color::Cyan),
        )]))
        .alignment(Alignment::Left);

        f.render_widget(status_widget, area);
    } else {
        // Empty status bar when no message
        let empty_widget = Paragraph::new("");
        f.render_widget(empty_widget, area);
    }
}
