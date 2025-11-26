use revw::app::{App, FileMode, FormatMode};

#[test]
fn test_order_entries_by_percentage_and_name() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "Zebra",
      "context": "Last alphabetically",
      "url": "https://zebra.com",
      "percentage": 50
    },
    {
      "name": "Apple",
      "context": "First alphabetically",
      "url": "https://apple.com",
      "percentage": 100
    },
    {
      "name": "Banana",
      "context": "Second alphabetically",
      "url": "https://banana.com",
      "percentage": 100
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Older entry"
    },
    {
      "date": "2025-01-15 00:00:00",
      "context": "Newer entry"
    }
  ]
}"#.to_string();

    app.order_entries();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Should be ordered by percentage (100, 100, 50), then by name (Apple, Banana, Zebra)
    assert_eq!(outside[0]["name"], "Apple");
    assert_eq!(outside[0]["percentage"], 100);
    assert_eq!(outside[1]["name"], "Banana");
    assert_eq!(outside[1]["percentage"], 100);
    assert_eq!(outside[2]["name"], "Zebra");
    assert_eq!(outside[2]["percentage"], 50);

    // Inside should be ordered by date (newest first)
    let inside = parsed["inside"].as_array().unwrap();
    assert_eq!(inside[0]["date"], "2025-01-15 00:00:00");
    assert_eq!(inside[1]["date"], "2025-01-01 00:00:00");

    assert!(app.is_modified);
    assert_eq!(app.status_message, "Ordered");
}

#[test]
fn test_order_by_percentage_only() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "Zebra",
      "context": "Should stay last",
      "url": "https://zebra.com",
      "percentage": 100
    },
    {
      "name": "Apple",
      "context": "Should stay middle",
      "url": "https://apple.com",
      "percentage": 50
    },
    {
      "name": "Banana",
      "context": "Should stay first",
      "url": "https://banana.com",
      "percentage": 100
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Older"
    },
    {
      "date": "2025-01-15 00:00:00",
      "context": "Newer"
    }
  ]
}"#.to_string();

    app.order_by_percentage();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Should be ordered by percentage only (100, 100, 50)
    // Name order should be preserved within same percentage
    assert_eq!(outside[0]["percentage"], 100);
    assert_eq!(outside[1]["percentage"], 100);
    assert_eq!(outside[2]["percentage"], 50);
    assert_eq!(outside[2]["name"], "Apple");

    // Inside should be ordered by date (newest first)
    let inside = parsed["inside"].as_array().unwrap();
    assert_eq!(inside[0]["date"], "2025-01-15 00:00:00");

    assert!(app.is_modified);
    assert_eq!(app.status_message, "Ordered by percentage");
}

#[test]
fn test_order_by_name_only() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "Zebra",
      "context": "Last alphabetically",
      "url": "https://zebra.com",
      "percentage": 100
    },
    {
      "name": "Apple",
      "context": "First alphabetically",
      "url": "https://apple.com",
      "percentage": 50
    },
    {
      "name": "Banana",
      "context": "Second alphabetically",
      "url": "https://banana.com",
      "percentage": 75
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Older"
    },
    {
      "date": "2025-01-15 00:00:00",
      "context": "Newer"
    }
  ]
}"#.to_string();

    app.order_by_name();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Should be ordered by name only (alphabetically)
    assert_eq!(outside[0]["name"], "Apple");
    assert_eq!(outside[0]["percentage"], 50);
    assert_eq!(outside[1]["name"], "Banana");
    assert_eq!(outside[1]["percentage"], 75);
    assert_eq!(outside[2]["name"], "Zebra");
    assert_eq!(outside[2]["percentage"], 100);

    // Inside should be ordered by date (newest first)
    let inside = parsed["inside"].as_array().unwrap();
    assert_eq!(inside[0]["date"], "2025-01-15 00:00:00");

    assert!(app.is_modified);
    assert_eq!(app.status_message, "Ordered by name");
}

#[test]
fn test_order_by_percentage_with_null_values() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "No percentage",
      "context": "Test",
      "url": "https://test.com",
      "percentage": null
    },
    {
      "name": "Has percentage",
      "context": "Test",
      "url": "https://test2.com",
      "percentage": 50
    }
  ],
  "inside": []
}"#.to_string();

    app.order_by_percentage();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Entry with percentage should come first (null treated as 0)
    assert_eq!(outside[0]["name"], "Has percentage");
    assert_eq!(outside[1]["name"], "No percentage");
}

#[test]
fn test_order_by_name_case_sensitive() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "zebra",
      "percentage": 0
    },
    {
      "name": "Apple",
      "percentage": 0
    },
    {
      "name": "banana",
      "percentage": 0
    }
  ],
  "inside": []
}"#.to_string();

    app.order_by_name();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Should be ordered alphabetically (case-sensitive: uppercase before lowercase)
    assert_eq!(outside[0]["name"], "Apple");
    assert_eq!(outside[1]["name"], "banana");
    assert_eq!(outside[2]["name"], "zebra");
}

#[test]
fn test_order_empty_arrays() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [],
  "inside": []
}"#.to_string();

    app.order_entries();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    assert_eq!(parsed["outside"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["inside"].as_array().unwrap().len(), 0);

    assert!(app.is_modified);
}

#[test]
fn test_order_by_percentage_descending() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "Low",
      "percentage": 25
    },
    {
      "name": "High",
      "percentage": 100
    },
    {
      "name": "Medium",
      "percentage": 50
    }
  ],
  "inside": []
}"#.to_string();

    app.order_by_percentage();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Should be ordered highest to lowest
    assert_eq!(outside[0]["percentage"], 100);
    assert_eq!(outside[1]["percentage"], 50);
    assert_eq!(outside[2]["percentage"], 25);
}

#[test]
fn test_order_preserves_other_fields() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json;
    app.json_input = r#"{
  "outside": [
    {
      "name": "B",
      "context": "Context B",
      "url": "https://b.com",
      "percentage": 50
    },
    {
      "name": "A",
      "context": "Context A",
      "url": "https://a.com",
      "percentage": 100
    }
  ],
  "inside": []
}"#.to_string();

    app.order_by_name();

    let parsed: serde_json::Value = serde_json::from_str(&app.json_input).unwrap();
    let outside = parsed["outside"].as_array().unwrap();

    // Check that all fields are preserved
    assert_eq!(outside[0]["name"], "A");
    assert_eq!(outside[0]["context"], "Context A");
    assert_eq!(outside[0]["url"], "https://a.com");
    assert_eq!(outside[0]["percentage"], 100);
}
