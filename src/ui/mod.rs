mod json_highlight;
mod markdown_highlight;
mod utils;
mod status_bar;
mod explorer;
mod cards;
mod edit_overlay;
mod content;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

use content::render_content;
use edit_overlay::render_edit_overlay;
use explorer::render_explorer;
use status_bar::render_status_bar;

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
