use revw::app::{App, FormatMode, InputMode};

#[test]
fn test_app_creation() {
    let app = App::new(FormatMode::Relf);
    assert_eq!(app.json_input, "");
    assert_eq!(app.rendered_content.len(), 0);
    assert_eq!(app.scroll, 0);
    assert!(matches!(app.input_mode, InputMode::Normal));
    assert!(matches!(app.format_mode, FormatMode::Relf));
}

#[test]
fn test_app_creation_relf() {
    let app = App::new(FormatMode::Relf);
    assert!(matches!(app.format_mode, FormatMode::Relf));
}

#[test]
fn test_simple_json_conversion() {
    let mut app = App::new(FormatMode::Relf);
    app.json_input = r#"{"outside": [], "inside": []}"#.to_string();
    app.convert_json();
    
    assert!(!app.rendered_content.is_empty());
}

#[test]
fn test_relf_format_conversion() {
    let mut app = App::new(FormatMode::Relf);
    app.json_input = r#"{"outside": [{"field1": "value1"}], "inside": [{"field2": "value2"}]}"#.to_string();
    app.convert_json();
    
    assert!(!app.rendered_content.is_empty());
    let content = app.rendered_content.join("\n");
    assert!(content.contains("OUTSIDE"));
    assert!(content.contains("INSIDE"));
}

#[test]
fn test_empty_json_handling() {
    let mut app = App::new(FormatMode::Relf);
    app.json_input = "".to_string();
    app.convert_json();
    
    assert!(app.rendered_content.is_empty());
}

#[test]
fn test_scroll_functionality() {
    let mut app = App::new(FormatMode::Relf);
    app.max_scroll = 10;
    
    app.scroll_down();
    assert_eq!(app.scroll, 1);
    
    app.scroll_up();
    assert_eq!(app.scroll, 0);
    
    app.scroll_to_bottom();
    assert_eq!(app.scroll, 10);
    
    app.scroll_to_top();
    assert_eq!(app.scroll, 0);
}

#[test]
fn test_status_message_handling() {
    let mut app = App::new(FormatMode::Relf);
    
    app.set_status("Test message");
    assert_eq!(app.status_message, "Test message");
    assert!(app.status_time.is_some());
}

#[test]
fn test_clear_content() {
    let mut app = App::new(FormatMode::Relf);
    app.json_input = "test".to_string();
    app.rendered_content = vec!["test".to_string()];
    app.scroll = 5;
    app.status_message = "test".to_string();
    
    app.clear_content();
    
    assert!(app.json_input.is_empty());
    assert!(app.rendered_content.is_empty());
    assert_eq!(app.scroll, 0);
    assert!(app.status_message.is_empty());
    assert!(app.file_path.is_none());
}