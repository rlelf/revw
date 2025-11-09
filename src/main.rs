mod app;
mod config;
mod content_ops;
mod input;
mod json_ops;
mod markdown_ops;
mod navigation;
mod rendering;
mod ui;

use anyhow::Result;
use clap::{Arg, Command};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs, io::stdout, panic, path::PathBuf};

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
        .arg(Arg::new("file").help("JSON file to view").index(1))
        .arg(
            Arg::new("json")
                .long("json")
                .help("Use JSON editing mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stdout")
                .long("stdout")
                .help("Output to stdout instead of interactive mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output to file (use '-' for stdout)")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("inside")
                .long("inside")
                .help("Output only INSIDE section")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("outside")
                .long("outside")
                .help("Output only OUTSIDE section")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("markdown")
                .long("markdown")
                .help("Output in Markdown format")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let format_mode = if matches.get_flag("json") {
        FormatMode::Edit
    } else {
        FormatMode::View
    };

    let stdout_mode = matches.get_flag("stdout");
    let output_file = matches.get_one::<String>("output");
    let inside_only = matches.get_flag("inside");
    let outside_only = matches.get_flag("outside");
    let markdown_mode = matches.get_flag("markdown");

    // If stdout mode or output file specified, run in non-interactive mode
    if stdout_mode || output_file.is_some() {
        let mut app = App::new(format_mode);

        // Load file if provided
        if let Some(file_path) = matches.get_one::<String>("file") {
            let path = PathBuf::from(file_path);
            let content = fs::read_to_string(&path)
                .map_err(|e| {
                    eprintln!("Error: Cannot read file '{}': {}", file_path, e);
                    std::process::exit(1);
                })
                .unwrap();

            app.json_input = content;
            app.convert_json();

            let output = if format_mode == FormatMode::Edit {
                // In Edit mode, output the JSON as-is
                app.json_input.clone()
            } else if markdown_mode {
                // Markdown mode: format entries as Markdown
                let mut output_lines = Vec::new();

                // Parse JSON to determine which section each entry belongs to
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                    if let Some(obj) = json_value.as_object() {
                        // OUTSIDE section
                        if !inside_only || outside_only {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                if !outside.is_empty() {
                                    output_lines.push("## OUTSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in outside {
                                        if let Some(item_obj) = item.as_object() {
                                            let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("name");
                                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                            let url = item_obj.get("url").and_then(|v| v.as_str());
                                            let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

                                            output_lines.push(format!("### {}", name));

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context = context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            // Only output URL if it's not null and not empty
                                            if let Some(url_str) = url {
                                                if !url_str.is_empty() {
                                                    output_lines.push(format!("#### URL: {}", url_str));
                                                }
                                            }

                                            // Only output percentage if it's not null
                                            if let Some(pct) = percentage {
                                                output_lines.push(format!("#### Percentage: {}%", pct));
                                            }

                                            output_lines.push("".to_string());
                                        }
                                    }
                                }
                            }
                        }

                        // INSIDE section
                        if !outside_only || inside_only {
                            if let Some(inside) = obj.get("inside").and_then(|v| v.as_array()) {
                                if !inside.is_empty() {
                                    output_lines.push("## INSIDE".to_string());
                                    output_lines.push("".to_string());

                                    for item in inside {
                                        if let Some(item_obj) = item.as_object() {
                                            let date = item_obj.get("date").and_then(|v| v.as_str()).unwrap_or("");
                                            let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");

                                            if !date.is_empty() {
                                                output_lines.push(format!("### {}", date));
                                            }

                                            // Replace literal \n with actual newlines in context
                                            if !context.is_empty() {
                                                let formatted_context = context.replace("\\n", "\n");
                                                output_lines.push(formatted_context);
                                            }

                                            output_lines.push("".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                output_lines.join("\n")
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

                    // Parse JSON to determine which section each entry belongs to
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&app.json_input) {
                        if let Some(obj) = json_value.as_object() {
                            if let Some(outside) = obj.get("outside").and_then(|v| v.as_array()) {
                                for item in outside {
                                    if let Some(item_obj) = item.as_object() {
                                        let name = item_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                        let context = item_obj.get("context").and_then(|v| v.as_str()).unwrap_or("");
                                        let url = item_obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
                                        let percentage = item_obj.get("percentage").and_then(|v| v.as_i64());

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
            };

            if let Some(output_path) = output_file {
                if output_path == "-" {
                    // Output to stdout
                    println!("{}", output);
                } else {
                    // Output to file
                    fs::write(output_path, output)?;
                }
            } else {
                // stdout flag was used
                println!("{}", output);
            }
        } else {
            eprintln!("Error: No input file specified");
            std::process::exit(1);
        }
    } else {
        // Interactive mode with better error handling
        let mut app = App::new(format_mode);

        // Load file if provided - no existence check for quick loading
        if let Some(file_path) = matches.get_one::<String>("file") {
            let path = PathBuf::from(file_path);
            app.load_file(path);
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
