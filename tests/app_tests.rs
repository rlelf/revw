use revw::app::{App, FileMode, FormatMode, InputMode};

#[test]
fn test_app_creation() {
    let app = App::new(FormatMode::View);
    assert_eq!(app.json_input, "");
    assert_eq!(app.rendered_content.len(), 0);
    assert_eq!(app.scroll, 0);
    assert!(matches!(app.input_mode, InputMode::Normal));
    assert!(matches!(app.format_mode, FormatMode::View));
}

#[test]
fn test_app_creation_view() {
    let app = App::new(FormatMode::View);
    assert!(matches!(app.format_mode, FormatMode::View));
}

#[test]
fn test_simple_json_conversion() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{"outside": [], "inside": []}"#.to_string();
    app.convert_json();

    // In View mode, entries are stored in relf_entries, not rendered_content
    assert!(app.relf_entries.is_empty()); // Empty JSON should produce empty entries
}

#[test]
fn test_relf_format_conversion() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input =
        r#"{"outside": [{"field1": "value1"}], "inside": [{"field2": "value2"}]}"#.to_string();
    app.convert_json();

    // In View mode, entries are stored in relf_entries
    assert_eq!(app.relf_entries.len(), 2); // One outside + one inside entry
}

#[test]
fn test_empty_json_handling() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = "".to_string();
    app.convert_json();

    assert!(app.rendered_content.is_empty());
}

#[test]
fn test_scroll_functionality() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
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
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test

    app.set_status("Test message");
    assert_eq!(app.status_message, "Test message");
    assert!(app.status_time.is_some());
}

#[test]
fn test_clear_content() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{"outside": [{"name": "test"}], "inside": []}"#.to_string();
    app.convert_json();
    app.scroll = 5;
    app.status_message = "test".to_string();

    // Verify entries exist before clearing
    assert!(!app.relf_entries.is_empty());

    app.clear_content();

    assert!(app.json_input.is_empty());
    assert!(app.rendered_content.is_empty());
    assert!(app.relf_entries.is_empty());
    assert_eq!(app.selected_entry_index, 0);
    assert_eq!(app.scroll, 0);
    assert_eq!(app.status_message, "Content cleared");
    assert!(app.file_path.is_none());
}

#[test]
fn test_clear_inside() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
  "outside": [
    {
      "name": "Test Outside",
      "context": "Outside entry",
      "url": "https://example.com",
      "percentage": 50
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Inside entry"
    }
  ]
}"#.to_string();
    app.convert_json();

    // Verify we have 2 entries (1 outside + 1 inside)
    assert_eq!(app.relf_entries.len(), 2);

    app.clear_inside();

    // After clearing inside, should only have 1 entry (outside)
    assert_eq!(app.relf_entries.len(), 1);
    assert_eq!(app.status_message, "INSIDE section cleared");
    assert!(app.is_modified);

    // Verify the JSON structure is correct
    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    assert_eq!(parsed["inside"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["outside"].as_array().unwrap().len(), 1);
}

#[test]
fn test_clear_outside() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
  "outside": [
    {
      "name": "Test Outside",
      "context": "Outside entry",
      "url": "https://example.com",
      "percentage": 50
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Inside entry"
    }
  ]
}"#.to_string();
    app.convert_json();

    // Verify we have 2 entries (1 outside + 1 inside)
    assert_eq!(app.relf_entries.len(), 2);
    app.selected_entry_index = 0; // Select outside entry

    app.clear_outside();

    // After clearing outside, should only have 1 entry (inside)
    assert_eq!(app.relf_entries.len(), 1);
    assert_eq!(app.selected_entry_index, 0); // Should reset to first entry
    assert_eq!(app.status_message, "OUTSIDE section cleared");
    assert!(app.is_modified);

    // Verify the JSON structure is correct
    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    assert_eq!(parsed["outside"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["inside"].as_array().unwrap().len(), 1);
}

#[test]
fn test_clear_content_in_edit_mode() {
    let mut app = App::new(FormatMode::Edit);
    app.json_input = r#"{"outside": [], "inside": []}"#.to_string();
    app.convert_json();

    app.clear_content();

    assert!(app.json_input.is_empty());
    assert!(app.rendered_content.is_empty());
    assert_eq!(app.status_message, "Content cleared");
}

#[test]
fn test_substitute_current_line_first_match() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo baz\nqux quux".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/foo/replaced/");

    assert_eq!(app.json_input, "replaced bar foo baz\nqux quux\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_current_line_all_matches() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo baz\nqux quux".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/foo/replaced/g");

    assert_eq!(app.json_input, "replaced bar replaced baz\nqux quux\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_entire_file_all_matches() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo\nbaz foo qux\nfoo quux".to_string();

    app.execute_substitute("%s/foo/replaced/g");

    assert_eq!(app.json_input, "replaced bar replaced\nbaz replaced qux\nreplaced quux\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_pattern_not_found() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar baz\nqux quux".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/notfound/replaced/");

    assert_eq!(app.json_input, "foo bar baz\nqux quux");
    assert!(!app.is_modified);
    assert!(app.status_message.contains("Pattern not found"));
}

#[test]
fn test_substitute_empty_pattern() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar\nbaz qux".to_string();

    app.execute_substitute("s//replaced/");

    assert_eq!(app.json_input, "foo bar\nbaz qux");
    assert!(!app.is_modified);
    assert!(app.status_message.contains("Empty pattern"));
}

#[test]
fn test_substitute_invalid_syntax() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar\nbaz qux".to_string();

    app.execute_substitute("s/foo");

    assert_eq!(app.json_input, "foo bar\nbaz qux");
    assert!(!app.is_modified);
    assert!(app.status_message.contains("Invalid substitute syntax"));
}

#[test]
fn test_substitute_only_in_edit_mode() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = "foo bar\nbaz qux".to_string();

    app.execute_substitute("s/foo/replaced/");

    assert_eq!(app.json_input, "foo bar\nbaz qux");
    assert!(!app.is_modified);
    assert!(app.status_message.contains("Substitute only works in Edit mode"));
}

#[test]
fn test_substitute_current_line_second_line() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "first line\nfoo bar foo\nthird line".to_string();
    app.content_cursor_line = 1;

    app.execute_substitute("s/foo/replaced/");

    assert_eq!(app.json_input, "first line\nreplaced bar foo\nthird line\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_entire_file_first_match_per_line() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo\nbaz foo qux\nfoo quux foo".to_string();

    app.execute_substitute("%s/foo/replaced/");

    assert_eq!(app.json_input, "replaced bar foo\nbaz replaced qux\nreplaced quux foo\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_with_special_characters() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "test@example.com\nfoo@bar.com".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/@/[at]/");

    assert_eq!(app.json_input, "test[at]example.com\nfoo@bar.com\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_empty_replacement() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar baz\nqux quux".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/foo//");

    assert_eq!(app.json_input, " bar baz\nqux quux\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_multiple_spaces() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo  bar  baz\nqux quux".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/  / /g");

    assert_eq!(app.json_input, "foo bar baz\nqux quux\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_confirmation_mode_builds_matches() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo\nbaz foo".to_string();

    app.execute_substitute("%s/foo/replaced/gc");

    // Should have 3 matches waiting for confirmation
    assert_eq!(app.substitute_confirmations.len(), 3);
    assert_eq!(app.current_substitute_index, 0);
    assert!(app.status_message.contains("Replace with"));
    assert!(!app.is_modified); // Not modified until confirmations are processed
}

#[test]
fn test_substitute_confirmation_single_match() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar baz".to_string();
    app.content_cursor_line = 0;

    app.execute_substitute("s/foo/replaced/c");

    assert_eq!(app.substitute_confirmations.len(), 1);
    assert_eq!(app.current_substitute_index, 0);
}

#[test]
fn test_substitute_confirmation_no_matches() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar baz".to_string();

    app.execute_substitute("%s/notfound/replaced/gc");

    assert_eq!(app.substitute_confirmations.len(), 0);
    assert!(app.status_message.contains("Pattern not found"));
}

#[test]
fn test_substitute_json_content() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{"name": "test", "value": "test"}"#.to_string();

    app.execute_substitute("%s/test/replaced/g");

    assert_eq!(app.json_input, r#"{"name": "replaced", "value": "replaced"}"#.to_string() + "\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_preserves_other_lines() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "line1\nfoo bar\nline3\nline4".to_string();
    app.content_cursor_line = 1;

    app.execute_substitute("s/foo/replaced/");

    assert_eq!(app.json_input, "line1\nreplaced bar\nline3\nline4\n");
}

#[test]
fn test_substitute_entire_word_replacement() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo foobar foo\nbarfoo foo".to_string();

    app.execute_substitute("%s/foo/XXX/g");

    // Note: This is simple string replacement, not word boundaries
    assert_eq!(app.json_input, "XXX XXXbar XXX\nbarXXX XXX\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_case_sensitive() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "Foo foo FOO\nfoo Foo".to_string();

    app.execute_substitute("%s/foo/replaced/g");

    // Should only replace lowercase 'foo'
    assert_eq!(app.json_input, "Foo replaced FOO\nreplaced Foo\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_numbers() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "version 1.0.0\nversion 1.0.0".to_string();

    app.execute_substitute("%s/1.0.0/2.0.0/g");

    assert_eq!(app.json_input, "version 2.0.0\nversion 2.0.0\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_single_line_file() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo".to_string();

    app.execute_substitute("%s/foo/replaced/g");

    assert_eq!(app.json_input, "replaced bar replaced\n");
    assert!(app.is_modified);
}

#[test]
fn test_substitute_undo() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo".to_string();
    let original = app.json_input.clone();

    app.execute_substitute("%s/foo/replaced/g");
    assert_eq!(app.json_input, "replaced bar replaced\n");

    app.undo();
    assert_eq!(app.json_input, original);
}

#[test]
fn test_substitute_redo() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo".to_string();

    app.execute_substitute("%s/foo/replaced/g");
    let modified = app.json_input.clone();

    app.undo();
    app.redo();
    assert_eq!(app.json_input, modified);
}

#[test]
fn test_substitute_no_undo_when_pattern_not_found() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar".to_string();

    let undo_stack_size = app.undo_stack.len();
    app.execute_substitute("s/notfound/replaced/");

    // Undo stack should be same size (no undo state added for failed substitute)
    assert_eq!(app.undo_stack.len(), undo_stack_size);
}

#[test]
fn test_substitute_multiple_operations_undo() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar baz".to_string();
    let original = app.json_input.clone();

    app.execute_substitute("s/foo/first/");
    app.execute_substitute("s/bar/second/");

    app.undo(); // Undo second substitution
    assert!(app.json_input.contains("bar"));
    assert!(app.json_input.contains("first"));

    app.undo(); // Undo first substitution
    assert_eq!(app.json_input, original);
}

#[test]
fn test_substitute_confirmation_accept() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar".to_string();

    // Start confirmation mode
    app.execute_substitute("s/foo/replaced/c");
    assert_eq!(app.substitute_confirmations.len(), 1);

    // Accept the substitution
    app.handle_substitute_confirmation('y');

    assert_eq!(app.json_input, "replaced bar\n");
    assert!(app.substitute_confirmations.is_empty());
    assert!(app.is_modified);
}

#[test]
fn test_substitute_confirmation_reject() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar".to_string();
    let original = app.json_input.clone();

    // Start confirmation mode
    app.execute_substitute("s/foo/replaced/c");

    // Reject the substitution
    app.handle_substitute_confirmation('n');

    assert_eq!(app.json_input, original);
    assert!(app.substitute_confirmations.is_empty());
}

#[test]
fn test_substitute_confirmation_quit() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo".to_string();
    let original = app.json_input.clone();

    // Start confirmation mode with multiple matches
    app.execute_substitute("s/foo/replaced/gc");
    assert_eq!(app.substitute_confirmations.len(), 2);

    // Quit confirmation
    app.handle_substitute_confirmation('q');

    assert_eq!(app.json_input, original);
    assert!(app.substitute_confirmations.is_empty());
    assert!(app.status_message.contains("cancelled"));
}

#[test]
fn test_substitute_confirmation_all() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    app.json_input = "foo bar foo baz".to_string();

    // Start confirmation mode
    app.execute_substitute("s/foo/replaced/gc");
    assert_eq!(app.substitute_confirmations.len(), 2);

    // Accept all substitutions
    app.handle_substitute_confirmation('a');

    assert_eq!(app.json_input, "replaced bar replaced baz\n");
    assert!(app.substitute_confirmations.is_empty());
    assert!(app.is_modified);
}

#[test]
fn test_substitute_confirmation_selective() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    // Use different lines to avoid position shift issues
    app.json_input = "foo bar\nbaz qux\nfoo quux".to_string();

    // Start confirmation mode with 2 matches (one per line)
    app.execute_substitute("%s/foo/replaced/gc");
    assert_eq!(app.substitute_confirmations.len(), 2);

    // Accept first
    app.handle_substitute_confirmation('y');
    assert_eq!(app.substitute_confirmations.len(), 2); // List doesn't shrink
    assert_eq!(app.current_substitute_index, 1);
    assert!(app.json_input.contains("replaced bar"));

    // Reject second (it won't be replaced)
    app.handle_substitute_confirmation('n');
    assert!(app.substitute_confirmations.is_empty());

    // First line replaced, third line not replaced
    assert_eq!(app.json_input, "replaced bar\nbaz qux\nfoo quux\n");
}

#[test]
fn test_view_mode_parsing() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
        "outside": [
            {
                "name": "Test Resource",
                "context": "Test context",
                "url": "https://test.com",
                "percentage": 50
            }
        ],
        "inside": [
            {
                "date": "2025-01-01 00:00:00",
                "context": "Test note"
            }
        ]
    }"#.to_string();

    app.convert_json();

    // Should have 2 entries (1 outside + 1 inside)
    assert_eq!(app.relf_entries.len(), 2);

    // First entry should be outside entry with 4 lines
    assert_eq!(app.relf_entries[0].lines.len(), 4);
    assert_eq!(app.relf_entries[0].lines[0], "Test Resource");
    assert_eq!(app.relf_entries[0].lines[3], "50%");

    // Second entry should be inside entry
    assert_eq!(app.relf_entries[1].lines.len(), 2);
}

#[test]
fn test_edit_mode_json_output() {
    let mut app = App::new(FormatMode::Edit);
    app.file_mode = FileMode::Json;
    let json = r#"{"test": "value"}"#;
    app.json_input = json.to_string();
    app.convert_json();

    // In Edit mode, rendered_content should contain the JSON lines
    assert!(!app.rendered_content.is_empty());
}

#[test]
fn test_empty_outside_section() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
        "outside": [],
        "inside": [
            {
                "date": "2025-01-01",
                "context": "Note"
            }
        ]
    }"#.to_string();

    app.convert_json();

    // Should only have 1 entry (inside)
    assert_eq!(app.relf_entries.len(), 1);
}

#[test]
fn test_empty_inside_section() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
        "outside": [
            {
                "name": "Resource",
                "percentage": 100
            }
        ],
        "inside": []
    }"#.to_string();

    app.convert_json();

    // Should only have 1 entry (outside)
    assert_eq!(app.relf_entries.len(), 1);
}

#[test]
fn test_outside_entry_missing_fields() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test
    app.json_input = r#"{
        "outside": [
            {
                "name": "Resource Only"
            }
        ],
        "inside": []
    }"#.to_string();

    app.convert_json();

    // Should have 1 entry
    assert_eq!(app.relf_entries.len(), 1);

    // Entry should have name but no percentage (null percentage is not rendered)
    assert!(app.relf_entries[0].lines.iter().any(|l| l == "Resource Only"));
    assert!(!app.relf_entries[0].lines.iter().any(|l| l == "0%"));
}

#[test]
fn test_mode_toggle() {
    let app = App::new(FormatMode::View);
    assert_eq!(app.format_mode, FormatMode::View);

    let app2 = App::new(FormatMode::Edit);
    assert_eq!(app2.format_mode, FormatMode::Edit);
}
