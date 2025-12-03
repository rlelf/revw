mod json_highlight;
pub mod markdown_highlight;
mod utils;
mod status_bar;
mod explorer;
mod cards;
mod edit_overlay;
mod content;
mod outline;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

use content::render_content;
use edit_overlay::render_edit_overlay;
use explorer::render_explorer;
use outline::render_outline;
use status_bar::render_status_bar;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Split horizontally based on explorer (left) and outline (right) panels
    // Side panels are 20% each
    let content_area = match (app.explorer_open, app.outline_open) {
        (true, true) => {
            // Both explorer and outline open: [explorer 20%] [content 60%] [outline 20%]
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(chunks[0]);

            render_explorer(f, app, horizontal_chunks[0]);
            render_outline(f, app, horizontal_chunks[2]);
            horizontal_chunks[1]
        }
        (true, false) => {
            // Only explorer open: [explorer 20%] [content 80%]
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(chunks[0]);

            render_explorer(f, app, horizontal_chunks[0]);
            horizontal_chunks[1]
        }
        (false, true) => {
            // Only outline open: [content 80%] [outline 20%]
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(chunks[0]);

            render_outline(f, app, horizontal_chunks[1]);
            horizontal_chunks[0]
        }
        (false, false) => {
            // Neither open: full content area
            chunks[0]
        }
    };

    // Always render content and status bar (even when overlay is active)
    render_content(f, app, content_area);
    render_status_bar(f, app, chunks[1]);

    // Render editing overlay on top if active
    if app.editing_entry {
        render_edit_overlay(f, app);
    }
}
