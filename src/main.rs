mod app;
mod config;
mod content_ops;
mod input;
mod json_ops;
mod markdown_ops;
mod navigation;
mod wrap;
mod rendering;
mod syntax_highlight;
mod ui;

use anyhow::Result;
use clap::{Arg, ArgGroup, Command};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs, io::{self, stdout, Read}, panic, path::PathBuf};

use app::{App, FormatMode};

fn main() -> Result<()> {
    // Set up panic handler to properly clean up terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Clean up terminal
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = execute!(stdout(), cursor::Show);

        // Call the original panic handler
        original_hook(panic_info);
    }));

    let matches = Command::new("revw")
        .version(env!("BUILD_VERSION"))
        .about("A vim-like TUI for managing notes and resources")
        .after_help(
            "EXAMPLES:\n  \
            # Open file in interactive mode\n  \
            revw file.md\n  \
            revw file.json\n\n  \
            # Output to stdout\n  \
            revw --stdout file.md\n  \
            revw --stdout file.json\n\n  \
            # Format conversion\n  \
            revw --stdout --json file.md\n  \
            revw --stdout --markdown file.json\n\n  \
            # Pipe from stdin\n  \
            cat file.md | revw --stdout\n  \
            cat file.json | revw --stdout\n\n  \
            # Filter entries\n  \
            revw --stdout --filter pattern file.md\n  \
            revw --stdout --filter pattern file.json\n  \
            revw --stdout --filter pattern --inside file.md\n  \
            revw --stdout --filter pattern --context 100 file.md\n\n  \
            # Order entries (writes back in-place)\n  \
            revw --order file.md\n  \
            revw --order-percentage file.json\n  \
            revw --order-name file.md\n  \
            revw --order-random file.json\n\n  \
            # Append entries from stdin (JSON or Markdown) into file\n  \
            cat new.md   | revw --append file.md\n  \
            cat new.json | revw --append file.json\n  \
            cat new.md   | revw --append --inside file.md\n\n  \
            # Delete entries by field (writes back in-place)\n  \
            revw --delete-outside-name pattern file.md\n  \
            revw --delete-outside-context pattern file.json\n  \
            revw --delete-inside-date pattern file.md\n  \
            revw --delete-inside-context pattern file.json\n\n\
            SUPPORTED FILE FORMATS:\n  \
            Markdown (file.md):\n  \
            ## OUTSIDE\n  \
            ### Resource\n  \
            Description\n  \
            **URL:** https://...\n  \
            **Percentage:** 100%\n  \
            ## INSIDE\n  \
            ### 2025-01-01 00:00:00\n  \
            Note content\n\n  \
            JSON (file.json):\n  \
            {\n    \
            \"outside\": [{\"name\": \"Resource\", \"context\": \"Description\", \"url\": \"https://...\", \"percentage\": 100}],\n    \
            \"inside\": [{\"date\": \"2025-01-01 00:00:00\", \"context\": \"Note content\"}]\n  \
            }\n\n\
            For interactive help, run 'revw' and press :h or ?"
        )
        .arg(
            Arg::new("file")
                .help("JSON or Markdown file(s) to view (supports multiple files / shell globs)")
                .num_args(0..)
                .index(1),
        )
        .arg(
            Arg::new("edit")
                .long("edit")
                .help("Use Edit mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stdout")
                .long("stdout")
                .help("Output to stdout (also reads from stdin if no file given)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("inside")
                .long("inside")
                .help("Output only INSIDE section")
                .conflicts_with("outside")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("outside")
                .long("outside")
                .help("Output only OUTSIDE section")
                .conflicts_with("inside")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("markdown")
                .long("markdown")
                .help("Output in Markdown format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output in JSON format")
                .action(clap::ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("output_format")
                .args(["markdown", "json"])
                .multiple(false),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .help("Show token counts for all formats and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .help("Filter entries by pattern")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("context")
                .long("context")
                .help("Show N chars before/after match in context field (requires --filter)")
                .requires("filter")
                .value_name("N")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("append")
                .long("append")
                .help("Append entries from stdin (JSON or Markdown) into file; use with --inside/--outside to limit section")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("order")
                .long("order")
                .help("Order entries by percentage then name and write back in-place")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("order-percentage")
                .long("order-percentage")
                .help("Order entries by percentage only and write back in-place")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("order-name")
                .long("order-name")
                .help("Order entries by name only and write back in-place")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("order-random")
                .long("order-random")
                .help("Order entries randomly and write back in-place")
                .action(clap::ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("order_ops")
                .args(["order", "order-percentage", "order-name", "order-random"])
                .multiple(false),
        )
        .arg(
            Arg::new("delete-outside-name")
                .long("delete-outside-name")
                .help("Delete outside entries where 'name' matches PATTERN (writes back in-place)")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("delete-outside-context")
                .long("delete-outside-context")
                .help("Delete outside entries where 'context' matches PATTERN (writes back in-place)")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("delete-inside-date")
                .long("delete-inside-date")
                .help("Delete inside entries where 'date' matches PATTERN (writes back in-place)")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("delete-inside-context")
                .long("delete-inside-context")
                .help("Delete inside entries where 'context' matches PATTERN (writes back in-place)")
                .value_name("PATTERN"),
        )
        .group(
            ArgGroup::new("delete_ops")
                .args(["delete-outside-name", "delete-outside-context", "delete-inside-date", "delete-inside-context"])
                .multiple(false),
        )
        .get_matches();

    let format_mode = if matches.get_flag("edit") {
        FormatMode::Edit
    } else {
        FormatMode::View
    };

    let stdout_mode = matches.get_flag("stdout");
    let inside_only = matches.get_flag("inside");
    let outside_only = matches.get_flag("outside");
    let markdown_mode = matches.get_flag("markdown");
    let json_mode = matches.get_flag("json");
    let token_mode = matches.get_flag("token");
    let filter_pattern = matches.get_one::<String>("filter");
    let context_chars = matches.get_one::<usize>("context").copied();
    let append_mode = matches.get_flag("append");
    let order_op: Option<&str> = if matches.get_flag("order") {
        Some("order")
    } else if matches.get_flag("order-percentage") {
        Some("order-percentage")
    } else if matches.get_flag("order-name") {
        Some("order-name")
    } else if matches.get_flag("order-random") {
        Some("order-random")
    } else {
        None
    };
    let delete_outside_name = matches.get_one::<String>("delete-outside-name");
    let delete_outside_context = matches.get_one::<String>("delete-outside-context");
    let delete_inside_date = matches.get_one::<String>("delete-inside-date");
    let delete_inside_context = matches.get_one::<String>("delete-inside-context");
    let delete_op: Option<(&str, &str)> = delete_outside_name.map(|p| ("outside-name", p.as_str()))
        .or_else(|| delete_outside_context.map(|p| ("outside-context", p.as_str())))
        .or_else(|| delete_inside_date.map(|p| ("inside-date", p.as_str())))
        .or_else(|| delete_inside_context.map(|p| ("inside-context", p.as_str())));

    // Detect if stdin is a pipe (not a tty)
    use std::io::IsTerminal;
    let stdin_piped = !io::stdin().is_terminal();

    // Helper: load content into app from a string, detecting format by path or content
    let load_content = |app: &mut App, content: String, path: Option<PathBuf>| {
        let is_markdown = path.as_ref()
            .and_then(|p| p.extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or_else(|| {
                // Heuristic: if no extension, check content
                content.trim_start().starts_with("## ")
            });

        if is_markdown {
            app.file_path = path;
            app.markdown_input = content;
            if let Ok(json) = app.parse_markdown(&app.markdown_input) {
                app.json_input = json;
            }
        } else {
            app.file_path = path;
            app.json_input = content;
        }
        app.convert_json();
    };

    // Collect file paths
    let file_paths: Vec<String> = matches
        .get_many::<String>("file")
        .unwrap_or_default()
        .cloned()
        .collect();

    // Generate text output for a loaded app
    let generate_output = |app: &App| -> String {
        if format_mode == FormatMode::Edit {
                // In Edit mode, output the JSON as-is
                app.json_input.clone()
            } else {
                // Parse JSON once for all output modes
                let json_value = match serde_json::from_str::<serde_json::Value>(&app.json_input) {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("Error: Invalid JSON");
                        std::process::exit(1);
                    }
                };

                // Apply entry-level filter if --filter was provided
                let json_value = if let Some(pattern) = &filter_pattern {
                    json_ops::JsonOperations::filter_entries(&json_value, pattern)
                } else {
                    json_value
                };

                // Trim context fields around match if --context N was provided
                let json_value = if let (Some(pattern), Some(chars)) = (&filter_pattern, context_chars) {
                    json_ops::JsonOperations::trim_context_around_match(&json_value, pattern, chars)
                } else {
                    json_value
                };

                // Return appropriate output based on mode
                if markdown_mode {
                    // Markdown mode: format entries as Markdown
                    let mut output_lines = Vec::new();

                    if let Some(obj) = json_value.as_object() {
                        // OUTSIDE section
                        if !inside_only {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                if !outside.is_empty() {
                                    output_lines.push("## OUTSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in outside {
                                        if let Some(item_obj) = item.as_object() {
                                            let name = item_obj
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let context = item_obj
                                                .get("context")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let url = item_obj.get("url").and_then(|v| v.as_str());
                                            let percentage =
                                                item_obj.get("percentage").and_then(|v| v.as_i64());

                                            if !name.is_empty() {
                                                output_lines.push(format!("### {}", name));
                                            }

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context =
                                                    context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            // Only output URL if it's not null and not empty
                                            if let Some(url_str) = url {
                                                if !url_str.is_empty() {
                                                    output_lines.push("".to_string());
                                                    output_lines
                                                        .push(format!("**URL:** {}", url_str));
                                                }
                                            }

                                            // Only output percentage if it's not null
                                            if let Some(pct) = percentage {
                                                output_lines.push("".to_string());
                                                output_lines
                                                    .push(format!("**Percentage:** {}%", pct));
                                            }

                                            // Only add blank line if we had any content
                                            if !name.is_empty()
                                                || !context.is_empty()
                                                || url.is_some()
                                                || percentage.is_some()
                                            {
                                                output_lines.push("".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // INSIDE section
                        if !outside_only {
                            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                                if !inside.is_empty() {
                                    output_lines.push("## INSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in inside {
                                        if let Some(item_obj) = item.as_object() {
                                            let date = item_obj
                                                .get("date")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let context = item_obj
                                                .get("context")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");

                                            if !date.is_empty() {
                                                output_lines.push(format!("### {}", date));
                                            }

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context =
                                                    context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            // Only add blank line if we had content
                                            if !date.is_empty() || !context.is_empty() {
                                                output_lines.push("".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    output_lines.join("\n")
                } else if json_mode {
                    // JSON mode: output as JSON
                    // Apply section filtering if needed
                    let filtered_json = if inside_only || outside_only {
                        let mut json_clone = json_value.clone();
                        if let Some(obj) = json_clone.as_object_mut() {
                            if inside_only {
                                obj.remove("outside");
                            }
                            if outside_only {
                                obj.remove("inside");
                            }
                        }
                        json_clone
                    } else {
                        json_value.clone()
                    };

                    serde_json::to_string_pretty(&filtered_json)
                        .unwrap_or_else(|_| app.json_input.clone())
                } else {
                    // In View mode, format the entries for text output
                    if app.relf_entries.is_empty() {
                        // No entries parsed, output raw content or rendered lines
                        if !app.rendered_content.is_empty() {
                            app.rendered_content.join("\n")
                        } else {
                            app.json_input.clone()
                        }
                    } else {
                        // Format entries as text
                        let mut output_lines = Vec::new();
                        let mut outside_entries: Vec<String> = Vec::new();
                        let mut inside_entries: Vec<String> = Vec::new();

                        // Use already parsed JSON
                        if let Some(obj) = json_value.as_object() {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                for item in outside {
                                    if let Some(item_obj) = item.as_object() {
                                        let name = item_obj
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let context = item_obj
                                            .get("context")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let url = item_obj
                                            .get("url")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let percentage =
                                            item_obj.get("percentage").and_then(|v| v.as_i64());

                                        let mut entry = String::new();
                                        entry.push_str(name);
                                        if !context.is_empty() {
                                            entry.push_str(&format!("\n{}", context));
                                        }
                                        if !url.is_empty() {
                                            entry.push_str(&format!("\n{}", url));
                                        }
                                        // Only add percentage if not null
                                        if let Some(pct) = percentage {
                                            entry.push_str(&format!("\n{}%", pct));
                                        }
                                        outside_entries.push(entry);
                                    }
                                }
                            }

                            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                                for item in inside {
                                    if let Some(item_obj) = item.as_object() {
                                        let mut entry_parts = Vec::new();
                                        for (_key, value) in item_obj {
                                            let value_str = match value {
                                                serde_json::Value::String(s) => s.clone(),
                                                serde_json::Value::Number(n) => n.to_string(),
                                                serde_json::Value::Bool(b) => b.to_string(),
                                                _ => value.to_string(),
                                            };
                                            if !value_str.is_empty() {
                                                entry_parts.push(value_str);
                                            }
                                        }
                                        inside_entries.push(entry_parts.join("\n"));
                                    }
                                }
                            }
                        }

                        // Filter based on --inside or --outside flags
                        if inside_only && !outside_only {
                            // Only INSIDE section
                            if !inside_entries.is_empty() {
                                output_lines.push("INSIDE".to_string());
                                output_lines.push("".to_string());
                                for entry in inside_entries {
                                    output_lines.push(entry);
                                    output_lines.push("".to_string());
                                }
                            }
                        } else if outside_only && !inside_only {
                            // Only OUTSIDE section
                            if !outside_entries.is_empty() {
                                output_lines.push("OUTSIDE".to_string());
                                output_lines.push("".to_string());
                                for entry in outside_entries {
                                    output_lines.push(entry);
                                    output_lines.push("".to_string());
                                }
                            }
                        } else {
                            // Both sections (default behavior)
                            if !outside_entries.is_empty() {
                                output_lines.push("OUTSIDE".to_string());
                                output_lines.push("".to_string());
                                for entry in outside_entries {
                                    output_lines.push(entry);
                                    output_lines.push("".to_string());
                                }
                            }

                            if !inside_entries.is_empty() {
                                output_lines.push("INSIDE".to_string());
                                output_lines.push("".to_string());
                                for entry in inside_entries {
                                    output_lines.push(entry);
                                    output_lines.push("".to_string());
                                }
                            }
                        }

                        output_lines.join("\n")
                    }
                }
            }
    };

    // --order / --order-percentage / --order-name / --order-random
    if let Some(op) = order_op {
        if file_paths.is_empty() {
            eprintln!("Error: --order* requires a file argument");
            std::process::exit(1);
        }
        for file_path in &file_paths {
            let path = PathBuf::from(file_path);
            let mut app = App::new(FormatMode::View);
            app.load_file(path.clone());
            match op {
                "order"            => app.order_entries(),
                "order-percentage" => app.order_by_percentage(),
                "order-name"       => app.order_by_name(),
                "order-random"     => app.order_random(),
                _ => unreachable!(),
            }
            // Write back (save_file uses app.file_path internally, already set by load_file)
            let output = if app.is_markdown_file() {
                app.markdown_input.clone()
            } else {
                app.json_input.clone()
            };
            fs::write(&path, output).unwrap_or_else(|e| {
                eprintln!("Error: Cannot write '{}': {}", file_path, e);
                std::process::exit(1);
            });
        }
        return Ok(());
    }

    // --append: read stdin, merge into file(s), write back in-place
    if append_mode {
        if file_paths.is_empty() {
            eprintln!("Error: --append requires a file argument");
            std::process::exit(1);
        }
        if !stdin_piped {
            eprintln!("Error: --append requires stdin input");
            std::process::exit(1);
        }
        let mut stdin_content = String::new();
        io::stdin().read_to_string(&mut stdin_content)?;

        // Parse stdin as JSON or Markdown using a temp app
        let tmp = App::new(format_mode);
        let stdin_json: serde_json::Value = if stdin_content.trim_start().starts_with('{') || stdin_content.trim_start().starts_with('[') {
            let v: serde_json::Value = match serde_json::from_str(&stdin_content) {
                Ok(v) => v,
                Err(e) => { eprintln!("Error: stdin is not valid JSON: {}", e); std::process::exit(1); }
            };
            // Validate: must be an object with at least one of "inside"/"outside" arrays
            if let Some(obj) = v.as_object() {
                let has_inside = obj.get("inside").and_then(|v| v.as_array()).is_some();
                let has_outside = obj.get("outside").and_then(|v| v.as_array()).is_some();
                if !has_inside && !has_outside {
                    eprintln!("Error: stdin JSON must be an object with \"inside\" and/or \"outside\" arrays");
                    eprintln!("  Expected: {{\"inside\": [...], \"outside\": [...]}}");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: stdin JSON must be an object with \"inside\" and/or \"outside\" arrays");
                eprintln!("  Expected: {{\"inside\": [...], \"outside\": [...]}}");
                std::process::exit(1);
            }
            v
        } else {
            // Markdown input: if --inside or --outside is specified and input lacks section headers,
            // auto-wrap the content with the appropriate header
            let section = if inside_only { Some("INSIDE") } else if outside_only { Some("OUTSIDE") } else { None };
            let processed = if let Some(sec) = section {
                if !stdin_content.contains("## OUTSIDE") && !stdin_content.contains("## INSIDE") {
                    format!("## {}\n{}", sec, stdin_content)
                } else {
                    stdin_content.clone()
                }
            } else {
                stdin_content.clone()
            };
            match tmp.parse_markdown(&processed) {
                Ok(json_str) => match serde_json::from_str(&json_str) {
                    Ok(v) => v,
                    Err(e) => { eprintln!("Error parsing stdin Markdown: {}", e); std::process::exit(1); }
                },
                Err(e) => { eprintln!("Error: stdin is not valid JSON or Markdown: {}", e); std::process::exit(1); }
            }
        };

        for file_path in &file_paths {
            let path = PathBuf::from(file_path);
            let mut app = App::new(format_mode);
            load_content(&mut app, fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error: Cannot read '{}': {}", file_path, e); std::process::exit(1);
            }), Some(path.clone()));

            let current: serde_json::Value = serde_json::from_str(&app.json_input).unwrap_or_else(|e| {
                eprintln!("Error: Invalid JSON in '{}': {}", file_path, e); std::process::exit(1);
            });

            let merged = json_ops::JsonOperations::append_entries(&current, &stdin_json, inside_only, outside_only);
            let output = serde_json::to_string_pretty(&merged).unwrap();

            if app.is_markdown_file() {
                // Write back as Markdown
                app.json_input = output;
                app.sync_markdown_from_json();
                fs::write(&path, &app.markdown_input).unwrap_or_else(|e| {
                    eprintln!("Error: Cannot write '{}': {}", file_path, e); std::process::exit(1);
                });
            } else {
                fs::write(&path, output).unwrap_or_else(|e| {
                    eprintln!("Error: Cannot write '{}': {}", file_path, e); std::process::exit(1);
                });
            }
        }
        return Ok(());
    }

    // --delete-outside-name / --delete-outside-context / --delete-inside-date / --delete-inside-context
    if let Some((op, pattern)) = delete_op {
        if file_paths.is_empty() {
            eprintln!("Error: --delete-* requires a file argument");
            std::process::exit(1);
        }
        for file_path in &file_paths {
            let path = PathBuf::from(file_path);
            let mut app = App::new(format_mode);
            load_content(&mut app, fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Error: Cannot read '{}': {}", file_path, e); std::process::exit(1);
            }), Some(path.clone()));

            let current: serde_json::Value = serde_json::from_str(&app.json_input).unwrap_or_else(|e| {
                eprintln!("Error: Invalid JSON in '{}': {}", file_path, e); std::process::exit(1);
            });

            let result = match op {
                "outside-name"    => json_ops::JsonOperations::delete_outside_by_name(&current, pattern),
                "outside-context" => json_ops::JsonOperations::delete_outside_by_context(&current, pattern),
                "inside-date"     => json_ops::JsonOperations::delete_inside_by_date(&current, pattern),
                "inside-context"  => json_ops::JsonOperations::delete_inside_by_context(&current, pattern),
                _ => unreachable!(),
            };
            let output = serde_json::to_string_pretty(&result).unwrap();

            if app.is_markdown_file() {
                app.json_input = output;
                app.sync_markdown_from_json();
                fs::write(&path, &app.markdown_input).unwrap_or_else(|e| {
                    eprintln!("Error: Cannot write '{}': {}", file_path, e); std::process::exit(1);
                });
            } else {
                fs::write(&path, output).unwrap_or_else(|e| {
                    eprintln!("Error: Cannot write '{}': {}", file_path, e); std::process::exit(1);
                });
            }
        }
        return Ok(());
    }

    // Helper: apply filter to app's json_input (and sync markdown if needed)
    let apply_filter_to_app = |app: &mut App| {
        if let Some(pattern) = &filter_pattern {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                let filtered = json_ops::JsonOperations::filter_entries(&json_val, pattern);
                if let Ok(s) = serde_json::to_string_pretty(&filtered) {
                    app.json_input = s;
                    if app.is_markdown_file() {
                        app.sync_markdown_from_json();
                    }
                }
            }
        }
    };

    // If token mode, show token counts and exit
    if token_mode {
        if file_paths.is_empty() && stdin_piped {
            let mut app = App::new(format_mode);
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            load_content(&mut app, content, None);
            apply_filter_to_app(&mut app);
            app.print_token_count();
        } else if file_paths.is_empty() {
            eprintln!("Error: No file specified for token count");
            std::process::exit(1);
        } else {
            for file_path in &file_paths {
                let path = PathBuf::from(file_path);
                let mut app = App::new(format_mode);
                app.load_file(path);
                apply_filter_to_app(&mut app);
                if file_paths.len() > 1 {
                    println!("=== {} ===", file_path);
                }
                app.print_token_count();
            }
        }
        return Ok(());
    }

    if stdout_mode || stdin_piped {
        if file_paths.is_empty() && stdin_piped {
            // Read from stdin
            let mut app = App::new(format_mode);
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            load_content(&mut app, content, None);
            println!("{}", generate_output(&app));
        } else if file_paths.is_empty() {
            eprintln!("Error: No input file specified and no stdin data");
            std::process::exit(1);
        } else {
            // Process each file
            for (idx, file_path) in file_paths.iter().enumerate() {
                let path = PathBuf::from(file_path);
                let content = fs::read_to_string(&path)
                    .map_err(|e| {
                        eprintln!("Error: Cannot read file '{}': {}", file_path, e);
                        std::process::exit(1);
                    })
                    .unwrap();
                let mut app = App::new(format_mode);
                load_content(&mut app, content, Some(path));
                if file_paths.len() > 1 {
                    if idx > 0 { println!(); }
                    println!("=== {} ===", file_path);
                }
                println!("{}", generate_output(&app));
            }
        }
    } else {
        // Interactive mode with better error handling
        let mut app = App::new(format_mode);

        // Load file if provided (first file only for interactive mode)
        if let Some(file_path) = file_paths.first() {
            let path = PathBuf::from(file_path);
            app.load_file(path);
        }

        // Pre-apply filter from --filter flag
        if let Some(pattern) = &filter_pattern {
            app.filter_pattern = pattern.to_string();
            app.convert_json();
        }

        // Set up terminal with error handling
        let setup_result = (|| -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
            enable_raw_mode()?;
            let mut stdout = stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            execute!(stdout, cursor::Hide)?;
            let backend = CrosstermBackend::new(stdout);
            Ok(Terminal::new(backend)?)
        })();

        let mut terminal = match setup_result {
            Ok(term) => term,
            Err(e) => {
                eprintln!("Failed to initialize terminal: {}", e);
                return Err(e);
            }
        };

        // Run the app with proper cleanup
        let res = input::run_app(&mut terminal, app);

        // Always clean up, even if there was an error
        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = execute!(terminal.backend_mut(), cursor::Show);
        let _ = terminal.show_cursor();

        if let Err(err) = res {
            eprintln!("Application error: {}", err);
            std::process::exit(1);
        }
    }

    Ok(())
}
