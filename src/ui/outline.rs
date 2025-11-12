use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_outline(f: &mut Frame, app: &App) {
    let area = f.area();

    // Create a centered popup (70% width, 70% height)
    let popup_width = ((area.width * 7) / 10).max(40).min(area.width - 4);
    let popup_height = ((area.height * 7) / 10).max(10).min(area.height - 4);

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
        width: popup_width
            .saturating_add(2)
            .min(area.width.saturating_sub(x_aligned.saturating_sub(1))),
        height: popup_height,
    };

    // Clear the wider area to fully erase any wide characters
    f.render_widget(Clear, clear_area);

    // Fill the clear area with background color using spaces
    let blank_lines: Vec<Line> = (0..clear_area.height)
        .map(|_| Line::from(" ".repeat(clear_area.width as usize)))
        .collect();
    let blank_paragraph = Paragraph::new(blank_lines)
        .style(Style::default().bg(app.colorscheme.background));
    f.render_widget(blank_paragraph, clear_area);

    // Get outline entries
    let entries = app.get_outline_entries();

    // Create list items
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.outline_selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(app.colorscheme.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.colorscheme.text)
                    .bg(app.colorscheme.background)
            };

            ListItem::new(Line::from(Span::styled(entry.clone(), style)))
        })
        .collect();

    // Create title
    let title = " Outline ".to_string();

    // Render the block with border
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(
            Style::default()
                .bg(app.colorscheme.background)
                .fg(app.colorscheme.text),
        );

    // Render the list with scroll support
    let list = List::new(items).block(block);

    // Calculate the scroll position
    // Make sure the selected item is visible
    let visible_height = popup_area.height.saturating_sub(2) as usize; // Subtract 2 for borders
    let total_items = entries.len();

    // Auto-scroll to keep selected item visible
    let scroll = if total_items > visible_height {
        let selected = app.outline_selected_index;
        let current_scroll = app.outline_scroll as usize;

        // If selected is below visible area, scroll down
        if selected >= current_scroll + visible_height {
            (selected - visible_height + 1) as u16
        }
        // If selected is above visible area, scroll up
        else if selected < current_scroll {
            selected as u16
        }
        // Otherwise keep current scroll
        else {
            app.outline_scroll
        }
    } else {
        0
    };

    // Create a stateful list with scroll
    use ratatui::widgets::ListState;
    let mut list_state = ListState::default();
    list_state.select(Some(app.outline_selected_index));

    // Set offset for scrolling when there are many items
    if total_items > visible_height {
        *list_state.offset_mut() = scroll as usize;
    }

    f.render_stateful_widget(list, popup_area, &mut list_state);
}
