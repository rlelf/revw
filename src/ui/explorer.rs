use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_explorer(f: &mut Frame, app: &App, area: Rect) {
    // Show only folder name, not full path
    let title = if let Some(folder_name) = app.explorer_current_dir.file_name().and_then(|n| n.to_str()) {
        format!(" {} ", folder_name)
    } else {
        " . ".to_string()
    };

    // Use explorer-specific colors
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.colorscheme.explorer_title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.colorscheme.explorer_border))
        .style(Style::default().bg(app.colorscheme.background));

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
        let mut name = entry.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???")
            .to_string();

        // Remove extension if show_extension is false and it's a file
        if !app.show_extension && entry.path.is_file() {
            if let Some(stem) = entry.path.file_stem().and_then(|s| s.to_str()) {
                name = stem.to_string();
            }
        }

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

        // Show directories and files with colorscheme colors
        let color = if is_selected {
            // Selected file/folder uses bright color
            app.colorscheme.explorer_file_selected
        } else if entry.path.is_dir() {
            app.colorscheme.explorer_folder
        } else {
            app.colorscheme.explorer_file
        };

        let style = if is_selected {
            Style::default().fg(color).bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        lines.push(Line::styled(display_text, style));
    }

    // Render without wrapping and apply horizontal scroll
    let content = Paragraph::new(lines)
        .scroll((0, app.explorer_horizontal_scroll));
    f.render_widget(content, inner_area);
}
