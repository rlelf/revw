use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
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

    // Clear the area
    f.render_widget(Clear, clear_area);

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

    // Render the list
    let list = List::new(items).block(block);

    f.render_widget(list, popup_area);
}
