use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_outline(f: &mut Frame, app: &App, area: Rect) {
    let title = " Outline ";
    let border_color = app.colorscheme.explorer_border;

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.colorscheme.explorer_title))
        .borders(Borders::ALL)
        .border_type(app.border_style.to_border_type())
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(app.colorscheme.background));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Get outline entries
    let entries = app.get_outline_entries();

    // Calculate visible range
    let visible_height = inner_area.height as usize;
    let total_items = entries.len();

    // Auto-scroll to keep selected item visible
    let scroll = if total_items > visible_height {
        let selected = app.outline_selected_index;
        let current_scroll = app.outline_scroll as usize;

        // If selected is below visible area, scroll down
        if selected >= current_scroll + visible_height {
            selected - visible_height + 1
        }
        // If selected is above visible area, scroll up
        else if selected < current_scroll {
            selected
        }
        // Otherwise keep current scroll
        else {
            current_scroll
        }
    } else {
        0
    };

    let start = scroll.min(total_items.saturating_sub(1));
    let end = (start + visible_height).min(total_items);

    // Render entries
    let mut lines = Vec::new();
    for (i, entry) in entries[start..end].iter().enumerate() {
        let abs_index = start + i;
        let is_selected = abs_index == app.outline_selected_index;

        let style = if is_selected {
            Style::default()
                .fg(app.colorscheme.explorer_file_selected)
                .bg(Color::Rgb(60, 60, 60))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(app.colorscheme.text)
        };

        lines.push(Line::styled(entry.clone(), style));
    }

    let content = Paragraph::new(lines)
        .scroll((0, app.outline_horizontal_scroll));
    f.render_widget(content, inner_area);
}
