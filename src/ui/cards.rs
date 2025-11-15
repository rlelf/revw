use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::rendering::RelfEntry;

use super::utils::highlight_search_in_line;

pub fn render_relf_cards(f: &mut Frame, app: &mut App, area: Rect) {
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

    let outer_block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.colorscheme.window_title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.colorscheme.window_border))
        .style(Style::default().bg(app.colorscheme.background));

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
            // Visual mode selection border
            Style::default().fg(app.colorscheme.card_visual).bg(app.colorscheme.background)
        } else if is_selected {
            // Selected card border
            Style::default().fg(app.colorscheme.card_selected).bg(app.colorscheme.background)
        } else {
            // Normal card border
            Style::default().fg(app.colorscheme.card_border).bg(app.colorscheme.background)
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

    // Top-left: name (on the border)
    if !name.is_empty() {
        let name_text = format!(" {} ", name);
        let name_span = if !app.search_query.is_empty() {
            highlight_search_in_line(
                &name_text,
                &app.search_query,
                Style::default().fg(app.colorscheme.card_title),
            )
        } else {
            Line::styled(name_text, Style::default().fg(app.colorscheme.card_title))
        };
        let name_area = Rect { x: card_area.x + 2, y: card_area.y, width: card_area.width.saturating_sub(4), height: 1 };
        let name_para = Paragraph::new(name_span).alignment(Alignment::Left);
        f.render_widget(name_para, name_area);
    }

    // Bottom-left: url (on the border) - render first
    if !url.is_empty() {
        let url_text = format!(" {} ", url);
        let url_span = if !app.search_query.is_empty() {
            highlight_search_in_line(
                &url_text,
                &app.search_query,
                Style::default().fg(app.colorscheme.card_title),
            )
        } else {
            Line::styled(url_text, Style::default().fg(app.colorscheme.card_title))
        };
        let url_area = Rect {
            x: card_area.x + 2,
            y: card_area.y + card_area.height.saturating_sub(1),
            width: card_area.width.saturating_sub(4),
            height: 1
        };
        let url_para = Paragraph::new(url_span).alignment(Alignment::Left);
        f.render_widget(url_para, url_area);
    }

    // Bottom-right: percentage (on the border) - render after url to ensure visibility
    if let Some(percentage) = entry.percentage {
        let percentage_text = format!(" {}% ", percentage);
        let percentage_span = Line::styled(
            percentage_text,
            Style::default().fg(app.colorscheme.card_title),
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
        // Context already contains actual newline characters
        let visible_lines = inner_area.height as usize;
        let total_lines = context.lines().count();
        // Allow scrolling to see all content (like overlay mode)
        let max_scroll = total_lines;
        let vscroll = if is_selected {
            (app.hscroll as usize).min(max_scroll)
        } else {
            0
        };

        let context_lines: Vec<Line> = context
            .lines()
            .skip(vscroll)
            .take(visible_lines)
            .map(|line| {
                if !app.search_query.is_empty() {
                    highlight_search_in_line(
                        line,
                        &app.search_query,
                        Style::default().fg(app.colorscheme.card_content),
                    )
                } else {
                    Line::styled(line.to_string(), Style::default().fg(app.colorscheme.card_content))
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
                Style::default().fg(app.colorscheme.card_title),
            )
        } else {
            Line::styled(
                date_text,
                Style::default().fg(app.colorscheme.card_title),
            )
        };
        let date_area = Rect { x: card_area.x + 2, y: card_area.y, width: card_area.width.saturating_sub(4), height: 1 };
        let date_para = Paragraph::new(date_span).alignment(Alignment::Left);
        f.render_widget(date_para, date_area);
    }

    // Context inside the card
    if let Some(context) = &entry.context {
        // Context already contains actual newline characters
        let visible_lines = inner_area.height as usize;
        let total_lines = context.lines().count();
        // Allow scrolling to see all content (like overlay mode)
        let max_scroll = total_lines;
        let vscroll = if is_selected {
            (app.hscroll as usize).min(max_scroll)
        } else {
            0
        };

        let context_lines: Vec<Line> = context
            .lines()
            .skip(vscroll)
            .take(visible_lines)
            .map(|line| {
                if !app.search_query.is_empty() {
                    highlight_search_in_line(
                        line,
                        &app.search_query,
                        Style::default().fg(app.colorscheme.card_content),
                    )
                } else {
                    Line::styled(line.to_string(), Style::default().fg(app.colorscheme.card_content))
                }
            })
            .collect();

        let context_para = Paragraph::new(context_lines).wrap(Wrap { trim: false });
        f.render_widget(context_para, inner_area);
    }
}
