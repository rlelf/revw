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
            revw file.json\n  \
            revw file.md\n\n  \
            # Output to stdout\n  \
            revw --stdout file.json\n  \
            revw --stdout file.md\n\n  \
            # Output in different formats\n  \
            revw --stdout --markdown file.json\n  \
            revw --stdout --json file.md\n\n  \
            # Pipe from stdin\n  \
            cat file.json | revw --stdout\n  \
            cat file.md | revw --stdout --markdown\n\n  \
            # Filter entries\n  \
            revw --stdout --filter pattern file.json\n  \
            revw --stdout --filter pattern file.md\n  \
            revw --stdout --filter pattern --inside file.json\n  \
            revw --stdout --filter pattern --markdown file.json\n\n\
            SUPPORTED FILE FORMATS:\n  \
            JSON (file.json):\n  \
            {\n    \
            \"outside\": [{\"name\": \"Resource\", \"context\": \"Description\", \"url\": \"https://...\", \"percentage\": 100}],\n    \
            \"inside\": [{\"date\": \"2025-01-01 00:00:00\", \"context\": \"Note content\"}]\n  \
            }\n\n  \
            Markdown (file.md):\n  \
            ## OUTSIDE\n  \
            ### Resource\n  \
            Description\n  \
            **URL:** https://...\n  \
            **Percentage:** 100%\n  \
            ## INSIDE\n  \
            ### 2025-01-01 00:00:00\n  \
            Note content\n\n\
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

    // If token mode, show token counts and exit
    if token_mode {
        if file_paths.is_empty() && stdin_piped {
            let mut app = App::new(format_mode);
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            load_content(&mut app, content, None);
            app.print_token_count();
        } else if file_paths.is_empty() {
            eprintln!("Error: No file specified for token count");
            std::process::exit(1);
        } else {
            for file_path in &file_paths {
                let path = PathBuf::from(file_path);
                let mut app = App::new(format_mode);
                app.load_file(path);
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
