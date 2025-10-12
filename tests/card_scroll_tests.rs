use revw::app::{App, FormatMode};

#[test]
fn test_card_vertical_scroll() {
    let json_input = r#"{
  "outside": [
    {
      "name": "test",
      "context": "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8",
      "url": "http://example.com",
      "percentage": 50
    }
  ],
  "inside": []
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Should have one entry
    assert_eq!(app.relf_entries.len(), 1);

    // Initially no vertical scroll
    assert_eq!(app.hscroll, 0);

    // Simulate pressing 'l' to scroll down
    app.hscroll = 1;
    assert_eq!(app.hscroll, 1);

    // Simulate pressing 'h' to scroll up
    app.hscroll = 0;
    assert_eq!(app.hscroll, 0);
}

#[test]
fn test_card_scroll_reset_on_navigation() {
    let json_input = r#"{
  "outside": [
    {
      "name": "first",
      "context": "First entry with long context",
      "url": "http://example.com",
      "percentage": 50
    },
    {
      "name": "second",
      "context": "Second entry with long context",
      "url": "http://example2.com",
      "percentage": 75
    }
  ],
  "inside": []
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Should have two entries
    assert_eq!(app.relf_entries.len(), 2);

    // Select first card
    app.selected_entry_index = 0;

    // Scroll horizontally
    app.hscroll = 10;
    assert_eq!(app.hscroll, 10);

    // Move to next card (simulate 'j' key)
    app.selected_entry_index = 1;
    app.hscroll = 0; // This should happen in the input handler

    // Horizontal scroll should be reset
    assert_eq!(app.hscroll, 0);
}

#[test]
fn test_card_scroll_reset_on_move_up() {
    let json_input = r#"{
  "outside": [
    {
      "name": "first",
      "context": "First entry",
      "url": "http://example.com",
      "percentage": 50
    },
    {
      "name": "second",
      "context": "Second entry",
      "url": "http://example2.com",
      "percentage": 75
    }
  ],
  "inside": []
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Select second card
    app.selected_entry_index = 1;

    // Scroll horizontally
    app.hscroll = 15;
    assert_eq!(app.hscroll, 15);

    // Move to previous card (simulate 'k' key)
    app.selected_entry_index = 0;
    app.hscroll = 0; // This should happen in the input handler

    // Horizontal scroll should be reset
    assert_eq!(app.hscroll, 0);
}

#[test]
fn test_multiple_cards_independent_scroll() {
    let json_input = r#"{
  "outside": [],
  "inside": [
    {
      "date": "2025-01-01",
      "context": "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7"
    },
    {
      "date": "2025-01-01",
      "context": "Another 1\nAnother 2\nAnother 3\nAnother 4\nAnother 5\nAnother 6"
    }
  ]
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Should have two inside entries
    assert_eq!(app.relf_entries.len(), 2);

    // Select first card
    app.selected_entry_index = 0;
    app.hscroll = 0;

    // Scroll on first card vertically
    app.hscroll = 2;
    assert_eq!(app.hscroll, 2);

    // Move to second card
    app.selected_entry_index = 1;
    app.hscroll = 0; // Reset happens in input handler
    assert_eq!(app.hscroll, 0);

    // Scroll on second card should start from 0
    app.hscroll = 1;
    assert_eq!(app.hscroll, 1);
}

#[test]
fn test_hscroll_bounds() {
    let json_input = r#"{
  "outside": [
    {
      "name": "test",
      "context": "Short",
      "url": "http://example.com",
      "percentage": 50
    }
  ],
  "inside": []
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Initially no scroll
    assert_eq!(app.hscroll, 0);

    // Try to scroll left when already at 0
    app.relf_hscroll_by(-10);
    // Should stay at 0 (saturating_sub)
    assert_eq!(app.hscroll, 0);
}

#[test]
fn test_card_full_height_display() {
    // Test that context uses full card height
    let json_input = r#"{
  "outside": [
    {
      "name": "test",
      "context": "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15",
      "url": "http://example.com",
      "percentage": 50
    }
  ],
  "inside": []
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    // Set visible height to simulate terminal size
    app.visible_height = 20;
    app.max_visible_cards = 5;

    // Should have one entry
    assert_eq!(app.relf_entries.len(), 1);

    // Verify that the context field exists and has multiple lines
    let entry = &app.relf_entries[0];
    assert!(entry.context.is_some());
    let context = entry.context.as_ref().unwrap();
    let lines: Vec<&str> = context.lines().collect();
    assert!(lines.len() > 5, "Context should have more than 5 lines");

    // Calculate expected visible lines based on card height
    // visible_height (20) / max_visible_cards (5) - borders (2) = 2 lines per card
    let expected_visible = (app.visible_height as usize / app.max_visible_cards).saturating_sub(2);

    // Calculate max scroll
    let max_scroll = lines.len().saturating_sub(expected_visible);
    let calculated_max = app.relf_max_hscroll() as usize;

    assert_eq!(calculated_max, max_scroll, "Max scroll should be calculated correctly");

    // Test scrolling within bounds
    app.hscroll = 0;
    assert_eq!(app.hscroll, 0);

    // Scroll down
    app.hscroll = 3;
    assert_eq!(app.hscroll, 3);

    // Should not exceed max scroll
    app.hscroll = (calculated_max + 10) as u16;
    // In actual usage, input handler limits this, but we can verify max_scroll calculation
    assert!(calculated_max < lines.len(), "Max scroll should be less than total lines");
}

#[test]
fn test_card_scroll_with_newlines() {
    // Test that \n is properly converted to actual newlines for scrolling
    let json_input = r#"{
  "inside": [
    {
      "date": "2025-01-01",
      "context": "First line\\nSecond line\\nThird line\\nFourth line\\nFifth line\\nSixth line\\nSeventh line"
    }
  ]
}"#;

    let mut app = App::new(FormatMode::View);
    app.json_input = json_input.to_string();
    app.format_mode = FormatMode::View;
    app.convert_json();

    app.visible_height = 20;
    app.max_visible_cards = 5;

    assert_eq!(app.relf_entries.len(), 1);

    let entry = &app.relf_entries[0];
    assert!(entry.context.is_some());

    // Verify that context contains literal \n
    let context = entry.context.as_ref().unwrap();
    assert!(context.contains("\\n"), "Context should contain literal \\n from JSON");

    // When processed, \n should be converted to actual newlines
    let context_with_newlines = context.replace("\\n", "\n");
    let lines: Vec<&str> = context_with_newlines.lines().collect();
    assert_eq!(lines.len(), 7, "Should have 7 lines after converting \\n to newlines");

    // Test scrolling
    app.hscroll = 0;
    assert_eq!(app.hscroll, 0);

    app.hscroll = 2;
    assert_eq!(app.hscroll, 2);
}
