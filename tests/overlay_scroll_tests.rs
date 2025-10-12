use revw::app::{App, FormatMode};

#[test]
fn test_overlay_scroll_initialization() {
    let app = App::new(FormatMode::View);
    assert_eq!(app.edit_hscroll, 0);
    assert_eq!(app.edit_vscroll, 0);
}

#[test]
fn test_cancel_editing_resets_scroll() {
    let mut app = App::new(FormatMode::View);
    app.editing_entry = true;
    app.edit_hscroll = 10;
    app.edit_vscroll = 5;

    app.cancel_editing_entry();

    assert_eq!(app.edit_hscroll, 0);
    assert_eq!(app.edit_vscroll, 0);
    assert!(!app.editing_entry);
}
