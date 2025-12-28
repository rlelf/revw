use revw::app::{App, FileMode, FormatMode};
use revw::content_ops::ContentOperations;
use revw::toon_ops::ToonOperations;
use serde_json::Value;

#[test]
fn test_parse_toon_basic() {
    let app = App::new(FormatMode::View);

    let toon_content = r#"outside[1]{name,context,url,percentage}:
  "Rust Programming Language","A systems programming language focused on safety, speed, and concurrency.",https://www.rust-lang.org/,100

inside[1]{date,context}:
  "2025-01-01 00:00:00","Finally learned how to use cargo! Running 'cargo new my_project' creates such a clean project structure."
"#;

    let result = app.parse_toon(toon_content);
    assert!(result.is_ok(), "Failed to parse toon: {:?}", result.err());

    let json_str = result.unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Check outside section
    let outside = json.get("outside").unwrap().as_array().unwrap();
    assert_eq!(outside.len(), 1);
    assert_eq!(outside[0]["name"], "Rust Programming Language");
    assert_eq!(outside[0]["percentage"], 100);

    // Check inside section
    let inside = json.get("inside").unwrap().as_array().unwrap();
    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0]["date"], "2025-01-01 00:00:00");
}

#[test]
fn test_convert_to_toon() {
    let mut app = App::new(FormatMode::View);
    app.file_mode = FileMode::Json; // Explicitly set to JSON mode for this test

    app.json_input = r#"{
  "outside": [
    {
      "name": "Rust Programming Language",
      "context": "A systems programming language focused on safety, speed, and concurrency.",
      "url": "https://www.rust-lang.org/",
      "percentage": 100
    }
  ],
  "inside": [
    {
      "date": "2025-01-01 00:00:00",
      "context": "Finally learned how to use cargo! Running 'cargo new my_project' creates such a clean project structure."
    }
  ]
}"#.to_string();

    let result = app.convert_to_toon();
    assert!(result.is_ok(), "Failed to convert to toon: {:?}", result.err());

    let toon_str = result.unwrap();
    assert!(toon_str.contains("outside[1]{name,context,url,percentage}:"));
    assert!(toon_str.contains("inside[1]{date,context}:"));
    assert!(toon_str.contains("Rust Programming Language"));
}

#[test]
fn test_parse_toon_multiple_entries() {
    let app = App::new(FormatMode::View);

    let toon_content = r#"outside[2]{name,context,url,percentage}:
  "Entry 1","Context 1",https://example.com/1,90
  "Entry 2","Context 2",https://example.com/2,85

inside[2]{date,context}:
  "2025-01-01 10:00:00","First entry"
  "2025-01-02 11:00:00","Second entry"
"#;

    let result = app.parse_toon(toon_content);
    assert!(result.is_ok(), "Failed to parse toon: {:?}", result.err());

    let json_str = result.unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Check outside section
    let outside = json.get("outside").unwrap().as_array().unwrap();
    assert_eq!(outside.len(), 2);

    // Check inside section
    let inside = json.get("inside").unwrap().as_array().unwrap();
    assert_eq!(inside.len(), 2);
}

#[test]
fn test_parse_toon_with_comma_in_quotes() {
    let app = App::new(FormatMode::View);

    let toon_content = r#"outside[1]{name,context,url,percentage}:
  "Test, Name","Context with, commas",https://example.com,75
"#;

    let result = app.parse_toon(toon_content);
    assert!(result.is_ok(), "Failed to parse toon: {:?}", result.err());

    let json_str = result.unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    let outside = json.get("outside").unwrap().as_array().unwrap();
    assert_eq!(outside[0]["name"], "Test, Name");
    assert_eq!(outside[0]["context"], "Context with, commas");
}
